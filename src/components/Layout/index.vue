<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute } from "vue-router";
import { Icon } from "@iconify/vue";
import { APP_NAV_ITEMS } from "./nav";

const route = useRoute();
const collapsed = ref(true);
const visibleNavItems = computed(() => APP_NAV_ITEMS.filter((item) => item.enabled !== false));

const activeNavKey = computed(() => {
  const feature = route.meta.feature;
  return typeof feature === "string" ? feature : null;
});
</script>

<template>
  <div class="app-layout">
    <aside class="app-layout__sidebar" :class="{ 'app-layout__sidebar--collapsed': collapsed }">
      <button
        class="app-layout__collapse-btn"
        type="button"
        :aria-label="collapsed ? '展开侧边栏' : '折叠侧边栏'"
        @click="collapsed = !collapsed"
      >
        <Icon :icon="collapsed ? 'mdi:chevron-right' : 'mdi:chevron-left'" aria-hidden="true" />
      </button>

      <nav class="app-layout__nav" aria-label="功能导航">
        <router-link
          v-for="item in visibleNavItems"
          :key="item.key"
          :to="item.to"
          class="app-layout__nav-item"
          :class="{ 'app-layout__nav-item--active': activeNavKey === item.key }"
          :title="collapsed ? item.label : undefined"
        >
          <Icon :icon="item.icon" aria-hidden="true" />
          <span v-if="!collapsed">{{ item.label }}</span>
        </router-link>
      </nav>
    </aside>

    <main class="app-layout__main">
      <router-view />
    </main>
  </div>
</template>

<style scoped lang="scss">
.app-layout {
  display: flex;
  height: 100vh;
  min-height: 0;
  background: var(--app-bg, #f5f5f5);
  color: var(--app-fg, rgba(0, 0, 0, 0.88));
}

.app-layout__sidebar {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 200px;
  min-width: 0;
  height: 100%;
  padding: 10px 8px;
  border-right: 1px solid rgba(0, 0, 0, 0.08);
  background: var(--app-layout-sidebar-bg, #fff);
  transition: width 0.2s ease;
}

.app-layout__sidebar--collapsed {
  width: 56px;
}

.app-layout__collapse-btn {
  width: 100%;
  height: 28px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: inherit;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;

  &:hover {
    background: rgba(0, 0, 0, 0.05);
  }
}

.app-layout__nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-height: 0;
}

.app-layout__nav-item {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  width: 100%;
  padding: 0 10px;
  border-radius: 6px;
  color: inherit;
  font-size: 12px;
  text-decoration: none;
  opacity: 0.72;
  transition:
    background-color 0.15s ease,
    opacity 0.15s ease;

  &:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.04);
  }

  &--active {
    opacity: 1;
    font-weight: 500;
    background: rgba(22, 119, 255, 0.1);
    color: #1677ff;
  }
}

.app-layout__main {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

[data-theme="dark"] .app-layout__sidebar {
  border-right-color: rgba(255, 255, 255, 0.1);
  --app-layout-sidebar-bg: #1f1f1f;
}

[data-theme="dark"] .app-layout__nav-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

[data-theme="dark"] .app-layout__collapse-btn:hover {
  background: rgba(255, 255, 255, 0.08);
}

[data-theme="dark"] .app-layout__nav-item--active {
  background: rgba(22, 119, 255, 0.18);
  color: #69b1ff;
}
</style>
