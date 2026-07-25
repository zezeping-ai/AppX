import type { Editor } from "@tiptap/core";
import type { Node as PmNode } from "@tiptap/pm/model";
import { NodeSelection, TextSelection } from "@tiptap/pm/state";
import type { EditorView } from "@tiptap/pm/view";
import { useEventListener } from "@vueuse/core";
import { clamp, minBy } from "lodash-es";

/** 与表格 columnResizing handleWidth 对齐 */
export const TABLE_RESIZE_EDGE_PX = 8;

/** 拖动判定阈值（px） */
const DRAG_THRESHOLD_PX = 3;

/** 矩形邻近命中上限（px） */
const NEAR_ATOM_MAX_DIST_PX = 48;

export const ATOM_NAMES = new Set(["label", "labelValue"]);

const POINTER_PASS_EVENTS = new Set([
  "mousedown",
  "mouseup",
  "mousemove",
  "pointerdown",
  "pointerup",
  "pointermove",
  "click",
  "dblclick",
]);

/** 工具栏插入 / Enter 新建后：仅这些场景自动进内部编辑 */
let pendingEnterEditPos: number | null = null;

export function requestAtomEnterEdit(pos: number) {
  pendingEnterEditPos = pos;
}

export function takeAtomEnterEdit(pos: number): boolean {
  if (pendingEnterEditPos !== pos) {
    return false;
  }
  pendingEnterEditPos = null;
  return true;
}

let atomRangeSelecting = false;

export function isAtomRangeSelecting() {
  return atomRangeSelecting;
}

export function isAtomNode(node: PmNode | null | undefined): node is PmNode {
  return !!node && ATOM_NAMES.has(node.type.name);
}

function setRangeSelecting(view: EditorView, on: boolean) {
  atomRangeSelecting = on;
  view.dom.classList.toggle("atom-range-selecting", on);
}

function blurActiveAtomField() {
  const el = document.activeElement;
  if (el instanceof HTMLTextAreaElement || el instanceof HTMLInputElement) {
    el.blur();
  }
}

function rangeFromAtomDom(
  view: EditorView,
  atomEl: HTMLElement,
): { from: number; to: number } | null {
  try {
    const pos = view.posAtDOM(atomEl, 0);
    const node = view.state.doc.nodeAt(pos);
    if (isAtomNode(node)) {
      return { from: pos, to: pos + node.nodeSize };
    }
    const $pos = view.state.doc.resolve(pos);
    if (isAtomNode($pos.nodeAfter)) {
      return { from: pos, to: pos + $pos.nodeAfter.nodeSize };
    }
  } catch {
    /* DOM 已卸载 */
  }
  return null;
}

/**
 * elementFromPoint / 最近矩形定位 atom。
 * 不用裸 posAtCoords（textarea·NodeView 上常落到文档开头）。
 */
export function findAtomRangeAtPoint(
  view: EditorView,
  clientX: number,
  clientY: number,
): { from: number; to: number } | null {
  for (const el of document.elementsFromPoint(clientX, clientY)) {
    if (!(el instanceof HTMLElement)) {
      continue;
    }
    const atomEl = el.closest<HTMLElement>("[data-rich-atom]");
    if (!atomEl || !view.dom.contains(atomEl)) {
      continue;
    }
    const range = rangeFromAtomDom(view, atomEl);
    if (range) {
      return range;
    }
  }

  const near = [...view.dom.querySelectorAll<HTMLElement>("[data-rich-atom]")].flatMap(
    (atomEl) => {
      const rect = atomEl.getBoundingClientRect();
      const dist = Math.hypot(
        clientX - clamp(clientX, rect.left, rect.right),
        clientY - clamp(clientY, rect.top, rect.bottom),
      );
      if (dist > NEAR_ATOM_MAX_DIST_PX) {
        return [];
      }
      const range = rangeFromAtomDom(view, atomEl);
      return range ? [{ ...range, dist }] : [];
    },
  );
  const best = minBy(near, (c) => c.dist);
  return best ? { from: best.from, to: best.to } : null;
}

export function expandRangeToAtoms(
  doc: PmNode,
  from: number,
  to: number,
): { from: number; to: number } {
  let a = Math.min(from, to);
  let b = Math.max(from, to);
  if (a === b) {
    return { from: a, to: b };
  }
  doc.nodesBetween(a, b, (node, pos) => {
    if (isAtomNode(node)) {
      a = Math.min(a, pos);
      b = Math.max(b, pos + node.nodeSize);
    }
  });
  return { from: a, to: b };
}

/** 表格单元格左右边缘：留给列宽拖拽，不接管 */
export function isTableResizeEdge(event: MouseEvent): boolean {
  const cell = (event.target as HTMLElement | null)?.closest?.("td, th");
  if (!cell) {
    return false;
  }
  const rect = cell.getBoundingClientRect();
  return (
    rect.right - event.clientX <= TABLE_RESIZE_EDGE_PX ||
    event.clientX - rect.left <= TABLE_RESIZE_EDGE_PX
  );
}

/**
 * TipTap 默认会对 TEXTAREA 全量 stopEvent。
 * 指针事件放行（由 onAtomPointerMouseDown 接管选区）；
 * 字段内键盘 / 剪贴板 / 拖放交给原生，避免粘贴进文档。
 */
export function atomNodeViewStopEvent({ event }: { event: Event }): boolean {
  const target = event.target as HTMLElement | null;
  if (!target) {
    return false;
  }

  const { type } = event;
  const tag = target.tagName;
  const isButton = tag === "BUTTON" || !!target.closest?.("button");
  const isField =
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    !!target.isContentEditable;

  if (isButton && (type === "mousedown" || type === "mouseup" || type === "click")) {
    return true;
  }

  // 指针事件交给 ProseMirror / 自定义拖选
  if (POINTER_PASS_EVENTS.has(type)) {
    return false;
  }

  // 字段内：keydown/paste/copy/cut/drop… 全部 stop，走原生
  return isField;
}

/**
 * atom 按下：getPos 锚定起点，DOM 命中终点。
 * 拖动/Shift → 范围选；单击字段 → 编辑；单击外壳 → onClickSelectNode。
 */
export function onAtomPointerMouseDown(
  event: MouseEvent,
  editor: Editor,
  options: {
    getPos: () => number | undefined;
    nodeSize: number;
    editField?: HTMLTextAreaElement | HTMLInputElement | null;
    onClickSelectNode?: () => void;
  },
) {
  if (event.button !== 0 || isTableResizeEdge(event)) {
    return;
  }

  const editField = options.editField ?? null;
  // 已在编辑 / 双击三击：放行原生光标与选词，勿 preventDefault
  if (
    editField &&
    (document.activeElement === editField || event.detail >= 2)
  ) {
    event.stopPropagation();
    if (document.activeElement !== editField) {
      editField.focus();
    }
    return;
  }

  const startPos = options.getPos();
  if (typeof startPos !== "number") {
    return;
  }

  event.stopPropagation();
  event.preventDefault();

  blurActiveAtomField();
  editor.view.focus();
  setRangeSelecting(editor.view, true);

  const startFrom = startPos;
  const startTo = startPos + options.nodeSize;
  const { selection } = editor.state;
  const anchorFrom = event.shiftKey ? selection.anchor : startFrom;
  const anchorTo = event.shiftKey ? selection.anchor : startTo;
  const startX = event.clientX;
  const startY = event.clientY;
  let dragged = false;
  let lastEnd = { from: startFrom, to: startTo };

  const applyRange = (clientX: number, clientY: number) => {
    const endHit = findAtomRangeAtPoint(editor.view, clientX, clientY) ?? lastEnd;
    lastEnd = endHit;
    const rawFrom = Math.min(anchorFrom, anchorTo, endHit.from, endHit.to);
    const rawTo = Math.max(anchorFrom, anchorTo, endHit.from, endHit.to);
    const { from, to } = expandRangeToAtoms(editor.state.doc, rawFrom, rawTo);
    if (from >= to) {
      return;
    }
    const { state, view } = editor;
    try {
      const sel = TextSelection.create(state.doc, from, to);
      if (!sel.eq(state.selection)) {
        view.dispatch(state.tr.setSelection(sel));
      }
    } catch {
      /* 非法位置 */
    }
  };

  if (event.shiftKey) {
    applyRange(event.clientX, event.clientY);
  }

  const stopMove = useEventListener(
    window,
    "mousemove",
    (e: MouseEvent) => {
      e.preventDefault();
      if (!dragged) {
        if (Math.hypot(e.clientX - startX, e.clientY - startY) <= DRAG_THRESHOLD_PX) {
          return;
        }
        dragged = true;
      }
      blurActiveAtomField();
      if (document.activeElement !== editor.view.dom) {
        editor.view.focus();
      }
      applyRange(e.clientX, e.clientY);
    },
    { capture: true },
  );

  const stopUp = useEventListener(
    window,
    "mouseup",
    (e: MouseEvent) => {
      stopMove();
      stopUp();
      if (dragged || event.shiftKey) {
        applyRange(e.clientX, e.clientY);
        editor.view.focus();
        setRangeSelecting(editor.view, false);
        return;
      }
      setRangeSelecting(editor.view, false);
      if (editField) {
        const { state, view } = editor;
        view.dispatch(
          state.tr
            .setSelection(TextSelection.create(state.doc, startTo))
            .scrollIntoView(),
        );
        requestAnimationFrame(() => editField.focus());
      } else {
        options.onClickSelectNode?.();
      }
    },
    { capture: true },
  );
}

export function focusBeforeAtom(editor: Editor, pos: number) {
  const { state, view } = editor;
  view.dispatch(
    state.tr.setSelection(TextSelection.create(state.doc, pos)).scrollIntoView(),
  );
  view.focus();
}

export function focusAfterAtom(editor: Editor, pos: number, nodeSize: number) {
  const after = pos + nodeSize;
  const { state, view } = editor;
  view.dispatch(
    state.tr
      .setSelection(TextSelection.create(state.doc, after))
      .scrollIntoView(),
  );
  view.focus();
}

/** atom 行首 Backspace：合并上段 / 删前一字符 / 空则删自身 */
export function backspaceAtAtomStart(
  editor: Editor,
  pos: number,
  node: PmNode,
  empty: boolean,
): boolean {
  const { state, view } = editor;
  const $pos = state.doc.resolve(pos);

  if ($pos.parentOffset === 0) {
    view.dispatch(
      state.tr.setSelection(TextSelection.create(state.doc, pos)).scrollIntoView(),
    );
    view.focus();
    if (editor.commands.joinBackward()) {
      return true;
    }
    if (empty) {
      return editor
        .chain()
        .focus()
        .deleteRange({ from: pos, to: pos + node.nodeSize })
        .run();
    }
    return true;
  }

  const before = $pos.nodeBefore;
  if (!before) {
    focusBeforeAtom(editor, pos);
    return true;
  }
  const from = pos - before.nodeSize;
  return editor
    .chain()
    .focus()
    .deleteRange({ from, to: pos })
    .setTextSelection(from)
    .run();
}

/** Label / LabelValue 扩展共用的方向键与删除快捷键 */
export function atomNodeKeyboardShortcuts(nodeName: string) {
  const isThis = (editor: Editor) => {
    const { selection } = editor.state;
    return selection instanceof NodeSelection && selection.node.type.name === nodeName;
  };

  return {
    ArrowLeft: ({ editor }: { editor: Editor }) => {
      if (!isThis(editor)) {
        return false;
      }
      const { selection } = editor.state;
      return editor.commands.setTextSelection(selection.from);
    },
    ArrowRight: ({ editor }: { editor: Editor }) => {
      if (!isThis(editor)) {
        return false;
      }
      const { selection } = editor.state;
      return editor.commands.setTextSelection(selection.to);
    },
    ArrowUp: ({ editor }: { editor: Editor }) => {
      if (!isThis(editor)) {
        return false;
      }
      editor.view.focus();
      return false;
    },
    ArrowDown: ({ editor }: { editor: Editor }) => {
      if (!isThis(editor)) {
        return false;
      }
      editor.view.focus();
      return false;
    },
    Backspace: ({ editor }: { editor: Editor }) => {
      if (!isThis(editor)) {
        return false;
      }
      return editor.commands.deleteSelection();
    },
    Delete: ({ editor }: { editor: Editor }) => {
      if (!isThis(editor)) {
        return false;
      }
      return editor.commands.deleteSelection();
    },
  };
}
