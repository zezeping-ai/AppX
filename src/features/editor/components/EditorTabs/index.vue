<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { encryptionFileIcon } from "@/modules/editor/encryption";
import type { OpenedEditorFile } from "@/modules/editor/types";

defineProps<{
  tabs: OpenedEditorFile[];
  activePath?: string | null;
}>();

const emit = defineEmits<{
  select: [path: string];
  close: [path: string];
}>();
</script>

<template>
  <div v-if="tabs.length" class="editor-tabs">
    <button
      v-for="tab in tabs"
      :key="tab.path"
      type="button"
      class="editor-tabs__item"
      :class="{ 'editor-tabs__item--active': tab.path === activePath }"
      @click="emit('select', tab.path)"
    >
      <Icon :icon="encryptionFileIcon(tab)" width="14" height="14" />
      <span class="editor-tabs__label">
        {{ tab.name }}<span v-if="tab.dirty" class="editor-tabs__dirty">*</span>
      </span>
      <span
        class="editor-tabs__close"
        title="关闭"
        @click.stop="emit('close', tab.path)"
      >
        <Icon icon="mdi:close" width="14" height="14" />
      </span>
    </button>
  </div>
</template>

<style scoped lang="scss">
.editor-tabs {
  display: flex;
  align-items: stretch;
  gap: 2px;
  padding: 0 8px;
  min-height: 36px;
  flex-shrink: 0;
  overflow-x: auto;
  border-bottom: 1px solid #e5e7eb;
  background: #fff;
}

.editor-tabs__item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 220px;
  padding: 0 10px;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: #4b5563;
  font-size: 12px;
  cursor: pointer;
  flex-shrink: 0;

  &:hover {
    background: #f3f4f6;
  }

  &--active {
    color: #1d4ed8;
    border-bottom-color: #1d4ed8;
    background: #fff;
  }
}

.editor-tabs__label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.editor-tabs__dirty {
  color: #ef4444;
}

.editor-tabs__close {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  color: #9ca3af;

  &:hover {
    background: #e5e7eb;
    color: #111827;
  }
}
</style>
