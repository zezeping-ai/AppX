<script setup lang="ts">
import { NodeViewWrapper, nodeViewProps } from "@tiptap/vue-3";
import { computed, ref } from "vue";
import { useAtomNodeView } from "../atom";

const props = defineProps(nodeViewProps);

const inputRef = ref<HTMLTextAreaElement | null>(null);
const {
  editable,
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
} = useAtomNodeView(props, {
  kind: "label",
  enterEditTarget: inputRef,
});

const label = computed(() => String(props.node.attrs.label ?? ""));
const labelSizer = computed(() => label.value || "标签");
const isEmpty = computed(() => !label.value);

function onInput(event: Event) {
  updateAttr("label", event);
}

function onKeydown(event: KeyboardEvent) {
  const input = event.target as HTMLTextAreaElement;
  if (handleClipboardKeys(event, input)) {
    return;
  }

  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    event.stopPropagation();
    insertSiblingAfter((editor) =>
      editor.state.schema.nodes.label.create({ label: "" }),
    );
    return;
  }
  if (event.key === "Tab" && !event.shiftKey) {
    event.preventDefault();
    event.stopPropagation();
    focusEditorAfterNode();
    return;
  }
  if (event.key === "Backspace" && isCaretAtStart(input)) {
    event.preventDefault();
    event.stopPropagation();
    backspaceAtStart(isEmpty.value);
    return;
  }
  if (event.key === "ArrowLeft" && isCaretAtStart(input)) {
    event.preventDefault();
    event.stopPropagation();
    focusEditorBeforeNode();
    return;
  }
  if (event.key === "ArrowRight" && isCaretAtEnd(input)) {
    event.preventDefault();
    event.stopPropagation();
    focusEditorAfterNode();
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
