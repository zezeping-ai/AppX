import type { Editor } from "@tiptap/core";
import { DOMSerializer, type Node as PmNode } from "@tiptap/pm/model";
import { NodeSelection } from "@tiptap/pm/state";

/** Label 纯文本 */
export function labelPlainText(attrs: { label?: unknown }) {
  return String(attrs.label ?? "");
}

/** LabelValue 纯文本：账号: xxx */
export function labelValuePlainText(attrs: {
  label?: unknown;
  value?: unknown;
}) {
  const label = String(attrs.label ?? "").trim();
  const value = String(attrs.value ?? "");
  if (!label) {
    return value;
  }
  if (!value) {
    return label;
  }
  return `${label}: ${value}`;
}

function atomPlainText(node: PmNode): string {
  if (node.type.name === "label") {
    return labelPlainText(node.attrs);
  }
  if (node.type.name === "labelValue") {
    return labelValuePlainText(node.attrs);
  }
  return node.textContent;
}

/** Cmd/Ctrl+C（不含 Shift） */
export function isCopyChord(event: KeyboardEvent) {
  return (
    (event.metaKey || event.ctrlKey) &&
    !event.altKey &&
    !event.shiftKey &&
    event.key.toLowerCase() === "c"
  );
}

/** Cmd/Ctrl+X */
export function isCutChord(event: KeyboardEvent) {
  return (
    (event.metaKey || event.ctrlKey) &&
    !event.altKey &&
    !event.shiftKey &&
    event.key.toLowerCase() === "x"
  );
}

export function selectAtomNode(editor: Editor, pos: number) {
  const { state, view } = editor;
  view.dispatch(
    state.tr
      .setSelection(NodeSelection.create(state.doc, pos))
      .scrollIntoView(),
  );
  view.focus();
}

async function writeHtmlAndText(html: string, text: string): Promise<boolean> {
  try {
    if (typeof ClipboardItem !== "undefined" && navigator.clipboard?.write) {
      await navigator.clipboard.write([
        new ClipboardItem({
          "text/plain": new Blob([text], { type: "text/plain" }),
          "text/html": new Blob([html], { type: "text/html" }),
        }),
      ]);
      return true;
    }
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    try {
      return document.execCommand("copy");
    } catch {
      return false;
    }
  }
}

/**
 * 将 atom 写入剪贴板（HTML + 纯文本）并选中该节点。
 * 粘贴回编辑器可还原；贴到外部走纯文本。
 */
export async function copyAtomNode(
  editor: Editor,
  pos: number,
  node: PmNode,
): Promise<boolean> {
  const { state, view } = editor;
  const selection = NodeSelection.create(state.doc, pos);
  const text = atomPlainText(node);

  const serializer = DOMSerializer.fromSchema(state.schema);
  const wrapper = document.createElement("div");
  serializer.serializeFragment(selection.content().content, { document }, wrapper);

  view.dispatch(state.tr.setSelection(selection).scrollIntoView());
  view.focus();

  return writeHtmlAndText(wrapper.innerHTML, text);
}

export async function cutAtomNode(
  editor: Editor,
  pos: number,
  node: PmNode,
): Promise<boolean> {
  const ok = await copyAtomNode(editor, pos, node);
  if (!ok) {
    return false;
  }
  return editor
    .chain()
    .focus()
    .deleteRange({ from: pos, to: pos + node.nodeSize })
    .run();
}
