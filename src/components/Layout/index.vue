<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Icon } from "@iconify/vue";
import { APP_NAV_ITEMS } from "./nav";
import { isAppSessionLocked } from "@/modules/appLock";
import { UNLOCK_ROUTE_PATH } from "@/router/standalone";

const router = useRouter();
const route = useRoute();
const collapsed = ref(true);
const visibleNavItems = computed(() => APP_NAV_ITEMS.filter((item) => item.enabled !== false));

const activeNavKey = computed(() => {
  const feature = route.meta.feature;
  return typeof feature === "string" ? feature : null;
});

async function checkLockOnFocus() {
  try {
    if (await isAppSessionLocked()) {
      await router.replace({
        path: UNLOCK_ROUTE_PATH,
        query: { redirect: route.fullPath },
      });
    }
  } catch {
    // 忽略异常，避免在极端情况下影响使用
  }
}

function onFocus() {
  void checkLockOnFocus();
}

onMounted(() => {
  window.addEventListener("focus", onFocus);
  document.addEventListener("visibilitychange", onFocus);
});

onUnmounted(() => {
  window.removeEventListener("focus", onFocus);
  document.removeEventListener("visibilitychange", onFocus);
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
  background: var(--app-bg);
  color: var(--app-fg);
}

.app-layout__sidebar {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 200px;
  min-width: 0;
  height: 100%;
  padding: 10px 8px;
  border-right: 1px solid var(--app-border);
  background: var(--app-layout-sidebar-bg);
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
    background: var(--app-hover-bg-strong);
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
    background: var(--app-hover-bg);
  }

  &--active {
    opacity: 1;
    font-weight: 500;
    background: var(--app-active-bg);
    color: var(--app-active-fg);
  }
}

.app-layout__main {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
</style>
