<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Icon } from "@iconify/vue";
import { useThemePreferences } from "@/features/appearance";
import {
  PREFERENCE_SECTIONS,
  type PreferenceSectionKey,
} from "@/features/preferences/constants/sections";

const router = useRouter();
const route = useRoute();
const { resolvedTheme } = useThemePreferences();
const siderTheme = computed(() => (resolvedTheme.value === "dark" ? "dark" : "light"));

const activeKey = computed<PreferenceSectionKey>({
  get: () =>
    (typeof route.query.section === "string" &&
      (route.query.section as PreferenceSectionKey)) ||
    "appearance",
  set: (value) => {
    router.replace({
      query: {
        ...route.query,
        section: value,
      },
    });
  },
});

const selectedKeys = computed<string[]>({
  get: () => [activeKey.value],
  set: (keys) => {
    const key = (keys?.[0] || "appearance") as PreferenceSectionKey;
    activeKey.value = key;
  },
});

const activeSection = computed(
  () =>
    PREFERENCE_SECTIONS.find((section) => section.key === activeKey.value) ??
    PREFERENCE_SECTIONS[0],
);
</script>

<template>
  <a-layout class="min-h-screen h-screen overflow-hidden">
    <a-layout-sider
      width="220"
      :theme="siderTheme"
      class="h-screen sticky top-0 overflow-auto border-r border-black/10"
    >
      <div class="px-3.5 pt-3.5 pb-2.5">
        <a-typography-title :level="5" class="m-0!">偏好设置</a-typography-title>
        <a-typography-text type="secondary" class="block mt-1.5 text-[11px] leading-snug">
          配置会自动保存到本地
        </a-typography-text>
      </div>

      <a-menu v-model:selected-keys="selectedKeys" mode="inline">
        <a-menu-item v-for="section in PREFERENCE_SECTIONS" :key="section.key">
          <a-space>
            <Icon :icon="section.icon" aria-hidden="true" />
            <span>{{ section.label }}</span>
          </a-space>
        </a-menu-item>
      </a-menu>
    </a-layout-sider>

    <a-layout class="h-screen overflow-hidden">
      <a-layout-content class="h-screen overflow-auto p-4 bg-[var(--app-bg,#f5f5f5)]">
        <div class="mx-auto flex w-full max-w-4xl flex-col">
          <component :is="activeSection.component" />
        </div>
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>
