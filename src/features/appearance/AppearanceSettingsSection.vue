<script setup lang="ts">
import { useThemePreferences, type ThemeMode } from "@/features/appearance";

const { themeMode } = useThemePreferences();

function onThemeGroupChange(event: { target: { value?: unknown } }) {
  const nextTheme = event.target.value;
  if (nextTheme === "system" || nextTheme === "light" || nextTheme === "dark") {
    themeMode.value = nextTheme satisfies ThemeMode;
  }
}
</script>

<template>
  <a-card title="主题配置" :bordered="false">
    <a-radio-group :value="themeMode" @change="onThemeGroupChange">
      <a-radio-button value="system">跟随系统</a-radio-button>
      <a-radio-button value="light">浅色</a-radio-button>
      <a-radio-button value="dark">深色</a-radio-button>
    </a-radio-group>
  </a-card>
</template>
