import { mergeAttributes, Node } from "@tiptap/core";
import { NodeSelection } from "@tiptap/pm/state";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import { atomNodeViewStopEvent, requestAtomEnterEdit } from "../atomNodeNav";
import LabelView from "./index.vue";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    label: {
      /** 插入行内标签（LabelValue 的基础形态） */
      insertLabel: (attrs?: { label?: string }) => ReturnType;
    };
  }
}

/** Label：行内 atom 小标签；LabelValue 是其「标签 + 内容」扩展 */
export const RichDocLabel = Node.create({
  name: "label",
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,
  draggable: false,

  addAttributes() {
    return {
      label: {
        default: "",
        parseHTML: (element) =>
          element.getAttribute("data-label") ?? element.getAttribute("data-text") ?? "",
        renderHTML: (attributes) => ({
          "data-label": attributes.label ?? "",
        }),
      },
    };
  },

  parseHTML() {
    return [
      { tag: 'span[data-type="label"]' },
      // 旧 Tag 节点 HTML
      { tag: 'span[data-type="tag"]' },
    ];
  },

  renderHTML({ node, HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(HTMLAttributes, {
        "data-type": "label",
        class: "rd-label",
      }),
      String(node.attrs.label || "标签"),
    ];
  },

  /** 纯文本剪贴板 / 搜索 */
  renderText({ node }) {
    return String(node.attrs.label ?? "");
  },

  addCommands() {
    return {
      insertLabel:
        (attrs) =>
        ({ state, tr, dispatch }) => {
          const node = state.schema.nodes.label.create({
            label: attrs?.label ?? "",
          });
          if (!dispatch) {
            return true;
          }
          tr.replaceSelectionWith(node, false);
          const pos = tr.selection.from - node.nodeSize;
          if (pos >= 0) {
            tr.setSelection(NodeSelection.create(tr.doc, pos));
            requestAtomEnterEdit(pos);
          }
          return true;
        },
    };
  },

  addKeyboardShortcuts() {
    return {
      ArrowLeft: ({ editor }) => {
        const { selection } = editor.state;
        if (!(selection instanceof NodeSelection) || selection.node.type.name !== this.name) {
          return false;
        }
        return editor.commands.setTextSelection(selection.from);
      },
      ArrowRight: ({ editor }) => {
        const { selection } = editor.state;
        if (!(selection instanceof NodeSelection) || selection.node.type.name !== this.name) {
          return false;
        }
        return editor.commands.setTextSelection(selection.to);
      },
      ArrowUp: ({ editor }) => {
        const { selection } = editor.state;
        if (!(selection instanceof NodeSelection) || selection.node.type.name !== this.name) {
          return false;
        }
        editor.view.focus();
        return false;
      },
      ArrowDown: ({ editor }) => {
        const { selection } = editor.state;
        if (!(selection instanceof NodeSelection) || selection.node.type.name !== this.name) {
          return false;
        }
        editor.view.focus();
        return false;
      },
      Backspace: ({ editor }) => {
        const { selection } = editor.state;
        if (!(selection instanceof NodeSelection) || selection.node.type.name !== this.name) {
          return false;
        }
        return editor.commands.deleteSelection();
      },
      Delete: ({ editor }) => {
        const { selection } = editor.state;
        if (!(selection instanceof NodeSelection) || selection.node.type.name !== this.name) {
          return false;
        }
        return editor.commands.deleteSelection();
      },
    };
  },

  addNodeView() {
    return VueNodeViewRenderer(LabelView, {
      // 放行指针事件，才能拖选多个 atom（见 atomNodeViewStopEvent）
      stopEvent: atomNodeViewStopEvent,
    });
  },
});
