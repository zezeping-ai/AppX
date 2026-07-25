<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { NodeViewWrapper, nodeViewProps } from "@tiptap/vue-3";
import { NodeSelection } from "@tiptap/pm/state";
import { message } from "ant-design-vue";
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
const value = computed(() => String(props.node.attrs.value ?? ""));
/** sizer 空时用 placeholder 撑宽；过长由 CSS 折行 */
const labelSizer = computed(() => label.value || "标签");
const valueSizer = computed(() => value.value || "值");
const copied = ref(false);
const labelRef = ref<HTMLTextAreaElement | null>(null);
const valueRef = ref<HTMLTextAreaElement | null>(null);

const isEmpty = computed(() => !label.value && !value.value);

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
    valueRef.value?.focus();
  },
  { immediate: true },
);

/** 字段：单击编辑；拖动 / Shift 连选多个 atom */
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

/** textarea 内：折叠光标时复制/剪切整颗；Escape 整选 */
function handleAtomClipboardKeys(
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
    const pos = props.getPos();
    if (typeof pos === "number") {
      event.preventDefault();
      event.stopPropagation();
      selectAtomNode(props.editor, pos);
    }
    return true;
  }
  return false;
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
  focusAfterAtom(props.editor, pos, props.node.nodeSize);
}

function focusEditorBeforeNode() {
  const pos = props.getPos();
  if (typeof pos !== "number") {
    return;
  }
  focusBeforeAtom(props.editor, pos);
}

function handleBackspaceAtStart(event: KeyboardEvent) {
  const pos = props.getPos();
  if (typeof pos !== "number") {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  backspaceAtAtomStart(props.editor, pos, props.node, isEmpty.value);
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
  requestAtomEnterEdit(after);
  view.dispatch(tr);
  view.focus();
}

function onLabelKeydown(event: KeyboardEvent) {
  const input = event.target as HTMLTextAreaElement;
  if (handleAtomClipboardKeys(event, input)) {
    return;
  }
  const atStart = input.selectionStart === 0 && input.selectionEnd === 0;

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
  // 行首 Backspace：合并上一段 / 删前方内容（否则焦点锁在 textarea 里合并不了）
  if (event.key === "Backspace" && atStart) {
    handleBackspaceAtStart(event);
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
      event.preventDefault();
      event.stopPropagation();
      valueRef.value?.focus();
      valueRef.value?.setSelectionRange(0, 0);
    }
  }
}

function onValueKeydown(event: KeyboardEvent) {
  const input = event.target as HTMLTextAreaElement;
  if (handleAtomClipboardKeys(event, input)) {
    return;
  }
  const atStart = input.selectionStart === 0 && input.selectionEnd === 0;

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
  if (event.key === "Backspace" && atStart) {
    event.preventDefault();
    event.stopPropagation();
    // 先回到 label；label 也空则走合并/删前逻辑
    if (label.value.length > 0) {
      labelRef.value?.focus();
      const len = labelRef.value?.value.length ?? 0;
      labelRef.value?.setSelectionRange(len, len);
      return;
    }
    handleBackspaceAtStart(event);
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
    if (atStart) {
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
        class="lv__input"
        rows="1"
        cols="1"
        wrap="soft"
        tabindex="-1"
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
  margin: 0 1px;
  padding: 0;
  border: 1px solid var(--app-border);
  border-radius: 4px;
  overflow: hidden;
  background: var(--app-surface);
  vertical-align: middle;
  box-sizing: border-box;

  &.ProseMirror-selectednode {
    /* 与正文划选一致的浅蓝底，不要粗蓝描边 */
    outline: none;
    background: var(--app-active-bg);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-active-fg) 45%, transparent);

    .lv__cell--label {
      background: color-mix(in srgb, var(--app-active-fg) 12%, var(--app-surface-muted));
    }
  }
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
  /* 未聚焦时禁止原生划选，避免拖选多个 atom 时被字段抢走 */
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
