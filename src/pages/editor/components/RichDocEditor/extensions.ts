import Highlight from "@tiptap/extension-highlight";
import Link from "@tiptap/extension-link";
import Placeholder from "@tiptap/extension-placeholder";
import { TableKit } from "@tiptap/extension-table";
import TaskItem from "@tiptap/extension-task-item";
import TaskList from "@tiptap/extension-task-list";
import { TextStyleKit } from "@tiptap/extension-text-style";
import StarterKit from "@tiptap/starter-kit";
import type { Extensions } from "@tiptap/vue-3";
import { RichDocCodeBlock } from "./CodeBlock/extension";
import { RichDocLabel } from "./Label/extension";
import { RichDocLabelValue } from "./LabelValue/extension";
import { RichDocUnderline } from "./Underline/extension";
import { TableResizeGuard } from "./TableResizeGuard";

/**
 * 富文档 TipTap 扩展入口。
 * 新增节点/标记时：在对应目录实现 → 在此注册。
 */
export function createRichDocExtensions(): Extensions {
  return [
    StarterKit.configure({
      codeBlock: false,
    }),
    RichDocUnderline,
    Link.configure({
      openOnClick: false,
      autolink: true,
      defaultProtocol: "https",
    }),
    Highlight.configure({
      multicolor: true,
    }),
    TextStyleKit.configure({
      // 字色 / 字号 / 字体；背景色由 Highlight 承担
      backgroundColor: false,
      lineHeight: false,
    }),
    TaskList,
    TaskItem.configure({ nested: true }),
    RichDocCodeBlock,
    RichDocLabel,
    RichDocLabelValue,
    Placeholder.configure({
      placeholder: "开始写作…",
    }),
    TableKit.configure({
      table: { resizable: true },
    }),
    // 仅松手后清 resize 光标；不打断拖拽写宽
    TableResizeGuard,
  ];
}

export { RichDocCodeBlock, RichDocLabel, RichDocLabelValue, RichDocUnderline };
