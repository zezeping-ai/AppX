import type { Editor } from "@tiptap/vue-3";
import type { MenuProps } from "ant-design-vue";

export type FontFamilyOption = {
  key: string;
  label: string;
  /** null = 恢复默认；值为 CSS font-family 栈 */
  value: string | null;
};

/** 常用字体（含跨平台回退） */
export const FONT_FAMILIES: FontFamilyOption[] = [
  { key: "default", label: "默认", value: null },
  {
    key: "sans",
    label: "无衬线",
    value: 'system-ui, -apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif',
  },
  {
    key: "serif",
    label: "衬线",
    value: 'Georgia, "Times New Roman", "Songti SC", "SimSun", serif',
  },
  {
    key: "song",
    label: "宋体",
    value: '"Songti SC", "SimSun", "Noto Serif CJK SC", serif',
  },
  {
    key: "hei",
    label: "黑体",
    value: '"Heiti SC", "SimHei", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif',
  },
  {
    key: "kai",
    label: "楷体",
    value: '"Kaiti SC", "KaiTi", "STKaiti", serif',
  },
  {
    key: "fang",
    label: "仿宋",
    value: '"STFangsong", "FangSong", "FangSong_GB2312", serif',
  },
  {
    key: "mono",
    label: "等宽",
    value: 'ui-monospace, SFMono-Regular, Menlo, Consolas, "Courier New", monospace',
  },
];

export function currentFontFamily(editor: Editor): string | null {
  return (
    (editor.getAttributes("textStyle").fontFamily as string | undefined) ?? null
  );
}

export function createFontFamilyMenu(editor: Editor): MenuProps {
  const current = currentFontFamily(editor);

  return {
    selectedKeys: [
      FONT_FAMILIES.find((f) => f.value === current)?.key ?? "default",
    ],
    items: FONT_FAMILIES.map((f) => ({
      key: f.key,
      label: f.label,
      style: f.value ? { fontFamily: f.value } : undefined,
    })),
    onClick: ({ key }) => {
      const opt = FONT_FAMILIES.find((f) => f.key === key);
      if (!opt) return;
      const chain = editor.chain().focus();
      if (opt.value == null) {
        chain.unsetFontFamily().run();
        return;
      }
      chain.setFontFamily(opt.value).run();
    },
  };
}
