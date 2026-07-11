<script setup lang="ts">
import { Icon } from "@iconify/vue";
import type { PaletteSourceTab } from "@/pages/clipboard-palette/useClipboardPalette";

defineProps<{
  sources: PaletteSourceTab[];
  activeBundle?: string;
  orientation: "horizontal" | "vertical";
}>();

const emit = defineEmits<{
  select: [string | undefined];
}>();
</script>

<template>
  <nav
    class="palette-source-tabs"
    :class="`palette-source-tabs--${orientation}`"
    aria-label="来源应用"
  >
    <button
      type="button"
      class="palette-source-tabs__item"
      :class="{ 'palette-source-tabs__item--active': !activeBundle }"
      title="全部"
      @click="emit('select', undefined)"
    >
      <Icon icon="mdi:view-grid-outline" aria-hidden="true" />
    </button>
    <button
      v-for="source in sources"
      :key="source.bundle"
      type="button"
      class="palette-source-tabs__item"
      :class="{ 'palette-source-tabs__item--active': activeBundle === source.bundle }"
      :title="source.name"
      @click="emit('select', source.bundle)"
    >
      <img
        v-if="source.iconUrl"
        :src="source.iconUrl"
        class="palette-source-tabs__icon"
        alt=""
      />
      <Icon v-else icon="mdi:application-outline" aria-hidden="true" />
    </button>
  </nav>
</template>

<style scoped lang="scss">
.palette-source-tabs {
  display: flex;
  flex-shrink: 0;
  min-height: 0;
  gap: 2px;

  &--horizontal {
    flex-direction: row;
    align-items: center;
    overflow-x: auto;
    padding: 4px 6px;
    border-bottom: 1px solid var(--app-border);

    &::-webkit-scrollbar {
      height: 3px;
    }

    &::-webkit-scrollbar-thumb {
      background: color-mix(in srgb, var(--app-fg) 18%, transparent);
      border-radius: 2px;
    }
  }

  &--vertical {
    flex-direction: column;
    align-items: center;
    width: 36px;
    padding: 6px 0;
    border-right: 1px solid var(--app-border);
    overflow-y: auto;
  }
}

.palette-source-tabs__item {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  padding: 0;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: inherit;
  font-size: 18px;
  opacity: 0.72;
  cursor: pointer;
  transition:
    background-color 0.15s ease,
    opacity 0.15s ease;

  &:hover {
    opacity: 1;
    background: var(--app-hover-bg);
  }

  &--active {
    opacity: 1;
    background: var(--app-active-bg);
    color: var(--app-active-fg);
  }
}

.palette-source-tabs__icon {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  object-fit: contain;
}
</style>
