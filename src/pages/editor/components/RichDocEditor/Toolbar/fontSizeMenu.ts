import type { Editor } from "@tiptap/vue-3";
import type { MenuProps } from "ant-design-vue";

export type FontSizeOption = {
  key: string;
  label: string;
  /** null = 恢复默认 */
  value: string | null;
};

export const FONT_SIZES: FontSizeOption[] = [
  { key: "default", label: "默认", value: null },
  { key: "12", label: "12", value: "12px" },
  { key: "14", label: "14", value: "14px" },
  { key: "16", label: "16", value: "16px" },
  { key: "18", label: "18", value: "18px" },
  { key: "20", label: "20", value: "20px" },
  { key: "24", label: "24", value: "24px" },
  { key: "28", label: "28", value: "28px" },
];

export function currentFontSize(editor: Editor): string | null {
  return (editor.getAttributes("textStyle").fontSize as string | undefined) ?? null;
}

export function createFontSizeMenu(editor: Editor): MenuProps {
  const current = currentFontSize(editor);

  return {
    selectedKeys: [
      FONT_SIZES.find((s) => s.value === current)?.key ?? "default",
    ],
    items: FONT_SIZES.map((s) => ({
      key: s.key,
      label: s.label,
    })),
    onClick: ({ key }) => {
      const opt = FONT_SIZES.find((s) => s.key === key);
      if (!opt) return;
      const chain = editor.chain().focus();
      if (opt.value == null) {
        chain.unsetFontSize().run();
        return;
      }
      chain.setFontSize(opt.value).run();
    },
  };
}
