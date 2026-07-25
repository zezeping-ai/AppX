import type { Editor } from "@tiptap/core";
import type { Node as PmNode } from "@tiptap/pm/model";
import { TextSelection } from "@tiptap/pm/state";

/** 与表格 columnResizing handleWidth 对齐，略放宽便于命中 */
const TABLE_RESIZE_EDGE_PX = 8;

/**
 * atom 内 mousedown 默认 stopPropagation，避免 PM 抢焦点；
 * 但落在单元格左右边缘时必须冒泡，否则列宽拖拽无法开始（光标能变、拖不动）。
 */
export function onAtomFieldMouseDown(event: MouseEvent) {
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
  event.stopPropagation();
}

/** 将光标放到行内 atom 之前（可继续输入 / 再退格删前一字符） */
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
 * - 段落开头 → 与上一段合并（去掉换行，接到同一行）
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
