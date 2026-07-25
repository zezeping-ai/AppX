import type { Editor } from "@tiptap/vue-3";
import { toggleBlockquoteSmart } from "./toggleBlockquote";
import type { ToolbarItem } from "./types";

/** 设置/编辑选区链接；清空地址则取消 */
function toggleLink(editor: Editor) {
  const chain = () => editor.chain().focus();
  const prev = (editor.getAttributes("link").href as string | undefined) ?? "";
  const url = window.prompt(
    editor.isActive("link") ? "链接地址（清空则取消）" : "链接地址",
    prev || "https://",
  );
  if (url == null) return;
  if (url.trim() === "") {
    chain().extendMarkRange("link").unsetLink().run();
    return;
  }
  chain().extendMarkRange("link").setLink({ href: url.trim() }).run();
}

/**
 * 工具栏项（对齐 Word「字体」组习惯）：
 * 撤销 → 粗斜下划删除 → 字体/字号 → 字色/高亮 → 清除 → …
 */
export function createFormatItems(editor: Editor): ToolbarItem[] {
  const chain = () => editor.chain().focus();

  return [
    {
      kind: "btn",
      title: "撤销",
      icon: "mdi:undo",
      disabled: () => !editor.can().undo(),
      run: () => chain().undo().run(),
    },
    {
      kind: "btn",
      title: "重做",
      icon: "mdi:redo",
      disabled: () => !editor.can().redo(),
      run: () => chain().redo().run(),
    },
    { kind: "sep" },
    {
      kind: "btn",
      title: "加粗",
      icon: "mdi:format-bold",
      active: () => editor.isActive("bold"),
      run: () => chain().toggleBold().run(),
    },
    {
      kind: "btn",
      title: "斜体",
      icon: "mdi:format-italic",
      active: () => editor.isActive("italic"),
      run: () => chain().toggleItalic().run(),
    },
    { kind: "underlineMenu" },
    {
      kind: "btn",
      title: "删除线",
      icon: "mdi:format-strikethrough-variant",
      active: () => editor.isActive("strike"),
      run: () => chain().toggleStrike().run(),
    },
    {
      kind: "btn",
      title: "行内代码",
      icon: "mdi:code-brackets",
      active: () => editor.isActive("code"),
      run: () => chain().toggleCode().run(),
    },
    { kind: "sep" },
    { kind: "fontFamilyMenu" },
    { kind: "fontSizeMenu" },
    { kind: "sep" },
    { kind: "colorMenu" },
    { kind: "highlightMenu" },
    {
      kind: "btn",
      title: "清除格式",
      icon: "mdi:eraser",
      run: () => {
        // 勿与 removeEmptyTextStyle 硬链：后者无空 span 时返回 false 会整链回滚
        chain().unsetAllMarks().run();
        chain().removeEmptyTextStyle().run();
      },
    },
    { kind: "sep" },
    {
      kind: "btn",
      title: "标题 1",
      label: "H1",
      active: () => editor.isActive("heading", { level: 1 }),
      run: () => chain().toggleHeading({ level: 1 }).run(),
    },
    {
      kind: "btn",
      title: "标题 2",
      label: "H2",
      active: () => editor.isActive("heading", { level: 2 }),
      run: () => chain().toggleHeading({ level: 2 }).run(),
    },
    {
      kind: "btn",
      title: "标题 3",
      label: "H3",
      active: () => editor.isActive("heading", { level: 3 }),
      run: () => chain().toggleHeading({ level: 3 }).run(),
    },
    { kind: "sep" },
    {
      kind: "btn",
      title: "无序列表",
      icon: "mdi:format-list-bulleted",
      active: () => editor.isActive("bulletList"),
      run: () => chain().toggleBulletList().run(),
    },
    {
      kind: "btn",
      title: "有序列表",
      icon: "mdi:format-list-numbered",
      active: () => editor.isActive("orderedList"),
      run: () => chain().toggleOrderedList().run(),
    },
    {
      kind: "btn",
      title: "待办列表",
      icon: "mdi:format-list-checks",
      active: () => editor.isActive("taskList"),
      run: () => chain().toggleTaskList().run(),
    },
    {
      kind: "btn",
      title: "引用",
      icon: "mdi:format-quote-close",
      active: () => editor.isActive("blockquote"),
      run: () => toggleBlockquoteSmart(editor),
    },
    {
      kind: "btn",
      title: "代码块",
      icon: "mdi:code-tags",
      active: () => editor.isActive("codeBlock"),
      run: () => chain().toggleCodeBlock().run(),
    },
    {
      kind: "btn",
      title: "分割线",
      icon: "mdi:minus",
      run: () => chain().setHorizontalRule().run(),
    },
    { kind: "labelMenu" },
    { kind: "sep" },
    {
      kind: "btn",
      title: "链接",
      icon: "mdi:link-variant",
      active: () => editor.isActive("link"),
      run: () => toggleLink(editor),
    },
    { kind: "tableMenu" },
  ];
}
