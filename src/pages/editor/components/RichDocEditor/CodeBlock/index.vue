<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { NodeViewContent, NodeViewWrapper, nodeViewProps } from "@tiptap/vue-3";
import { message } from "ant-design-vue";
import { useClipboard } from "@vueuse/core";
import { computed, ref } from "vue";
import {
  canFormatCodeLanguage,
  CODE_LANGUAGE_OPTIONS,
  formatCodeByLanguage,
} from "./helpers";

const props = defineProps(nodeViewProps);

const formatting = ref(false);
const { copy, copied } = useClipboard({ copiedDuring: 1200, legacy: true });

const language = computed(() => (props.node.attrs.language as string | null) ?? "");
const canFormat = computed(() => canFormatCodeLanguage(language.value));
const editable = computed(() => props.editor.isEditable);
const hasCode = computed(() => Boolean(props.node.textContent));

function onLanguageChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  props.updateAttributes({ language: value || null });
}

async function onCopy() {
  const source = props.node.textContent;
  if (!source) {
    return;
  }
  try {
    await copy(source);
  } catch {
    message.error("复制失败");
  }
}

async function onFormat() {
  if (!canFormat.value || !editable.value) {
    return;
  }

  formatting.value = true;
  try {
    const source = props.node.textContent;
    const result = await formatCodeByLanguage(source, language.value);
    if (!result.ok) {
      message.error(result.error);
      return;
    }
    if (result.code === source) {
      return;
    }

    const pos = props.getPos();
    if (typeof pos !== "number") {
      return;
    }

    const { state } = props.editor;
    props.editor
      .chain()
      .focus()
      .command(({ tr, dispatch }) => {
        if (!dispatch) {
          return true;
        }
        const content = result.code ? [state.schema.text(result.code)] : [];
        const next = state.schema.nodes.codeBlock.create(props.node.attrs, content);
        tr.replaceWith(pos, pos + props.node.nodeSize, next);
        return true;
      })
      .run();
  } finally {
    formatting.value = false;
  }
}
</script>

<template>
  <NodeViewWrapper class="code-block" as="div">
    <div class="code-block__bar" contenteditable="false">
      <label class="code-block__lang">
        <select
          class="code-block__lang-select"
          :value="language"
          :disabled="!editable"
          title="代码语言"
          @change="onLanguageChange"
        >
          <option
            v-for="option in CODE_LANGUAGE_OPTIONS"
            :key="option.value || 'plain'"
            :value="option.value"
          >
            {{ option.label }}
          </option>
        </select>
        <Icon
          class="code-block__lang-chevron"
          icon="mdi:chevron-down"
          width="14"
          height="14"
          aria-hidden="true"
        />
      </label>
      <div class="code-block__actions">
        <button
          type="button"
          class="code-block__action"
          title="格式化代码"
          :disabled="!editable || formatting || !canFormat"
          @click="onFormat"
        >
          <Icon icon="mdi:code-braces" width="14" height="14" />
        </button>
        <button
          type="button"
          class="code-block__action"
          :title="copied ? '已复制' : '复制代码'"
          :disabled="!hasCode"
          @click="onCopy"
        >
          <Icon
            :icon="copied ? 'mdi:check' : 'mdi:content-copy'"
            width="14"
            height="14"
          />
        </button>
      </div>
    </div>
    <pre><code><NodeViewContent /></code></pre>
  </NodeViewWrapper>
</template>

<style scoped lang="scss">
.code-block {
  margin: 0.75em 0;
  border: 1px solid var(--app-border);
  border-radius: 6px;
  overflow: hidden;
  background: var(--app-code-bg);
  color: var(--app-code-fg);
}

.code-block__bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-height: 32px;
  padding: 2px 4px 2px 6px;
  border-bottom: 1px solid var(--app-border);
  background: var(--app-surface-muted);
}

.code-block__lang {
  position: relative;
  display: inline-flex;
  align-items: center;
  min-width: 0;
  max-width: 160px;
  color: var(--app-fg-secondary);
}

.code-block__lang-select {
  appearance: none;
  box-sizing: border-box;
  width: 100%;
  height: 26px;
  margin: 0;
  padding: 0 22px 0 8px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: inherit;
  font-size: 12px;
  line-height: 26px;
  outline: none;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: var(--app-hover-bg);
    color: var(--app-fg);
  }

  &:focus-visible {
    background: var(--app-hover-bg);
    color: var(--app-fg);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-active-fg) 40%, transparent);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
}

.code-block__lang-chevron {
  position: absolute;
  right: 4px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
  opacity: 0.7;
}

.code-block__actions {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  flex-shrink: 0;
}

.code-block__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  padding: 0;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--app-fg-secondary);
  cursor: pointer;

  &:hover:not(:disabled) {
    background: var(--app-hover-bg);
    color: var(--app-fg);
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
}

pre {
  margin: 0;
  padding: 10px 12px;
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.55;
  background: transparent;
}

code {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}
</style>
