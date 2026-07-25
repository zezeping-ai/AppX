import type { Editor } from "@tiptap/vue-3";

export type ColorOption = {
  key: string;
  label: string;
  /** null = 恢复默认 */
  value: string | null;
};

/** 字色 / 下划线色预设 */
export const TEXT_COLORS: ColorOption[] = [
  { key: "default", label: "默认", value: null },
  { key: "slate", label: "灰", value: "#64748b" },
  { key: "red", label: "红", value: "#dc2626" },
  { key: "orange", label: "橙", value: "#ea580c" },
  { key: "amber", label: "黄", value: "#ca8a04" },
  { key: "green", label: "绿", value: "#16a34a" },
  { key: "teal", label: "青", value: "#0d9488" },
  { key: "blue", label: "蓝", value: "#2563eb" },
  { key: "violet", label: "紫", value: "#7c3aed" },
  { key: "pink", label: "粉", value: "#db2777" },
];

/** 高亮色预设（浅底） */
export const HIGHLIGHT_COLORS: ColorOption[] = [
  { key: "none", label: "无", value: null },
  { key: "yellow", label: "黄", value: "#fef08a" },
  { key: "green", label: "绿", value: "#bbf7d0" },
  { key: "blue", label: "蓝", value: "#bfdbfe" },
  { key: "pink", label: "粉", value: "#fbcfe8" },
  { key: "orange", label: "橙", value: "#fed7aa" },
];

export function currentTextColor(editor: Editor): string | null {
  return (editor.getAttributes("textStyle").color as string | undefined) ?? null;
}

export function currentHighlightColor(editor: Editor): string | null {
  if (!editor.isActive("highlight")) return null;
  return (editor.getAttributes("highlight").color as string | undefined) ?? "#fef08a";
}

export function currentUnderlineColor(editor: Editor): string | null {
  if (!editor.isActive("underline")) return null;
  return (editor.getAttributes("underline").color as string | undefined) ?? null;
}

export function applyTextColor(editor: Editor, value: string | null) {
  const chain = editor.chain().focus();
  if (value == null) {
    chain.unsetColor().run();
    return;
  }
  chain.setColor(value).run();
}

export function applyHighlight(editor: Editor, value: string | null) {
  const chain = editor.chain().focus();
  if (value == null) {
    chain.unsetHighlight().run();
    return;
  }
  chain.setHighlight({ color: value }).run();
}

export function applyUnderlineColor(editor: Editor, value: string | null) {
  editor.chain().focus().setUnderlineColor(value).run();
}
