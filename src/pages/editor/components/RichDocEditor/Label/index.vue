<script setup lang="ts">
import { NodeViewWrapper, nodeViewProps } from "@tiptap/vue-3";
import { NodeSelection, TextSelection } from "@tiptap/pm/state";
import { computed, nextTick, ref, watch } from "vue";

const props = defineProps(nodeViewProps);

const editable = computed(() => props.editor.isEditable);
const label = computed(() => String(props.node.attrs.label ?? ""));
const labelSizer = computed(() => label.value || "标签");
const inputRef = ref<HTMLTextAreaElement | null>(null);

watch(
  () => props.selected,
  async (selected) => {
    if (!selected || !editable.value) {
      return;
    }
    await nextTick();
    if (document.activeElement === inputRef.value) {
      return;
    }
    inputRef.value?.focus();
  },
  { immediate: true },
);

function onFieldMouseDown(event: MouseEvent) {
  event.stopPropagation();
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
  // atom 之后的位置 = 外侧第一个字符「前面」
  const after = pos + props.node.nodeSize;
  const { state, view } = props.editor;
  const selection = TextSelection.create(state.doc, after);
  view.dispatch(state.tr.setSelection(selection).scrollIntoView());
  view.focus();
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
  view.dispatch(tr);
  view.focus();
}

function onKeydown(event: KeyboardEvent) {
  const input = event.target as HTMLTextAreaElement;
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
  <NodeViewWrapper as="span" class="rd-label" contenteditable="false">
    <span class="rd-label__cell">
      <span class="rd-label__sizer" aria-hidden="true">{{ labelSizer }}</span>
      <textarea
        ref="inputRef"
        class="rd-label__input"
        rows="1"
        cols="1"
        wrap="soft"
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
  margin: 0 3px;
  padding: 0;
  border: 1px solid var(--app-border);
  border-radius: 4px;
  overflow: hidden;
  background: var(--app-surface-muted);
  vertical-align: middle;
  box-sizing: border-box;
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
  user-select: text;

  &:focus {
    color: var(--app-fg-secondary);
    background: var(--app-hover-bg);
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
