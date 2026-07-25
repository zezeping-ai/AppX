import { mergeAttributes, Node } from "@tiptap/core";
import { NodeSelection } from "@tiptap/pm/state";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import {
  atomNodeKeyboardShortcuts,
  atomNodeViewStopEvent,
  labelValuePlainText,
  requestAtomEnterEdit,
} from "../atom";
import LabelValueView from "./index.vue";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    labelValue: {
      /** 插入标签值（Label 的扩展：标注 + 内容） */
      insertLabelValue: (attrs?: { label?: string; value?: string }) => ReturnType;
    };
  }
}

/** LabelValue：行内 atom，左右均为 attrs */
export const RichDocLabelValue = Node.create({
  name: "labelValue",
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,
  draggable: false,

  addAttributes() {
    return {
      label: {
        default: "",
        parseHTML: (element) => element.getAttribute("data-label") ?? "",
        renderHTML: (attributes) => ({
          "data-label": attributes.label ?? "",
        }),
      },
      value: {
        default: "",
        parseHTML: (element) => element.getAttribute("data-value") ?? "",
        renderHTML: (attributes) => ({
          "data-value": attributes.value ?? "",
        }),
      },
    };
  },

  parseHTML() {
    return [{ tag: 'span[data-type="label-value"]' }];
  },

  renderHTML({ node, HTMLAttributes }) {
    const label = String(node.attrs.label || "标签");
    const value = String(node.attrs.value || "值");
    return [
      "span",
      mergeAttributes(HTMLAttributes, {
        "data-type": "label-value",
        class: "lv",
      }),
      ["span", { class: "lv__label" }, label],
      ["span", { class: "lv__value" }, value],
    ];
  },

  renderText({ node }) {
    return labelValuePlainText(node.attrs);
  },

  addCommands() {
    return {
      insertLabelValue:
        (attrs) =>
        ({ state, tr, dispatch }) => {
          const node = state.schema.nodes.labelValue.create({
            label: attrs?.label?.trim() || "",
            value: attrs?.value ?? "",
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
    return VueNodeViewRenderer(LabelValueView, {
      stopEvent: atomNodeViewStopEvent,
    });
  },
});
