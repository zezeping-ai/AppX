<script setup lang="ts">
import { ref, toRef } from "vue";
import { useMonacoEditor } from "@/components/MonacoEditor/useMonacoEditor";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    language?: string;
    readOnly?: boolean;
    /** 按内容自动增高，交给外层滚动 */
    autoHeight?: boolean;
    minHeight?: number;
    maxHeight?: number;
  }>(),
  {
    language: "plaintext",
    readOnly: false,
    autoHeight: false,
    minHeight: 120,
    maxHeight: Number.POSITIVE_INFINITY,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();

const hostRef = ref<HTMLElement | null>(null);
const languageRef = toRef(props, "language");
const readOnlyRef = toRef(props, "readOnly");

useMonacoEditor(hostRef, {
  language: () => languageRef.value,
  readOnly: () => readOnlyRef.value,
  value: () => props.modelValue,
  onChange: (value) => emit("update:modelValue", value),
  autoHeight: () => props.autoHeight,
  minHeight: () => props.minHeight,
  maxHeight: () => props.maxHeight,
});
</script>

<template>
  <div
    ref="hostRef"
    class="monaco-editor-host"
    :class="{ 'monaco-editor-host--auto-height': autoHeight }"
  />
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

.monaco-editor-host--auto-height {
  height: auto;
  min-height: unset;
}
</style>
