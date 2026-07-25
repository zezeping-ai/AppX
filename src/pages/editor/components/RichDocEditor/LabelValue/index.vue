<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { NodeViewWrapper, nodeViewProps } from "@tiptap/vue-3";
import { NodeSelection, TextSelection } from "@tiptap/pm/state";
import { message } from "ant-design-vue";
import { computed, nextTick, ref, watch } from "vue";

const props = defineProps(nodeViewProps);

const editable = computed(() => props.editor.isEditable);
const label = computed(() => String(props.node.attrs.label ?? ""));
const value = computed(() => String(props.node.attrs.value ?? ""));
/** sizer 空时用 placeholder 撑宽；过长由 CSS 折行 */
const labelSizer = computed(() => label.value || "标签");
const valueSizer = computed(() => value.value || "值");
const copied = ref(false);
const labelRef = ref<HTMLTextAreaElement | null>(null);
const valueRef = ref<HTMLTextAreaElement | null>(null);

watch(
  () => props.selected,
  async (selected) => {
    if (!selected || !editable.value) {
      return;
    }
    await nextTick();
    const active = document.activeElement;
    if (active === labelRef.value || active === valueRef.value) {
      return;
    }
    valueRef.value?.focus();
  },
  { immediate: true },
);

/** 仅阻止冒泡到 ProseMirror；勿 preventDefault，否则无法单击定位/拖选文字 */
function onFieldMouseDown(event: MouseEvent) {
  event.stopPropagation();
}

/** 统一换行符，保留内部换行 */
function sanitize(raw: string) {
  return raw.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
}

function onLabelInput(event: Event) {
  const el = event.target as HTMLTextAreaElement;
  const next = sanitize(el.value);
  if (el.value !== next) {
    el.value = next;
  }
  props.updateAttributes({ label: next });
}

function onValueInput(event: Event) {
  const el = event.target as HTMLTextAreaElement;
  const next = sanitize(el.value);
  if (el.value !== next) {
    el.value = next;
  }
  props.updateAttributes({ value: next });
}

function focusEditorAfterNode() {
  const pos = props.getPos();
  if (typeof pos !== "number") {
    return;
  }
  // atom 之后的位置 = 外侧第一个字符「前面」
  const after = pos + props.node.nodeSize;
  const { state, view } = props.editor;
  view.dispatch(state.tr.setSelection(TextSelection.create(state.doc, after)).scrollIntoView());
  view.focus();
}

function insertSiblingAfter() {
  const pos = props.getPos();
  if (typeof pos !== "number") {
    return;
  }
  const after = pos + props.node.nodeSize;
  const next = props.editor.state.schema.nodes.labelValue.create({
    label: "",
    value: "",
  });
  const { state, view } = props.editor;
  let tr = state.tr.insert(after, next);
  tr = tr.setSelection(NodeSelection.create(tr.doc, after));
  view.dispatch(tr);
  view.focus();
}

function onLabelKeydown(event: KeyboardEvent) {
  // Enter：跳到 value；Shift+Enter：内部换行
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
  if (event.key === "ArrowRight") {
    const input = event.target as HTMLTextAreaElement;
    if (input.selectionStart === input.value.length && input.selectionEnd === input.value.length) {
      event.preventDefault();
      event.stopPropagation();
      valueRef.value?.focus();
      valueRef.value?.setSelectionRange(0, 0);
    }
  }
}

function onValueKeydown(event: KeyboardEvent) {
  const input = event.target as HTMLTextAreaElement;
  // Enter：插入下一个；Shift+Enter：内部换行
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
    return;
  }
  if (event.key === "ArrowLeft") {
    if (input.selectionStart === 0 && input.selectionEnd === 0) {
      event.preventDefault();
      event.stopPropagation();
      labelRef.value?.focus();
      const len = labelRef.value?.value.length ?? 0;
      labelRef.value?.setSelectionRange(len, len);
    }
  }
}

async function onCopyValue() {
  if (!value.value) {
    return;
  }
  try {
    await navigator.clipboard.writeText(value.value);
    copied.value = true;
    window.setTimeout(() => {
      copied.value = false;
    }, 1200);
  } catch {
    message.error("复制失败");
  }
}
</script>

<template>
  <NodeViewWrapper as="span" class="lv" contenteditable="false">
    <span class="lv__cell lv__cell--label">
      <span class="lv__sizer" aria-hidden="true">{{ labelSizer }}</span>
      <textarea
        ref="labelRef"
        class="lv__input"
        rows="1"
        cols="1"
        wrap="soft"
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
        class="lv__input"
        rows="1"
        cols="1"
        wrap="soft"
        :value="value"
        :readonly="!editable"
        placeholder="值"
        @mousedown="onFieldMouseDown"
        @input="onValueInput"
        @keydown="onValueKeydown"
      />
    </span>
    <button
      type="button"
      class="lv__copy"
      tabindex="-1"
      :title="copied ? '已复制' : '复制'"
      @click="onCopyValue"
    >
      <Icon :icon="copied ? 'mdi:check' : 'mdi:content-copy'" width="12" height="12" />
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
  margin: 0 4px;
  padding: 0;
  border: 1px solid var(--app-border);
  border-radius: 4px;
  overflow: hidden;
  background: var(--app-surface);
  vertical-align: middle;
  box-sizing: border-box;
}

/* 同格 sizer：label / value 同样自适应宽、顶满后折行增高 */
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
  /* width/min-width 归零，格子宽度只由 sizer 决定，避免 textarea 默认 cols=20 撑满 */
  width: 100%;
  min-width: 0;
  height: 100%;
  resize: none;
  outline: none;
  appearance: none;
  background: transparent;
  caret-color: var(--app-fg);
  user-select: text;
  color: inherit;
  overflow: hidden;
  overflow-y: hidden;

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
</style>
