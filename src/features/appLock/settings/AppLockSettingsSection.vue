<script setup lang="ts">
import { onMounted, ref } from "vue";
import { message } from "ant-design-vue";
import {
  authenticateBiometry,
  getBiometryStatusView,
  getAppLockSettings,
  isBiometryUserDismissed,
  saveAppLockSettings,
} from "@/modules/appLock";
import SecurityEncryptionSection from "./SecurityEncryptionSection.vue";

const loading = ref(false);
const toggling = ref(false);

const enabled = ref(false);
const lockOnStartup = true;

const biometryStatus = ref<Awaited<ReturnType<typeof getBiometryStatusView>> | null>(null);

function applyView(view: { enabled: boolean }) {
  enabled.value = view.enabled;
}

async function refresh() {
  loading.value = true;
  try {
    const view = await getAppLockSettings();
    applyView(view);
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function refreshBiometryStatus() {
  biometryStatus.value = await getBiometryStatusView();
}

async function onToggleEnabled(checked: boolean) {
  if (toggling.value) return;
  if (checked && !biometryStatus.value?.available) {
    message.error("当前设备不支持生物识别，无法启用应用锁");
    enabled.value = false;
    return;
  }
  const previous = enabled.value;
  enabled.value = checked;
  toggling.value = true;
  try {
    const view = await saveAppLockSettings({
      enabled: checked,
      lockOnStartup,
    });
    applyView(view);
    message.success(checked ? "应用锁已启用" : "应用锁已关闭");
  } catch (error) {
    enabled.value = previous;
    message.error(String(error));
  } finally {
    toggling.value = false;
  }
}

async function onTestBiometry() {
  try {
    await authenticateBiometry("请验证以测试生物识别");
    message.success("验证成功");
  } catch (error) {
    if (isBiometryUserDismissed(error)) return;
    message.error(String(error));
  }
}

onMounted(() => {
  void refresh();
  void refreshBiometryStatus();
});
</script>

<template>
  <a-space direction="vertical" size="middle" class="app-lock-settings">
    <SecurityEncryptionSection />
    <a-card title="应用锁" :bordered="false" :loading="loading">
      <a-space direction="vertical" size="small" class="w-full">
        <a-checkbox :checked="enabled" :disabled="toggling" @update:checked="onToggleEnabled">
          启用应用锁
        </a-checkbox>
        <a-typography-text type="secondary" class="block text-[12px] leading-snug">
          开启后，应用启动时需要生物识别或设备密码验证。
        </a-typography-text>
        <a-tag :color="enabled ? 'green' : 'default'">
          {{ enabled ? "已启用" : "未启用" }}
        </a-tag>
        <template v-if="biometryStatus?.available">
          <a-typography-text>可用：{{ biometryStatus.typeLabel }}</a-typography-text>
          <a-button @click="onTestBiometry">
            测试验证
          </a-button>
        </template>
        <template v-else>
          <a-typography-text type="secondary">
            不可用：{{ biometryStatus?.reason ?? "检测中…" }}
          </a-typography-text>
        </template>
      </a-space>
    </a-card>
  </a-space>
</template>

<style scoped lang="scss">
.app-lock-settings {
  width: 100%;
  max-width: 760px;
}
</style>
