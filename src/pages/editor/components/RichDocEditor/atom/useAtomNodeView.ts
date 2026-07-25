import type { Editor } from "@tiptap/core";
import type { Node as PmNode } from "@tiptap/pm/model";
import type { NodeViewProps } from "@tiptap/vue-3";
import { NodeSelection } from "@tiptap/pm/state";
import { computed, nextTick, watch, type Ref } from "vue";
import {
  copyAtomNode,
  cutAtomNode,
  isCopyChord,
  isCutChord,
  selectAtomNode,
} from "./clipboard";
import {
  backspaceAtAtomStart,
  focusAfterAtom,
  focusBeforeAtom,
  onAtomPointerMouseDown,
  requestAtomEnterEdit,
  takeAtomEnterEdit,
} from "./pointer";

/** 统一换行符 */
export function sanitizeAtomFieldValue(raw: string) {
  return raw.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

export function resolveAtomPos(getPos: NodeViewProps["getPos"]): number | undefined {
  const pos = getPos();
  return typeof pos === "number" ? pos : undefined;
}

type AtomKind = "label" | "labelValue";

/**
 * Label / LabelValue NodeView 共用：选区、剪贴板、指针、进出编辑。
 */
export function useAtomNodeView(
  props: NodeViewProps,
  options: {
    kind: AtomKind;
    /** 插入后应聚焦的输入框 */
    enterEditTarget: Ref<HTMLTextAreaElement | null>;
  },
) {
  const editable = computed(() => props.editor.isEditable);

  watch(
    () => props.selected,
    async (selected) => {
      if (!selected || !editable.value) {
        return;
      }
      const pos = resolveAtomPos(props.getPos);
      if (pos === undefined || !takeAtomEnterEdit(pos)) {
        return;
      }
      await nextTick();
      options.enterEditTarget.value?.focus();
    },
    { immediate: true },
  );

  function getPosSafe() {
    return resolveAtomPos(props.getPos);
  }

  function onFieldMouseDown(event: MouseEvent) {
    const editField = (event.target as HTMLElement | null)?.closest?.(
      "textarea, input",
    ) as HTMLTextAreaElement | HTMLInputElement | null;
    onAtomPointerMouseDown(event, props.editor, {
      editField,
      getPos: getPosSafe,
      nodeSize: props.node.nodeSize,
    });
  }

  function onShellMouseDown(event: MouseEvent) {
    const target = event.target as HTMLElement | null;
    if (target?.closest?.("textarea, input, button")) {
      return;
    }
    onAtomPointerMouseDown(event, props.editor, {
      getPos: getPosSafe,
      nodeSize: props.node.nodeSize,
      onClickSelectNode: () => {
        const pos = getPosSafe();
        if (pos === undefined) {
          return;
        }
        selectAtomNode(props.editor, pos);
      },
    });
  }

  async function copyWholeAtom(event: KeyboardEvent) {
    const pos = getPosSafe();
    if (pos === undefined) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    await copyAtomNode(props.editor, pos, props.node);
  }

  async function cutWholeAtom(event: KeyboardEvent) {
    if (!editable.value) {
      return;
    }
    const pos = getPosSafe();
    if (pos === undefined) {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    await cutAtomNode(props.editor, pos, props.node);
  }

  /** textarea 内：折叠光标时复制/剪切整颗；Esc 退出编辑 */
  function handleClipboardKeys(
    event: KeyboardEvent,
    input: HTMLTextAreaElement,
  ): boolean {
    const collapsed = input.selectionStart === input.selectionEnd;
    if (isCopyChord(event) && collapsed) {
      void copyWholeAtom(event);
      return true;
    }
    if (isCutChord(event) && collapsed) {
      void cutWholeAtom(event);
      return true;
    }
    if (event.key === "Escape") {
      const pos = getPosSafe();
      if (pos !== undefined) {
        event.preventDefault();
        event.stopPropagation();
        selectAtomNode(props.editor, pos);
      }
      return true;
    }
    return false;
  }

  function updateAttr(key: string, event: Event) {
    const el = event.target as HTMLTextAreaElement;
    const next = sanitizeAtomFieldValue(el.value);
    if (el.value !== next) {
      el.value = next;
    }
    props.updateAttributes({ [key]: next });
  }

  function focusEditorAfterNode() {
    const pos = getPosSafe();
    if (pos === undefined) {
      return;
    }
    focusAfterAtom(props.editor, pos, props.node.nodeSize);
  }

  function focusEditorBeforeNode() {
    const pos = getPosSafe();
    if (pos === undefined) {
      return;
    }
    focusBeforeAtom(props.editor, pos);
  }

  function backspaceAtStart(isEmpty: boolean) {
    const pos = getPosSafe();
    if (pos === undefined) {
      return;
    }
    backspaceAtAtomStart(props.editor, pos, props.node, isEmpty);
  }

  function insertSiblingAfter(createNode: (editor: Editor) => PmNode) {
    const pos = getPosSafe();
    if (pos === undefined) {
      return;
    }
    const after = pos + props.node.nodeSize;
    const next = createNode(props.editor);
    const { state, view } = props.editor;
    let tr = state.tr.insert(after, next);
    tr = tr.setSelection(NodeSelection.create(tr.doc, after));
    requestAtomEnterEdit(after);
    view.dispatch(tr);
    view.focus();
  }

  function isCaretAtStart(input: HTMLTextAreaElement) {
    return input.selectionStart === 0 && input.selectionEnd === 0;
  }

  function isCaretAtEnd(input: HTMLTextAreaElement) {
    const len = input.value.length;
    return input.selectionStart === len && input.selectionEnd === len;
  }

  return {
    editable,
    getPosSafe,
    onFieldMouseDown,
    onShellMouseDown,
    handleClipboardKeys,
    updateAttr,
    focusEditorAfterNode,
    focusEditorBeforeNode,
    backspaceAtStart,
    insertSiblingAfter,
    isCaretAtStart,
    isCaretAtEnd,
    kind: options.kind,
  };
}
