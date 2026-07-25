import type { Editor } from "@tiptap/core";
import { useClipboard, useClipboardItems } from "@vueuse/core";
import { DOMSerializer, type Node as PmNode } from "@tiptap/pm/model";
import { NodeSelection } from "@tiptap/pm/state";
import { toString } from "lodash-es";

/** Label 纯文本 */
export function labelPlainText(attrs: { label?: unknown }) {
  return toString(attrs.label ?? "");
}

/** LabelValue 纯文本：账号: xxx */
export function labelValuePlainText(attrs: {
  label?: unknown;
  value?: unknown;
}) {
  const label = toString(attrs.label ?? "").trim();
  const value = toString(attrs.value ?? "");
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

/** Cmd/Ctrl + key（不含 Alt/Shift） */
function isModChord(event: KeyboardEvent, key: string) {
  return (
    (event.metaKey || event.ctrlKey) &&
    !event.altKey &&
    !event.shiftKey &&
    event.key.toLowerCase() === key
  );
}

/** Cmd/Ctrl+C */
export function isCopyChord(event: KeyboardEvent) {
  return isModChord(event, "c");
}

/** Cmd/Ctrl+X */
export function isCutChord(event: KeyboardEvent) {
  return isModChord(event, "x");
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

/** 模块级单例：复用 vueuse 剪贴板能力（含 legacy execCommand） */
const { copy: copyClipboardItems, isSupported: clipboardItemsSupported } =
  useClipboardItems();
const { copy: copyClipboardText } = useClipboard({ legacy: true });

/** HTML + 纯文本；失败则退化为纯文本（含 execCommand） */
async function writeHtmlAndText(html: string, text: string): Promise<boolean> {
  if (clipboardItemsSupported.value && typeof ClipboardItem !== "undefined") {
    try {
      await copyClipboardItems([
        new ClipboardItem({
          "text/plain": new Blob([text], { type: "text/plain" }),
          "text/html": new Blob([html], { type: "text/html" }),
        }),
      ]);
      return true;
    } catch {
      /* 降级纯文本 */
    }
  }

  try {
    await copyClipboardText(text);
    return true;
  } catch {
    return false;
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
