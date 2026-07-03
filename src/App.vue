<script setup lang="ts">
import { computed, provide } from "vue";
import { theme as antdTheme } from "ant-design-vue";
import zhCN from "ant-design-vue/es/locale/zh_CN";
import { useThemePreferences } from "@/features/appearance";
import { antdProgrammaticRootConfigKey } from "@/hooks/antdProgrammaticContext";

const { resolvedTheme } = useThemePreferences();

const configTheme = computed(() => ({
  algorithm:
    resolvedTheme.value === "dark"
      ? [antdTheme.darkAlgorithm, antdTheme.compactAlgorithm]
      : [antdTheme.defaultAlgorithm, antdTheme.compactAlgorithm],
}));

const antAppConfig = computed(() => ({
  theme: configTheme.value,
  componentSize: "small" as const,
  locale: zhCN,
}));

/** 供 useModal / useDialog / useDrawer 程序化挂载时继承主题与尺寸 */
provide(antdProgrammaticRootConfigKey, antAppConfig);
</script>

<template>
  <a-config-provider v-bind="antAppConfig">
    <a-app class="app-root">
      <router-view />
    </a-app>
  </a-config-provider>
</template>

<style scoped lang="scss">
.app-root {
  min-height: 100vh;
  background: var(--app-bg, #f5f5f5);
  color: var(--app-fg, rgba(0, 0, 0, 0.88));
}
</style>
