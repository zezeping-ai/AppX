<script setup lang="ts">
import { EditorContent } from "@tiptap/vue-3";
import { toRef } from "vue";
import RichDocToolbar from "./Toolbar/index.vue";
import { useRichDocEditor } from "./useRichDocEditor";

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

const { editor, parseError, broken, canEdit } = useRichDocEditor({
  modelValue: toRef(props, "modelValue"),
  readOnly: toRef(props, "readOnly"),
  onUpdate: (value) => emit("update:modelValue", value),
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
</style>

<style lang="scss">
@use "./styles/content.scss";
</style>
