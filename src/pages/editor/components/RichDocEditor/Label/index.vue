<script setup lang="ts">
import { NodeViewWrapper, nodeViewProps } from "@tiptap/vue-3";
import { NodeSelection } from "@tiptap/pm/state";
import { computed, nextTick, ref, watch } from "vue";
import {
  copyAtomNode,
  cutAtomNode,
  isCopyChord,
  isCutChord,
  selectAtomNode,
} from "../atomClipboard";
import {
  backspaceAtAtomStart,
  focusAfterAtom,
  focusBeforeAtom,
  onAtomPointerMouseDown,
  requestAtomEnterEdit,
  takeAtomEnterEdit,
} from "../atomNodeNav";

const props = defineProps(nodeViewProps);

const editable = computed(() => props.editor.isEditable);
const label = computed(() => String(props.node.attrs.label ?? ""));
const labelSizer = computed(() => label.value || "标签");
const inputRef = ref<HTMLTextAreaElement | null>(null);
const isEmpty = computed(() => !label.value);

/**
 * 仅「插入/Enter 新建」进编辑；方向键落到节点上不进 textarea。
 * 点击进编辑由 onAtomPointerMouseDown 直接 focus。
 */
watch(
  () => props.selected,
  async (selected) => {
    if (!selected || !editable.value) {
      return;
    }
    const pos = props.getPos();
    if (typeof pos !== "number" || !takeAtomEnterEdit(pos)) {
      return;
    }
    await nextTick();
    inputRef.value?.focus();
  },
  { immediate: true },
);

function onFieldMouseDown(event: MouseEvent) {
  const editField = (event.target as HTMLElement | null)?.closest?.(
    "textarea, input",
  ) as HTMLTextAreaElement | HTMLInputElement | null;
  onAtomPointerMouseDown(event, props.editor, {
    editField,
    getPos: () => {
      const pos = props.getPos();
      return typeof pos === "number" ? pos : undefined;
    },
    nodeSize: props.node.nodeSize,
  });
}

/** 外壳：拖选多个 / Shift 连选；单击整选不进编辑 */
function onShellMouseDown(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest?.("textarea, input, button")) {
    return;
  }
  onAtomPointerMouseDown(event, props.editor, {
    getPos: () => {
      const pos = props.getPos();
      return typeof pos === "number" ? pos : undefined;
    },
    nodeSize: props.node.nodeSize,
    onClickSelectNode: () => {
      const pos = props.getPos();
      if (typeof pos !== "number") {
        return;
      }
      selectAtomNode(props.editor, pos);
    },
  });
}

async function copyWholeAtom(event: KeyboardEvent) {
  const pos = props.getPos();
  if (typeof pos !== "number") {
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
  const pos = props.getPos();
  if (typeof pos !== "number") {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  await cutAtomNode(props.editor, pos, props.node);
}

function sanitize(raw: string) {
  return raw.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function onInput(event: Event) {
  const el = event.target as HTMLTextAreaElement;
  const next = sanitize(el.value);
  if (el.value !== next) {
    el.value = next;
  }
  props.updateAttributes({ label: next });
}

function focusEditorAfterNode() {
  const pos = props.getPos();
  if (typeof pos !== "number") {
    return;
  }
  focusAfterAtom(props.editor, pos, props.node.nodeSize);
}

function focusEditorBeforeNode() {
  const pos = props.getPos();
  if (typeof pos !== "number") {
    return;
  }
  focusBeforeAtom(props.editor, pos);
}

function insertSiblingAfter() {
  const pos = props.getPos();
  if (typeof pos !== "number") {
    return;
  }
  const after = pos + props.node.nodeSize;
  const next = props.editor.state.schema.nodes.label.create({ label: "" });
  const { state, view } = props.editor;
  let tr = state.tr.insert(after, next);
  tr = tr.setSelection(NodeSelection.create(tr.doc, after));
  requestAtomEnterEdit(after);
  view.dispatch(tr);
  view.focus();
}

function onKeydown(event: KeyboardEvent) {
  const input = event.target as HTMLTextAreaElement;
  const atStart = input.selectionStart === 0 && input.selectionEnd === 0;
  const collapsed = input.selectionStart === input.selectionEnd;

  // 光标未划选：Cmd/Ctrl+C 复制整颗；有划选则走浏览器复制字段文字
  if (isCopyChord(event) && collapsed) {
    void copyWholeAtom(event);
    return;
  }
  if (isCutChord(event) && collapsed) {
    void cutWholeAtom(event);
    return;
  }
  // Escape：整选节点（不进编辑），便于再 Cmd+C
  if (event.key === "Escape") {
    const pos = props.getPos();
    if (typeof pos === "number") {
      event.preventDefault();
      event.stopPropagation();
      selectAtomNode(props.editor, pos);
    }
    return;
  }

  // Enter：插入下一个标签；Shift+Enter：内部换行
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    event.stopPropagation();
    insertSiblingAfter();
    return;
  }
  if (event.key === "Tab" && !event.shiftKey) {
    event.preventDefault();
    event.stopPropagation();
    focusEditorAfterNode();
    return;
  }
  if (event.key === "Backspace" && atStart) {
    const pos = props.getPos();
    if (typeof pos !== "number") {
      return;
    }
    event.preventDefault();
    event.stopPropagation();
    backspaceAtAtomStart(props.editor, pos, props.node, isEmpty.value);
    return;
  }
  if (event.key === "ArrowLeft" && atStart) {
    event.preventDefault();
    event.stopPropagation();
    focusEditorBeforeNode();
    return;
  }
  if (event.key === "ArrowRight") {
    if (input.selectionStart === input.value.length && input.selectionEnd === input.value.length) {
      // 阻止冒泡，否则 PM 再右移一格会落到外侧第一个字「后面」
      event.preventDefault();
      event.stopPropagation();
      focusEditorAfterNode();
    }
  }
}
</script>

<template>
  <NodeViewWrapper
    as="span"
    class="rd-label"
    data-rich-atom="label"
    contenteditable="false"
    @mousedown="onShellMouseDown"
  >
    <span class="rd-label__cell">
      <span class="rd-label__sizer" aria-hidden="true">{{ labelSizer }}</span>
      <textarea
        ref="inputRef"
        class="rd-label__input"
        rows="1"
        cols="1"
        wrap="soft"
        tabindex="-1"
        :value="label"
        :readonly="!editable"
        placeholder="标签"
        @mousedown="onFieldMouseDown"
        @input="onInput"
        @keydown="onKeydown"
      />
    </span>
  </NodeViewWrapper>
</template>

<style scoped lang="scss">
.rd-label {
  display: inline-flex !important;
  align-items: stretch;
  width: max-content;
  max-width: 100%;
  min-height: 20px;
  height: auto;
  margin: 0 1px;
  padding: 0;
  border: 1px solid var(--app-border);
  border-radius: 4px;
  overflow: hidden;
  background: var(--app-surface-muted);
  vertical-align: middle;
  box-sizing: border-box;

  &.ProseMirror-selectednode {
    /* 与正文划选一致的浅蓝底，不要粗蓝描边 */
    outline: none;
    background: var(--app-active-bg);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-active-fg) 45%, transparent);
  }
}

.rd-label__cell {
  display: inline-grid;
  flex: 0 1 auto;
  min-width: 0;
  align-self: stretch;
  box-sizing: border-box;
}

.rd-label__sizer,
.rd-label__input {
  grid-area: 1 / 1;
  box-sizing: border-box;
  margin: 0;
  padding: 1px 6px;
  border: 0;
  font-family: inherit;
  font-size: 11px;
  font-weight: 500;
  line-height: 18px;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  word-break: break-word;
  overflow: hidden;
  color: var(--app-fg-muted);
}

.rd-label__sizer {
  visibility: hidden;
  pointer-events: none;
}

.rd-label__input {
  width: 100%;
  min-width: 0;
  height: 100%;
  resize: none;
  outline: none;
  appearance: none;
  background: transparent;
  caret-color: var(--app-fg);
  /* 未聚焦时禁止原生划选，避免拖选多个 atom 时被字段抢走 */
  user-select: none;

  &:focus {
    color: var(--app-fg-secondary);
    background: var(--app-hover-bg);
    user-select: text;
  }

  &:read-only {
    cursor: default;
  }

  &::placeholder {
    color: var(--app-fg-muted);
    opacity: 0.7;
  }
}
</style>
