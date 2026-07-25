import { mergeAttributes, Node } from "@tiptap/core";
import { NodeSelection } from "@tiptap/pm/state";
import { VueNodeViewRenderer } from "@tiptap/vue-3";
import { atomNodeViewStopEvent, requestAtomEnterEdit } from "../atomNodeNav";
import LabelValueView from "./index.vue";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    labelValue: {
      /** 插入标签值（Label 的扩展：标注 + 内容） */
      insertLabelValue: (attrs?: { label?: string; value?: string }) => ReturnType;
    };
  }
}

/** Label：行内 atom，左右均为 attrs；是 Label 的「标签 + 内容」扩展 */
export const RichDocLabelValue = Node.create({
  name: "labelValue",
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,
  // 拖拽整颗节点会与 input 内拖选冲突，编辑优先
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

  /** 纯文本：标签: 值 */
  renderText({ node }) {
    const label = String(node.attrs.label ?? "").trim();
    const value = String(node.attrs.value ?? "");
    if (!label) return value;
    if (!value) return label;
    return `${label}: ${value}`;
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
            // 仅插入后进编辑；方向键落到节点上不进输入框
            requestAtomEnterEdit(pos);
          }
          return true;
        },
    };
  },

  addKeyboardShortcuts() {
    return {
      // 选中整颗 atom：当作整体，方向键跳出，不进内部输入框
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
        // 交给默认上下移动，但确保焦点在编辑器而非 textarea
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
    return VueNodeViewRenderer(LabelValueView, {
      // 放行指针事件，才能拖选多个 atom（见 atomNodeViewStopEvent）
      stopEvent: atomNodeViewStopEvent,
    });
  },
});
