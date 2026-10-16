<script setup lang="ts">
import { Icon } from "@/components/Icon";
import { NodeViewWrapper, nodeViewProps } from "@tiptap/vue-3";
import { message } from "ant-design-vue";
import { useClipboard } from "@vueuse/core";
import { computed, ref } from "vue";
import { useAtomNodeView } from "../atom";

const props = defineProps(nodeViewProps);

const labelRef = ref<HTMLTextAreaElement | null>(null);
const valueRef = ref<HTMLTextAreaElement | null>(null);
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
  kind: "labelValue",
  enterEditTarget: valueRef,
});

const label = computed(() => String(props.node.attrs.label ?? ""));
const value = computed(() => String(props.node.attrs.value ?? ""));
const secret = computed(() => Boolean(props.node.attrs.secret));
const revealed = ref(false);
const masked = computed(() => secret.value && !revealed.value);
const displayValue = computed(() =>
  masked.value ? "•".repeat(value.value.length || 3) : value.value,
);
const labelSizer = computed(() => label.value || "标签");
const valueSizer = computed(() => (masked.value ? displayValue.value : value.value) || "值");
const isEmpty = computed(() => !label.value && !value.value);
const secretTitle = computed(() => {
  if (!secret.value) return "标记为密码";
  if (!editable.value) return revealed.value ? "隐藏值" : "显示值";
  return "取消密码标记";
});
const secretIcon = computed(() => {
  if (!secret.value) return "mdi:eye-off-outline";
  if (!editable.value && revealed.value) return "mdi:eye";
  return "mdi:eye-off";
});

const { copy, copied } = useClipboard({ copiedDuring: 1200, legacy: true });

function onLabelInput(event: Event) {
  updateAttr("label", event);
}

function onValueInput(event: Event) {
  updateAttr("value", event);
}

function focusFieldEnd(el: HTMLTextAreaElement | null) {
  el?.focus();
  const len = el?.value.length ?? 0;
  el?.setSelectionRange(len, len);
}

function onLabelKeydown(event: KeyboardEvent) {
  const input = event.target as HTMLTextAreaElement;
  if (handleClipboardKeys(event, input)) {
    return;
  }

  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    event.stopPropagation();
    valueRef.value?.focus();
    return;
  }
  if (event.key === "Tab") {
    event.preventDefault();
    event.stopPropagation();
    valueRef.value?.focus();
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
    valueRef.value?.focus();
    valueRef.value?.setSelectionRange(0, 0);
  }
}

function onValueKeydown(event: KeyboardEvent) {
  const input = event.target as HTMLTextAreaElement;
  if (handleClipboardKeys(event, input)) {
    return;
  }

  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    event.stopPropagation();
    insertSiblingAfter((editor) =>
      editor.state.schema.nodes.labelValue.create({ label: "", value: "" }),
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
    if (label.value.length > 0) {
      focusFieldEnd(labelRef.value);
      return;
    }
    backspaceAtStart(isEmpty.value);
    return;
  }
  if (event.key === "ArrowRight" && isCaretAtEnd(input)) {
    event.preventDefault();
    event.stopPropagation();
    focusEditorAfterNode();
    return;
  }
  if (event.key === "ArrowLeft" && isCaretAtStart(input)) {
    event.preventDefault();
    event.stopPropagation();
    focusFieldEnd(labelRef.value);
  }
}

function toggleSecret() {
  const next = !secret.value;
  props.updateAttributes({ secret: next });
  if (!next) revealed.value = false;
}

/**
 * 可编辑：切换 secret 标记（写入文档，可保存）
 * 只读：临时窥视明文（不改文档）
 */
function onSecretClick() {
  if (!editable.value) {
    if (secret.value) revealed.value = !revealed.value;
    return;
  }
  toggleSecret();
}

async function onCopyValue() {
  if (!value.value) {
    return;
  }
  try {
    await copy(value.value);
  } catch {
    message.error("复制失败");
  }
}
</script>

<template>
  <NodeViewWrapper
    as="span"
    class="lv"
    data-rich-atom="labelValue"
    contenteditable="false"
    @mousedown="onShellMouseDown"
  >
    <span class="lv__cell lv__cell--label">
      <span class="lv__sizer" aria-hidden="true">{{ labelSizer }}</span>
      <textarea
        ref="labelRef"
        class="lv__input"
        rows="1"
        cols="1"
        wrap="soft"
        tabindex="-1"
        :value="label"
        :readonly="!editable"
        placeholder="标签"
        @mousedown="onFieldMouseDown"
        @input="onLabelInput"
        @keydown="onLabelKeydown"
      />
    </span>
    <span class="lv__cell lv__cell--value">
      <span class="lv__sizer" aria-hidden="true">{{ valueSizer }}</span>
      <textarea
        ref="valueRef"
        :class="['lv__input', { 'lv__input--masked': masked }]"
        rows="1"
        cols="1"
        wrap="soft"
        tabindex="-1"
        :value="masked ? displayValue : value"
        :readonly="!editable || masked"
        placeholder="值"
        @mousedown="onFieldMouseDown"
        @input="onValueInput"
        @keydown="onValueKeydown"
      />
    </span>
    <button
      v-if="editable || secret"
      type="button"
      class="lv__toggle-secret"
      :class="{ 'lv__toggle-secret--active': secret }"
      tabindex="-1"
      :title="secretTitle"
      @click="onSecretClick"
    >
      <Icon
        :icon="secretIcon"
        :size="12"
      />
    </button>
    <button
      type="button"
      class="lv__copy"
      tabindex="-1"
      :title="copied ? '已复制' : '复制'"
      @click="onCopyValue"
    >
      <Icon :icon="copied ? 'mdi:check' : 'mdi:content-copy'" :size="12" />
    </button>
  </NodeViewWrapper>
</template>

<style scoped lang="scss">
.lv {
  display: inline-flex !important;
  align-items: stretch;
  width: max-content;
  max-width: 100%;
  min-height: 22px;
  height: auto;
  margin: 0 1px;
  padding: 0;
  border: 1px solid var(--app-border);
  border-radius: 4px;
  overflow: hidden;
  background: var(--app-surface);
  vertical-align: middle;
  box-sizing: border-box;

  &.ProseMirror-selectednode {
    outline: none;
    background: var(--app-active-bg);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-active-fg) 45%, transparent);

    .lv__cell--label {
      background: color-mix(in srgb, var(--app-active-fg) 12%, var(--app-surface-muted));
    }
  }
}

.lv__cell {
  display: inline-grid;
  flex: 0 1 auto;
  min-width: 0;
  align-self: stretch;
  box-sizing: border-box;
}

.lv__cell--label {
  background: var(--app-surface-muted);
  border-right: 1px solid var(--app-border);
}

.lv__sizer,
.lv__input {
  grid-area: 1 / 1;
  box-sizing: border-box;
  margin: 0;
  padding: 1px 6px;
  border: 0;
  font-family: inherit;
  line-height: 20px;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  word-break: break-word;
  overflow: hidden;
}

.lv__sizer {
  visibility: hidden;
  pointer-events: none;
}

.lv__input {
  width: 100%;
  min-width: 0;
  height: 100%;
  resize: none;
  outline: none;
  appearance: none;
  background: transparent;
  caret-color: var(--app-fg);
  user-select: none;
  color: inherit;
  overflow: hidden;
  overflow-y: hidden;

  &:focus {
    user-select: text;
  }

  &:read-only {
    cursor: default;
  }
}

.lv__cell--label .lv__sizer,
.lv__cell--label .lv__input {
  font-size: 11px;
  font-weight: 500;
  color: var(--app-fg-muted);
}

.lv__cell--label .lv__input:focus {
  color: var(--app-fg-secondary);
  background: var(--app-hover-bg);
}

.lv__cell--value .lv__sizer,
.lv__cell--value .lv__input {
  font-size: 13px;
  color: var(--app-fg);
}

.lv__cell--value .lv__input:focus {
  background: var(--app-hover-bg);
}

.lv__input::placeholder {
  color: var(--app-fg-muted);
  opacity: 0.7;
}

.lv__toggle-secret,
.lv__copy {
  display: inline-flex;
  flex: 0 0 20px;
  align-items: center;
  justify-content: center;
  width: 20px;
  padding: 0;
  border: 0;
  border-left: 1px solid var(--app-border);
  background: transparent;
  color: var(--app-fg-muted);
  cursor: pointer;

  &:hover {
    background: var(--app-hover-bg);
    color: var(--app-fg);
  }
}

.lv__cell--value .lv__input.lv__input--masked {
  user-select: none;
  color: var(--app-fg-muted);
}

.lv__toggle-secret:not(.lv__toggle-secret--active) {
  opacity: 0;
}

.lv:hover .lv__toggle-secret:not(.lv__toggle-secret--active) {
  opacity: 0.5;

  &:hover {
    opacity: 1;
  }
}
</style>
