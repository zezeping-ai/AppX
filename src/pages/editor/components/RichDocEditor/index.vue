<script setup lang="ts">
import Placeholder from "@tiptap/extension-placeholder";
import { TableKit } from "@tiptap/extension-table";
import StarterKit from "@tiptap/starter-kit";
import { EditorContent, useEditor } from "@tiptap/vue-3";
import { tryOnScopeDispose } from "@vueuse/core";
import { computed, ref, watch, watchEffect } from "vue";
import {
  axdocDocsEqual,
  parseAxdocContent,
  serializeAxdoc,
} from "@/modules/editor/axdoc";
import { RichDocCodeBlock } from "./CodeBlock/extension";
import RichDocToolbar from "./Toolbar.vue";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    readOnly?: boolean;
  }>(),
  {
    readOnly: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const initialParse = parseAxdocContent(props.modelValue);
const parseError = ref(initialParse.ok ? null : initialParse.error);
const broken = computed(() => Boolean(parseError.value));
const canEdit = computed(() => !props.readOnly && !broken.value);

const editor = useEditor({
  content: initialParse.doc,
  editable: canEdit.value,
  extensions: [
    StarterKit.configure({
      codeBlock: false,
    }),
    RichDocCodeBlock,
    Placeholder.configure({
      placeholder: "开始写作…",
    }),
    TableKit.configure({
      table: { resizable: true },
    }),
  ],
  onUpdate: ({ editor: current }) => {
    if (broken.value) {
      return;
    }
    const doc = current.getJSON() as Record<string, unknown>;
    const baseline = parseAxdocContent(props.modelValue).doc;
    if (axdocDocsEqual(doc, baseline)) {
      return;
    }
    emit("update:modelValue", serializeAxdoc(doc));
  },
});

watch(
  () => props.modelValue,
  (value) => {
    const current = editor.value;
    if (!current) {
      return;
    }

    const parsed = parseAxdocContent(value);
    if (!parsed.ok) {
      // 外部写入了损坏内容：锁定编辑，保留磁盘原串，不 setContent 覆盖
      parseError.value = parsed.error;
      current.setEditable(false);
      return;
    }

    parseError.value = null;
    const prev = current.getJSON() as Record<string, unknown>;
    if (axdocDocsEqual(parsed.doc, prev)) {
      return;
    }
    current.commands.setContent(parsed.doc, { emitUpdate: false });
  },
);

watchEffect(() => {
  editor.value?.setEditable(!props.readOnly && !broken.value);
});

tryOnScopeDispose(() => {
  editor.value?.destroy();
});
</script>

<template>
  <div class="rich-doc-editor" :class="{ 'rich-doc-editor--broken': broken }">
    <div v-if="broken" class="rich-doc-editor__error">
      {{ parseError }}（已禁止编辑，避免覆盖原文件；请修复内容或从备份恢复）
    </div>
    <RichDocToolbar v-if="editor && canEdit" :editor="editor" />
    <EditorContent :editor="editor" class="rich-doc-editor__content" />
  </div>
</template>

<style scoped lang="scss">
.rich-doc-editor {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 240px;
  border: 1px solid var(--app-border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--app-surface);
}

.rich-doc-editor--broken {
  border-color: var(--app-danger-border);
}

.rich-doc-editor__error {
  padding: 8px 12px;
  border-bottom: 1px solid var(--app-danger-border);
  background: var(--app-danger-bg);
  color: var(--app-danger-fg);
  font-size: 12px;
  line-height: 1.5;
}

.rich-doc-editor__content {
  flex: 1;
  min-height: 0;
  overflow: auto;

  :deep(.tiptap) {
    min-height: 100%;
    padding: 16px 20px;
    outline: none;
    color: var(--app-fg);
    font-size: 14px;
    line-height: 1.7;

    > * + * {
      margin-top: 0.6em;
    }

    h1,
    h2,
    h3 {
      line-height: 1.3;
      font-weight: 700;
    }

    h1 {
      font-size: 1.75em;
    }

    h2 {
      font-size: 1.4em;
    }

    h3 {
      font-size: 1.15em;
    }

    ul,
    ol {
      padding-left: 1.4em;
    }

    blockquote {
      margin: 0;
      padding-left: 12px;
      border-left: 3px solid var(--app-divider);
      color: var(--app-fg-secondary);
    }

    /* 代码块外壳由 CodeBlockView 负责；此处只保留语法高亮 token */
    .code-block pre code .hljs-comment,
    .code-block pre code .hljs-quote {
      color: var(--app-fg-muted);
      font-style: italic;
    }

    .code-block pre code .hljs-keyword,
    .code-block pre code .hljs-selector-tag {
      color: #7c3aed;
    }

    .code-block pre code .hljs-string,
    .code-block pre code .hljs-doctag,
    .code-block pre code .hljs-template-variable {
      color: #059669;
    }

    .code-block pre code .hljs-number,
    .code-block pre code .hljs-literal {
      color: #d97706;
    }

    .code-block pre code .hljs-title,
    .code-block pre code .hljs-section,
    .code-block pre code .hljs-selector-id {
      color: #2563eb;
    }

    .code-block pre code .hljs-attr,
    .code-block pre code .hljs-attribute,
    .code-block pre code .hljs-variable,
    .code-block pre code .hljs-template-tag {
      color: #0891b2;
    }

    .code-block pre code .hljs-built_in,
    .code-block pre code .hljs-type,
    .code-block pre code .hljs-params {
      color: #db2777;
    }

    .code-block pre code .hljs-meta,
    .code-block pre code .hljs-regexp {
      color: #ea580c;
    }

    [data-theme="dark"] & {
      .code-block pre code .hljs-keyword,
      .code-block pre code .hljs-selector-tag {
        color: #c4b5fd;
      }

      .code-block pre code .hljs-string,
      .code-block pre code .hljs-doctag {
        color: #6ee7b7;
      }

      .code-block pre code .hljs-number {
        color: #fbbf24;
      }

      .code-block pre code .hljs-title,
      .code-block pre code .hljs-section {
        color: #93c5fd;
      }

      .code-block pre code .hljs-attr,
      .code-block pre code .hljs-attribute,
      .code-block pre code .hljs-variable {
        color: #67e8f9;
      }

      .code-block pre code .hljs-built_in,
      .code-block pre code .hljs-type {
        color: #f9a8d4;
      }

      .code-block pre code .hljs-meta,
      .code-block pre code .hljs-regexp {
        color: #fdba74;
      }
    }

    /* 行内 code（非代码块） */
    :not(pre) > code {
      font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
      padding: 0.1em 0.35em;
      border-radius: 4px;
      background: var(--app-surface-muted);
    }

    p.is-editor-empty:first-child::before {
      content: attr(data-placeholder);
      float: left;
      height: 0;
      color: var(--app-fg-muted);
      pointer-events: none;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      table-layout: fixed;
      margin: 0;
      overflow: hidden;

      td,
      th {
        position: relative;
        min-width: 48px;
        padding: 6px 8px;
        border: 1px solid var(--app-border);
        vertical-align: top;
        word-break: break-word;
      }

      th {
        background: var(--app-surface-muted);
        font-weight: 600;
        text-align: left;
        vertical-align: middle;
      }

      /* 单元格内段落默认无多余外边距，避免表头看起来贴顶 */
      td > p,
      th > p {
        margin: 0;
      }

      .selectedCell::after {
        content: "";
        position: absolute;
        inset: 0;
        background: var(--app-active-bg);
        pointer-events: none;
        z-index: 2;
      }

      .column-resize-handle {
        position: absolute;
        top: 0;
        right: -1px;
        bottom: -2px;
        width: 2px;
        background: var(--app-divider);
        pointer-events: none;
      }
    }

    .tableWrapper {
      margin: 0.75em 0;
      overflow-x: auto;
    }

    &.resize-cursor {
      cursor: ew-resize;
      cursor: col-resize;
    }

    a {
      color: var(--app-active-fg);
    }
  }
}
</style>
