import { mergeAttributes, Node } from "@tiptap/core";
import { NodeSelection } from "@tiptap/pm/state";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import {
  atomNodeKeyboardShortcuts,
  atomNodeViewStopEvent,
  labelPlainText,
  requestAtomEnterEdit,
} from "../atom";
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

  renderText({ node }) {
    return labelPlainText(node.attrs);
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
    return atomNodeKeyboardShortcuts(this.name);
  },

  addNodeView() {
    return VueNodeViewRenderer(LabelView, {
      stopEvent: atomNodeViewStopEvent,
    });
  },
});
