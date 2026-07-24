<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Icon } from "@iconify/vue";
import { message } from "ant-design-vue";
import { useClipboard } from "@vueuse/core";
import { useThemePreferences, type ThemeMode } from "@/features/appearance";
import { getDatabasePath, revealDatabaseInFolder } from "@/modules/app";

const { themeMode } = useThemePreferences();
const { copy } = useClipboard({ legacy: true });

const dbPath = ref("");
const dbLoading = ref(false);
const revealing = ref(false);

function onThemeGroupChange(event: { target: { value?: unknown } }) {
  const nextTheme = event.target.value;
  if (nextTheme === "system" || nextTheme === "light" || nextTheme === "dark") {
    themeMode.value = nextTheme satisfies ThemeMode;
  }
}

async function refreshDatabasePath() {
  dbLoading.value = true;
  try {
    dbPath.value = await getDatabasePath();
  } catch (error) {
    message.error(String(error));
  } finally {
    dbLoading.value = false;
  }
}

async function onCopyPath() {
  if (!dbPath.value) return;
  await copy(dbPath.value);
  message.success("路径已复制");
}

async function onRevealInFolder() {
  revealing.value = true;
  try {
    await revealDatabaseInFolder();
  } catch (error) {
    message.error(String(error));
  } finally {
    revealing.value = false;
  }
}

onMounted(() => {
  void refreshDatabasePath();
});
</script>

<template>
  <a-space direction="vertical" size="small" class="app-settings">
    <a-card title="主题" size="small" :bordered="false">
      <a-radio-group :value="themeMode" size="small" @change="onThemeGroupChange">
        <a-radio-button value="system">跟随系统</a-radio-button>
        <a-radio-button value="light">浅色</a-radio-button>
        <a-radio-button value="dark">深色</a-radio-button>
      </a-radio-group>
    </a-card>

    <a-card title="数据存储" size="small" :bordered="false" :loading="dbLoading">
      <a-space direction="vertical" size="small" class="w-full">
        <a-input :value="dbPath" size="small" readonly>
          <template #suffix>
            <a-tooltip title="复制路径">
              <a-button type="text" size="small" :disabled="!dbPath" @click="onCopyPath">
                <Icon icon="mdi:content-copy" width="14" height="14" />
              </a-button>
            </a-tooltip>
          </template>
        </a-input>
        <a-space size="small">
          <a-button size="small" :loading="revealing" :disabled="!dbPath" @click="onRevealInFolder">
            <template #icon>
              <Icon icon="mdi:folder-open-outline" width="14" height="14" />
            </template>
            在文件管理器中显示
          </a-button>
          <a-button size="small" :disabled="dbLoading" @click="refreshDatabasePath">刷新</a-button>
        </a-space>
      </a-space>
    </a-card>
  </a-space>
</template>

<style scoped lang="scss">
.app-settings {
  width: 100%;
  max-width: 640px;
}
</style>
