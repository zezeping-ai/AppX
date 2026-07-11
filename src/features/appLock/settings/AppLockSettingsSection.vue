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
const lockOnWindowShow = ref(false);

const biometryStatus = ref<Awaited<ReturnType<typeof getBiometryStatusView>> | null>(null);

function applyView(view: { enabled: boolean; lockOnWindowShow: boolean }) {
  enabled.value = view.enabled;
  lockOnWindowShow.value = view.lockOnWindowShow;
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

async function persistSettings(successMessage: string) {
  const previous = { enabled: enabled.value, lockOnWindowShow: lockOnWindowShow.value };
  toggling.value = true;
  try {
    const view = await saveAppLockSettings({
      enabled: enabled.value,
      lockOnStartup: true,
      lockOnWindowShow: lockOnWindowShow.value,
    });
    applyView(view);
    message.success(successMessage);
  } catch (error) {
    enabled.value = previous.enabled;
    lockOnWindowShow.value = previous.lockOnWindowShow;
    message.error(String(error));
  } finally {
    toggling.value = false;
  }
}

async function onToggleEnabled(checked: boolean) {
  if (toggling.value) return;
  if (checked && !biometryStatus.value?.available) {
    message.error("当前设备不支持生物识别，无法启用应用锁");
    enabled.value = false;
    return;
  }
  enabled.value = checked;
  await persistSettings(checked ? "应用锁已启用" : "应用锁已关闭");
}

async function onToggleLockOnWindowShow(checked: boolean) {
  if (toggling.value || !enabled.value) return;
  lockOnWindowShow.value = checked;
  await persistSettings(
    checked ? "已开启：每次显示主窗口需解锁" : "已关闭：关闭窗口后再次打开无需解锁",
  );
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
          开启后，应用启动时需生物识别或设备密码验证一次；关闭主窗口后从托盘再次打开，默认无需重新解锁。
        </a-typography-text>

        <div
          v-if="enabled"
          class="app-lock-settings__sub app-surface-muted"
          :class="{ 'app-lock-settings__sub--disabled': toggling }"
        >
          <a-checkbox
            :checked="lockOnWindowShow"
            :disabled="toggling"
            @update:checked="onToggleLockOnWindowShow"
          >
            每次显示主窗口时需重新解锁
          </a-checkbox>
          <a-typography-text type="secondary" class="app-lock-settings__sub-hint">
            开启后，从托盘或 Dock 重新显示主窗口时也会要求验证。
          </a-typography-text>
        </div>

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

.app-lock-settings__sub {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 4px;
  padding: 10px 12px;
  border-radius: 8px;

  &--disabled {
    opacity: 0.55;
  }
}

.app-lock-settings__sub-hint {
  display: block;
  margin: 0 0 0 24px;
  font-size: 12px;
  line-height: 1.5;
}
</style>
