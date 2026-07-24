<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { nextTick, onMounted, ref } from "vue";

const props = defineProps<{
  depth: number;
  value: string;
  kind: "file" | "folder";
}>();

const emit = defineEmits<{
  confirm: [value: string];
  cancel: [];
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const draft = ref(props.value);

onMounted(async () => {
  await nextTick();
  inputRef.value?.focus();
  inputRef.value?.select();
});

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Enter") {
    event.preventDefault();
    emit("confirm", draft.value);
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    emit("cancel");
  }
}

function onBlur() {
  emit("confirm", draft.value);
}
</script>

<template>
  <div class="explorer-inline" :style="{ paddingLeft: `${depth * 12 + 8}px` }">
    <span class="explorer-inline__chevron" />
    <Icon
      :icon="kind === 'folder' ? 'mdi:folder-outline' : 'mdi:file-document-outline'"
      width="15"
      height="15"
      class="explorer-inline__icon"
    />
    <input
      ref="inputRef"
      v-model="draft"
      class="explorer-inline__input"
      type="text"
      spellcheck="false"
      @keydown="onKeydown"
      @blur="onBlur"
      @click.stop
    />
  </div>
</template>

<style scoped lang="scss">
.explorer-inline {
  display: flex;
  align-items: center;
  gap: 4px;
  min-height: 22px;
  padding-right: 8px;
}

.explorer-inline__chevron {
  width: 16px;
  flex-shrink: 0;
}

.explorer-inline__icon {
  flex-shrink: 0;
  color: var(--app-fg-muted);
}

.explorer-inline__input {
  flex: 1;
  min-width: 0;
  height: 20px;
  padding: 0 4px;
  border: 1px solid var(--app-active-fg);
  border-radius: 2px;
  outline: none;
  font-size: 13px;
  color: var(--app-fg);
  background: var(--app-surface);
}
</style>
