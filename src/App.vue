<script setup lang="ts">
import { computed } from "vue";
import { theme as antdTheme } from "ant-design-vue";

const configTheme = computed(() => ({
  algorithm: antdTheme.defaultAlgorithm,
}));

const componentSize = computed(() => "small" as const);
</script>

<template>
  <a-config-provider :theme="configTheme" :component-size="componentSize">
    <router-view />
  </a-config-provider>
</template>
