<script setup lang="ts">
import { computed } from "vue";
import { Icon as IconifyIcon } from "@iconify/vue";

const props = withDefaults(
  defineProps<{
    /** Iconify name, e.g. `mdi:settings` */
    icon: string;
    size?: number | string;
    color?: string;
  }>(),
  {
    size: "1em",
    color: "currentColor",
  },
);

const fontSize = computed(() =>
  typeof props.size === "number" ? `${props.size}px` : props.size,
);
</script>

<template>
  <IconifyIcon
    class="app-icon"
    :icon="icon"
    :style="{ fontSize, color }"
    aria-hidden="true"
  />
</template>

<style scoped>
.app-icon {
  display: inline-block;
  flex-shrink: 0;
  vertical-align: -0.125em;
  line-height: 1;
}
</style>
