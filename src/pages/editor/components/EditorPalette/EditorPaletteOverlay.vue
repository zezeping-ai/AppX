<script setup lang="ts">
import { Icon } from "@/components/Icon";

withDefaults(
  defineProps<{
    open: boolean;
    dialogLabel: string;
    /** quick | search：宽度与顶距略不同 */
    variant?: "quick" | "search";
    searchIcon?: string;
  }>(),
  {
    variant: "quick",
    searchIcon: "mdi:file-search-outline",
  },
);

const emit = defineEmits<{
  close: [];
  keydown: [event: KeyboardEvent];
}>();
</script>

<template>
  <Teleport to="body">
    <div
      v-if="open"
      class="editor-palette"
      :class="`editor-palette--${variant}`"
      role="dialog"
      aria-modal="true"
      :aria-label="dialogLabel"
      @keydown="emit('keydown', $event)"
    >
      <div class="editor-palette__backdrop" @click="emit('close')" />
      <div class="editor-palette__panel" :class="`editor-palette__panel--${variant}`">
        <div class="editor-palette__search">
          <Icon :icon="searchIcon" class="editor-palette__search-icon" />
          <slot name="input" />
          <slot name="searchExtras" />
        </div>
        <slot name="status" />
        <slot />
        <footer class="editor-palette__footer">
          <slot name="footer">
            <span>↑↓ 选择</span>
            <span>Enter 打开</span>
            <span>Esc 关闭</span>
          </slot>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<!-- 非 scoped：列表等 class 由子组件 slot 内容使用 -->
<style lang="scss">
@use "./editorPalette.scss";
</style>
