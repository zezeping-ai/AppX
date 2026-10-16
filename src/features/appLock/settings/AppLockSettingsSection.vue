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
import { getErrorMessage } from "@/shared/error";
import SecurityEncryptionSection from "./SecurityEncryptionSection.vue";

const loading = ref(false);
const toggling = ref(false);

const enabled = ref(false);
const lockOnWindowShow = ref(false);
const hasUnlockPin = ref(false);
const unlockPin = ref("");
const unlockPinConfirm = ref("");

const biometryStatus = ref<Awaited<ReturnType<typeof getBiometryStatusView>> | null>(null);

function applyView(view: {
  enabled: boolean;
  lockOnWindowShow: boolean;
  hasUnlockPin: boolean;
}) {
  enabled.value = view.enabled;
  lockOnWindowShow.value = view.lockOnWindowShow;
  hasUnlockPin.value = view.hasUnlockPin;
}

async function refresh() {
  loading.value = true;
  try {
    const view = await getAppLockSettings();
    applyView(view);
  } catch (error) {
    message.error(getErrorMessage(error, "加载应用锁设置失败"));
  } finally {
    loading.value = false;
  }
}

async function refreshBiometryStatus() {
  biometryStatus.value = await getBiometryStatusView();
}

async function persistSettings(
  successMessage: string,
  options?: { unlockPin?: string },
) {
  const previous = {
    enabled: enabled.value,
    lockOnWindowShow: lockOnWindowShow.value,
    hasUnlockPin: hasUnlockPin.value,
  };
  toggling.value = true;
  try {
    const view = await saveAppLockSettings({
      enabled: enabled.value,
      lockOnStartup: true,
      lockOnWindowShow: lockOnWindowShow.value,
      unlockPin: options?.unlockPin,
    });
    applyView(view);
    if (options?.unlockPin) {
      unlockPin.value = "";
      unlockPinConfirm.value = "";
    }
    message.success(successMessage);
  } catch (error) {
    enabled.value = previous.enabled;
    lockOnWindowShow.value = previous.lockOnWindowShow;
    hasUnlockPin.value = previous.hasUnlockPin;
    message.error(getErrorMessage(error, "保存失败"));
  } finally {
    toggling.value = false;
  }
}

async function onSavePin() {
  if (toggling.value) return;
  const pin = unlockPin.value.trim();
  const confirm = unlockPinConfirm.value.trim();
  if (pin.length < 4) {
    message.warning("解锁 PIN 至少 4 位");
    return;
  }
  if (pin !== confirm) {
    message.warning("两次输入的 PIN 不一致");
    return;
  }
  await persistSettings(hasUnlockPin.value ? "解锁 PIN 已更新" : "解锁 PIN 已设置", {
    unlockPin: pin,
  });
}

async function onToggleEnabled(checked: boolean) {
  if (toggling.value) return;
  if (checked && !hasUnlockPin.value) {
    const pin = unlockPin.value.trim();
    const confirm = unlockPinConfirm.value.trim();
    if (pin.length < 4) {
      message.error("启用应用锁前请先设置解锁 PIN（至少 4 位）");
      enabled.value = false;
      return;
    }
    if (pin !== confirm) {
      message.error("两次输入的 PIN 不一致");
      enabled.value = false;
      return;
    }
    enabled.value = true;
    await persistSettings("应用锁已启用", { unlockPin: pin });
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
    message.error(getErrorMessage(error, "生物识别失败"));
  }
}

onMounted(() => {
  void refresh();
  void refreshBiometryStatus();
});
</script>

<template>
  <a-space direction="vertical" size="small" class="app-lock-settings">
    <SecurityEncryptionSection />
    <a-card title="应用锁" size="small" :bordered="false" :loading="loading">
      <a-space direction="vertical" size="small" class="w-full">
        <a-checkbox :checked="enabled" :disabled="toggling" @update:checked="onToggleEnabled">
          启用应用锁（启动时验证）
        </a-checkbox>

        <a-checkbox
          v-if="enabled"
          class="app-lock-settings__sub"
          :checked="lockOnWindowShow"
          :disabled="toggling"
          @update:checked="onToggleLockOnWindowShow"
        >
          每次显示主窗口时需重新解锁
        </a-checkbox>

        <div class="app-lock-settings__pin app-surface-muted">
          <div class="app-lock-settings__pin-title">
            解锁 PIN
            <a-tag :color="hasUnlockPin ? 'green' : 'orange'" class="ml-1">
              {{ hasUnlockPin ? "已设置" : "未设置" }}
            </a-tag>
          </div>
          <a-typography-text type="secondary" class="text-[12px]">
            解锁时由后端校验；生物识别仅作辅助确认，不能单独解锁。
          </a-typography-text>
          <a-space direction="vertical" size="small" class="w-full mt-2">
            <a-input-password
              v-model:value="unlockPin"
              placeholder="新 PIN（至少 4 位）"
              :maxlength="64"
              :disabled="toggling"
            />
            <a-input-password
              v-model:value="unlockPinConfirm"
              placeholder="再次确认 PIN"
              :maxlength="64"
              :disabled="toggling"
            />
            <a-button size="small" :disabled="toggling" @click="onSavePin">
              {{ hasUnlockPin ? "更新 PIN" : "设置 PIN" }}
            </a-button>
          </a-space>
        </div>

        <a-space size="small" wrap>
          <a-tag :color="enabled ? 'green' : 'default'">
            {{ enabled ? "已启用" : "未启用" }}
          </a-tag>
          <template v-if="biometryStatus?.available">
            <a-tag color="blue">{{ biometryStatus.typeLabel }}</a-tag>
            <a-button size="small" @click="onTestBiometry">测试验证</a-button>
          </template>
          <a-typography-text v-else type="secondary" class="text-[12px]">
            {{ biometryStatus?.reason ?? "检测中…" }}
          </a-typography-text>
        </a-space>
      </a-space>
    </a-card>
  </a-space>
</template>

<style scoped lang="scss">
.app-lock-settings {
  width: 100%;
  max-width: 640px;
}

.app-lock-settings__sub {
  margin-left: 24px;
}

.app-lock-settings__pin {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  border-radius: 6px;
}

.app-lock-settings__pin-title {
  display: flex;
  align-items: center;
  font-size: 13px;
  font-weight: 500;
}
</style>
