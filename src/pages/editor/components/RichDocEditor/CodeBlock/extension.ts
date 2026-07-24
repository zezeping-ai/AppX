import { CodeBlockLowlight } from "@tiptap/extension-code-block-lowlight";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import CodeBlockView from "./index.vue";
import { richDocLowlight } from "./helpers";

/** 带语雀式顶栏（语言 + 格式化）的代码块 */
export const RichDocCodeBlock = CodeBlockLowlight.extend({
  addNodeView() {
    return VueNodeViewRenderer(CodeBlockView);
  },

  addKeyboardShortcuts() {
    return {
      ...this.parent?.(),
      // 代码块内 Cmd/Ctrl+A：先选中本段；已全选本段后再放行文档全选
      "Mod-a": () => {
        const { state } = this.editor;
        const { $from, $to, from, to } = state.selection;
        if ($from.parent.type.name !== this.name || $from.parent !== $to.parent) {
          return false;
        }

        const start = $from.start();
        const end = $from.end();
        if (from === start && to === end) {
          return false;
        }

        return this.editor.commands.setTextSelection({ from: start, to: end });
      },
    };
  },
}).configure({
  lowlight: richDocLowlight,
  defaultLanguage: null,
});
