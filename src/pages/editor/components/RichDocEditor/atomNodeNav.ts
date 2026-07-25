import type { Editor } from "@tiptap/core";
import type { Node as PmNode } from "@tiptap/pm/model";
import { NodeSelection, TextSelection } from "@tiptap/pm/state";
import type { EditorView } from "@tiptap/pm/view";

/** 与表格 columnResizing handleWidth 对齐，略放宽便于命中 */
const TABLE_RESIZE_EDGE_PX = 8;

const ATOM_NAMES = new Set(["label", "labelValue"]);

/** 工具栏插入 / Enter 新建后：仅这些场景自动进内部编辑 */
let pendingEnterEditPos: number | null = null;

/** 请求在指定 atom 位置进入内部编辑（插入后） */
export function requestAtomEnterEdit(pos: number) {
  pendingEnterEditPos = pos;
}

/** 若该位置有待进入编辑请求则消费并返回 true */
export function takeAtomEnterEdit(pos: number): boolean {
  if (pendingEnterEditPos !== pos) {
    return false;
  }
  pendingEnterEditPos = null;
  return true;
}

/** 正在拖选：禁止 textarea 抢焦点 / auto-focus */
let atomRangeSelecting = false;

export function isAtomRangeSelecting() {
  return atomRangeSelecting;
}

/** 是否「仅整选了这一颗」（可进编辑）；范围选区里的 selected 不算 */
export function isSoleAtomNodeSelection(
  editor: Editor,
  pos: number | undefined,
): boolean {
  if (typeof pos !== "number") {
    return false;
  }
  const { selection } = editor.state;
  return selection instanceof NodeSelection && selection.from === pos;
}

function isAtomNode(node: PmNode | null | undefined): node is PmNode {
  return !!node && ATOM_NAMES.has(node.type.name);
}

function setRangeSelecting(view: EditorView, on: boolean) {
  atomRangeSelecting = on;
  view.dom.classList.toggle("atom-range-selecting", on);
}

function blurActiveAtomField() {
  const el = document.activeElement;
  if (
    el instanceof HTMLTextAreaElement ||
    el instanceof HTMLInputElement
  ) {
    el.blur();
  }
}

/** 从 NodeView DOM 解析 atom 的文档位置 */
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
    /* DOM 已卸载等 */
  }
  return null;
}

/**
 * 用 elementFromPoint / 最近矩形定位 atom。
 * 不用裸 posAtCoords：在 textarea·NodeView 上常落到文档开头，造成「从上选到这」。
 */
function findAtomRangeAtPoint(
  view: EditorView,
  clientX: number,
  clientY: number,
): { from: number; to: number } | null {
  const stack = document.elementsFromPoint(clientX, clientY);
  for (const el of stack) {
    if (!(el instanceof HTMLElement)) {
      continue;
    }
    const atomEl = el.closest("[data-rich-atom]") as HTMLElement | null;
    if (!atomEl || !view.dom.contains(atomEl)) {
      continue;
    }
    const range = rangeFromAtomDom(view, atomEl);
    if (range) {
      return range;
    }
  }

  // 行间距空隙：取最近的 atom（垂直列表拖选）
  const atoms = view.dom.querySelectorAll<HTMLElement>("[data-rich-atom]");
  let best: { from: number; to: number; dist: number } | null = null;
  for (const atomEl of atoms) {
    const rect = atomEl.getBoundingClientRect();
    const clampedX = Math.max(rect.left, Math.min(clientX, rect.right));
    const clampedY = Math.max(rect.top, Math.min(clientY, rect.bottom));
    const dist = Math.hypot(clientX - clampedX, clientY - clampedY);
    if (dist > 48) {
      continue;
    }
    const range = rangeFromAtomDom(view, atomEl);
    if (!range) {
      continue;
    }
    if (!best || dist < best.dist) {
      best = { ...range, dist };
    }
  }
  return best;
}

function expandRangeToAtoms(
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

/**
 * TipTap 默认会对 TEXTAREA/INPUT 的所有事件 stopEvent。
 * 指针事件放行；从 atom 内起拖由 onAtomPointerMouseDown 自行维护选区。
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

  if (
    type === "mousedown" ||
    type === "mouseup" ||
    type === "mousemove" ||
    type === "pointerdown" ||
    type === "pointerup" ||
    type === "pointermove" ||
    type === "click" ||
    type === "dblclick"
  ) {
    return false;
  }

  if (
    type.startsWith("drag") ||
    type === "drop" ||
    type === "copy" ||
    type === "cut" ||
    type === "paste"
  ) {
    return false;
  }

  if (isField) {
    return true;
  }

  return false;
}

/**
 * atom 上按下：用 getPos 锚定起点（可靠），用 DOM 命中终点。
 * - 拖动 / Shift → 只覆盖拖过的那几颗
 * - 单击字段 → 进编辑且不留选中态
 * - 单击外壳 → onClickSelectNode
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
  if (event.button !== 0) {
    return;
  }

  const cell = (event.target as HTMLElement | null)?.closest?.("td, th");
  if (cell) {
    const rect = cell.getBoundingClientRect();
    if (
      rect.right - event.clientX <= TABLE_RESIZE_EDGE_PX ||
      event.clientX - rect.left <= TABLE_RESIZE_EDGE_PX
    ) {
      return;
    }
  }

  const startPos = options.getPos();
  if (typeof startPos !== "number") {
    return;
  }

  event.stopPropagation();
  event.preventDefault();

  // 立刻交还焦点并禁用字段指针，避免拖过其它行时 textarea 进入插入态
  blurActiveAtomField();
  editor.view.focus();
  setRangeSelecting(editor.view, true);

  const startFrom = startPos;
  const startTo = startPos + options.nodeSize;
  const { selection } = editor.state;

  /** Shift：以原锚点为一端；否则以当前 atom 整颗为端点 */
  const anchorFrom = event.shiftKey ? selection.anchor : startFrom;
  const anchorTo = event.shiftKey ? selection.anchor : startTo;

  const startX = event.clientX;
  const startY = event.clientY;
  let dragged = false;
  const editField = options.editField ?? null;
  let lastEnd = { from: startFrom, to: startTo };

  const applyRange = (clientX: number, clientY: number) => {
    const endHit =
      findAtomRangeAtPoint(editor.view, clientX, clientY) ?? lastEnd;
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
      /* 非法位置忽略 */
    }
  };

  if (event.shiftKey) {
    applyRange(event.clientX, event.clientY);
  }

  const onMove = (e: MouseEvent) => {
    e.preventDefault();
    if (!dragged) {
      if (Math.hypot(e.clientX - startX, e.clientY - startY) <= 3) {
        return;
      }
      dragged = true;
    }
    blurActiveAtomField();
    if (document.activeElement !== editor.view.dom) {
      editor.view.focus();
    }
    applyRange(e.clientX, e.clientY);
  };

  const onUp = (e: MouseEvent) => {
    window.removeEventListener("mousemove", onMove, true);
    window.removeEventListener("mouseup", onUp, true);

    if (dragged || event.shiftKey) {
      applyRange(e.clientX, e.clientY);
      editor.view.focus();
      setRangeSelecting(editor.view, false);
      return;
    }

    // 纯点击：结束拖选态后再进编辑
    setRangeSelecting(editor.view, false);

    if (editField) {
      const { state, view } = editor;
      view.dispatch(
        state.tr
          .setSelection(TextSelection.create(state.doc, startTo))
          .scrollIntoView(),
      );
      requestAnimationFrame(() => {
        editField.focus();
      });
    } else {
      options.onClickSelectNode?.();
    }
  };

  window.addEventListener("mousemove", onMove, true);
  window.addEventListener("mouseup", onUp, true);
}

/** 将光标放到行内 atom 之前 */
export function focusBeforeAtom(editor: Editor, pos: number) {
  const { state, view } = editor;
  view.dispatch(
    state.tr.setSelection(TextSelection.create(state.doc, pos)).scrollIntoView(),
  );
  view.focus();
}

/** 将光标放到行内 atom 之后 */
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

/**
 * 在 atom 内部「行首」按 Backspace：
 * - 段落开头 → 与上一段合并
 * - 前面还有内容 → 删除紧邻的前一个字符/节点
 * - 无法合并且自身为空 → 删除该 atom
 */
export function backspaceAtAtomStart(
  editor: Editor,
  pos: number,
  node: PmNode,
  isEmpty: boolean,
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
    if (isEmpty) {
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
