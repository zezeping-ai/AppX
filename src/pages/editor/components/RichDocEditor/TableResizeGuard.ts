import { Extension } from "@tiptap/core";
import type { EditorView } from "@tiptap/pm/view";
import { Plugin } from "@tiptap/pm/state";
import {
  cellAround,
  columnResizingPluginKey,
  TableMap,
} from "@tiptap/pm/tables";

/** 与 Table 默认 handleWidth 对齐，略放宽便于首击命中 */
const HANDLE_WIDTH = 6;

type ResizePluginState = {
  activeHandle: number;
  dragging: unknown;
};

function getResizeState(view: EditorView): ResizePluginState | null {
  return (
    (columnResizingPluginKey.getState(view.state) as ResizePluginState | undefined) ??
    null
  );
}

function domCellAround(target: EventTarget | null): HTMLElement | null {
  let el = target as Node | null;
  while (el) {
    if (el instanceof HTMLElement) {
      const name = el.nodeName;
      if (name === "TD" || name === "TH") return el;
      if (el.classList.contains("tiptap") || el.classList.contains("ProseMirror")) {
        return null;
      }
    }
    el = el.parentNode;
  }
  return null;
}

/** 对齐 prosemirror-tables edgeCell */
function edgeCell(
  view: EditorView,
  event: MouseEvent,
  side: "left" | "right",
): number {
  const offset = side === "right" ? -HANDLE_WIDTH : HANDLE_WIDTH;
  const found = view.posAtCoords({
    left: event.clientX + offset,
    top: event.clientY,
  });
  if (!found) return -1;

  const $cell = cellAround(view.state.doc.resolve(found.pos));
  if (!$cell) return -1;
  if (side === "right") return $cell.pos;

  const map = TableMap.get($cell.node(-1));
  const start = $cell.start(-1);
  const index = map.map.indexOf($cell.pos - start);
  return index % map.width === 0 ? -1 : start + map.map[index - 1]!;
}

/**
 * 首击表格线时先写入 activeHandle，让同一次 mousedown 能开始拖。
 */
function ensureResizeHandleOnMouseDown(view: EditorView, event: MouseEvent) {
  if (!view.editable) return;
  const resize = getResizeState(view);
  if (!resize || resize.dragging) return;
  if (resize.activeHandle > -1) return;

  const cellEl = domCellAround(event.target);
  if (!cellEl) return;

  const { left, right } = cellEl.getBoundingClientRect();
  let cell = -1;
  if (event.clientX - left <= HANDLE_WIDTH) {
    cell = edgeCell(view, event, "left");
  } else if (right - event.clientX <= HANDLE_WIDTH) {
    cell = edgeCell(view, event, "right");
  }
  if (cell < 0) return;

  view.dispatch(
    view.state.tr.setMeta(columnResizingPluginKey, { setHandle: cell }),
  );
}

function clearResizeHandle(view: EditorView, focus: boolean) {
  const resize = getResizeState(view);
  if (!resize) return;
  if (resize.activeHandle < 0 && !resize.dragging) return;

  view.dispatch(
    view.state.tr.setMeta(columnResizingPluginKey, { setHandle: -1 }),
  );
  if (focus) {
    view.focus();
  }
}

/**
 * 表格列宽拖拽体验修补：
 * 1) 首击边缘即可开始拖
 * 2) 松手后强制清掉 dragging / activeHandle
 *    （官方只清 dragging；activeHandle 残留时，别处按下仍会继续改列宽）
 */
export const TableResizeGuard = Extension.create({
  name: "tableResizeGuard",

  addProseMirrorPlugins() {
    return [
      new Plugin({
        props: {
          handleKeyDown(view, event) {
            if (event.key !== "Escape") return false;
            const resize = getResizeState(view);
            if (!resize || (resize.activeHandle < 0 && !resize.dragging)) {
              return false;
            }
            clearResizeHandle(view, true);
            return true;
          },
        },
        view(editorView) {
          /** 本轮是否真正进入过 dragging，用于松手后决定是否抢回焦点 */
          let dragSession = false;

          const onMouseDownCapture = (event: MouseEvent) => {
            if (event.button !== 0) return;
            ensureResizeHandleOnMouseDown(editorView, event);
            queueMicrotask(() => {
              if (getResizeState(editorView)?.dragging) {
                dragSession = true;
              }
            });
          };

          /**
           * 等官方 mouseup → finish 写完列宽后再清。
           * 必须清 activeHandle：残留时别处 mousedown 仍会进入改列宽。
           * pointerup 可能早于 mouseup，故用 setTimeout(0) 排到 finish 之后。
           */
          const releaseResize = () => {
            window.setTimeout(() => {
              const resize = getResizeState(editorView);
              if (!resize) return;
              if (!resize.dragging && resize.activeHandle < 0) {
                dragSession = false;
                return;
              }
              const focus = dragSession || Boolean(resize.dragging);
              dragSession = false;
              clearResizeHandle(editorView, focus);
            }, 0);
          };

          editorView.dom.addEventListener("mousedown", onMouseDownCapture, true);
          window.addEventListener("mouseup", releaseResize);
          window.addEventListener("pointerup", releaseResize);
          window.addEventListener("pointercancel", releaseResize);

          return {
            destroy() {
              editorView.dom.removeEventListener(
                "mousedown",
                onMouseDownCapture,
                true,
              );
              window.removeEventListener("mouseup", releaseResize);
              window.removeEventListener("pointerup", releaseResize);
              window.removeEventListener("pointercancel", releaseResize);
            },
          };
        },
      }),
    ];
  },
});
