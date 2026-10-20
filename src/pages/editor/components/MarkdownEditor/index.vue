<script setup lang="ts">
import { toRef } from "vue";
import { useMarkdownEditor } from "./useMarkdownEditor";
import "@milkdown/crepe/theme/common/style.css";
import "@milkdown/crepe/theme/frame.css";

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

const { MarkdownEditorHost } = useMarkdownEditor({
  modelValue: toRef(props, "modelValue"),
  readOnly: toRef(props, "readOnly"),
  onUpdate: (value) => emit("update:modelValue", value),
});
</script>

<template>
  <div class="markdown-editor">
    <MarkdownEditorHost />
  </div>
</template>

<style scoped lang="scss">
.markdown-editor {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-height: 240px;
  border: 1px solid var(--app-border);
  border-radius: 8px;
  overflow: auto;
  background: var(--app-surface);
}
</style>

<style lang="scss">
@use "./styles/content.scss";
</style>
