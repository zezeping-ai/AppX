<script setup lang="tsx">
import { Icon } from "@iconify/vue";
import { getVersion } from "@tauri-apps/api/app";
import { ref, onMounted } from "vue";

const version = ref("");

onMounted(async () => {
  version.value = await getVersion();
});

function renderHero() {
  return (
    <div class="home-hero">
      <Icon icon="mdi:application-outline" width={48} height={48} />
      <a-typography-title level={3}>AppX</a-typography-title>
      <a-typography-text type="secondary">
        Tauri + Vue 3 桌面应用脚手架，已集成托盘、自动更新与 GitHub Actions 发布流程。
      </a-typography-text>
    </div>
  );
}
</script>

<template>
  <a-layout class="home-layout">
    <a-layout-content class="home-content">
      <component :is="renderHero" />
      <a-space direction="vertical" align="center" size="middle">
        <a-typography-text>当前版本：v{{ version || "..." }}</a-typography-text>
        <a-typography-text type="secondary">
          通过菜单「帮助 → 检查更新」可检测 GitHub Release 并自动安装新版本。
        </a-typography-text>
      </a-space>
    </a-layout-content>
  </a-layout>
</template>

<style scoped lang="scss">
.home-layout {
  min-height: 100vh;
  background: var(--app-bg, #f5f5f5);
}

.home-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
  padding: 48px 24px;
}

.home-hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
  max-width: 520px;
}
</style>
