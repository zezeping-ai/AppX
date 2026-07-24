<script setup lang="ts">
import { ref, toRef } from "vue";
import { useMonacoEditor } from "@/components/MonacoEditor/useMonacoEditor";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    language?: string;
    readOnly?: boolean;
  }>(),
  {
    language: "plaintext",
    readOnly: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const containerRef = ref<HTMLElement | null>(null);
const languageRef = toRef(props, "language");
const readOnlyRef = toRef(props, "readOnly");

useMonacoEditor(containerRef, {
  language: () => languageRef.value,
  readOnly: () => readOnlyRef.value,
  value: () => props.modelValue,
  onChange: (value) => emit("update:modelValue", value),
});
</script>

<template>
  <div ref="containerRef" class="monaco-editor-host" />
</template>

<style scoped lang="scss">
.monaco-editor-host {
  width: 100%;
  height: 100%;
  min-height: 240px;
  border: 1px solid var(--app-border);
  border-radius: 8px;
  overflow: hidden;
  background: var(--app-surface);
}
</style>
