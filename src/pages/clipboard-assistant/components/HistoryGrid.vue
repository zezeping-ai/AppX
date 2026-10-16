<script setup lang="ts">
import type { ClipboardItemSummary } from "@/modules/clipboardAssistant";
import HistoryItemCard from "@/pages/clipboard-assistant/components/HistoryItemCard.vue";

defineProps<{
  items: ClipboardItemSummary[];
  showSourceAppIcon: boolean;
}>();

const emit = defineEmits<{
  detail: [ClipboardItemSummary];
  pin: [ClipboardItemSummary];
  delete: [ClipboardItemSummary];
}>();
</script>

<template>
  <div class="history-grid">
    <HistoryItemCard
      v-for="item in items"
      :key="item.id"
      :item="item"
      :show-source-app-icon="showSourceAppIcon"
      @detail="emit('detail', item)"
      @pin="emit('pin', item)"
      @delete="emit('delete', item)"
    />
  </div>
</template>

<style scoped lang="scss">
.history-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
  align-content: start;
}
</style>
