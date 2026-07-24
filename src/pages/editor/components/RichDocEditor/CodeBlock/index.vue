<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { NodeViewContent, NodeViewWrapper, nodeViewProps } from "@tiptap/vue-3";
import { message } from "ant-design-vue";
import { computed, ref } from "vue";
import {
  canFormatCodeLanguage,
  CODE_LANGUAGE_OPTIONS,
  formatCodeByLanguage,
} from "./helpers";

const props = defineProps(nodeViewProps);

const formatting = ref(false);

const language = computed(() => (props.node.attrs.language as string | null) ?? "");
const canFormat = computed(() => canFormatCodeLanguage(language.value));
const editable = computed(() => props.editor.isEditable);

function onLanguageChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  props.updateAttributes({ language: value || null });
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
      <select
        class="code-block__lang"
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
      <button
        type="button"
        class="code-block__format"
        title="格式化代码"
        :disabled="!editable || formatting || !canFormat"
        @click="onFormat"
      >
        <Icon icon="mdi:code-braces" width="14" height="14" />
        格式化
      </button>
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
  padding: 4px 8px;
  border-bottom: 1px solid var(--app-border);
  background: var(--app-surface-muted);
}

.code-block__lang {
  height: 24px;
  max-width: 140px;
  padding: 0 6px;
  border: 1px solid var(--app-border);
  border-radius: 4px;
  background: var(--app-surface);
  color: var(--app-fg);
  font-size: 12px;
  outline: none;
  cursor: pointer;

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  &:focus {
    border-color: var(--app-active-fg);
  }
}

.code-block__format {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 24px;
  padding: 0 8px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--app-fg-secondary);
  font-size: 12px;
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
