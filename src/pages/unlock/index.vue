<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { message } from "ant-design-vue";
import {
  authenticateBiometry,
  getBiometryStatusView,
  getAppLockSettings,
  isBiometryUserDismissed,
  unlockAppLockSession,
} from "@/modules/appLock";
import { bootstrapAfterUnlock } from "@/modules/bootstrapAfterUnlock";
import { getErrorMessage } from "@/shared/error";

const router = useRouter();
const route = useRoute();

const loading = ref(false);
const unlocking = ref(false);
const pin = ref("");
const view = ref<Awaited<ReturnType<typeof getAppLockSettings>> | null>(null);
const biometryStatus = ref<Awaited<ReturnType<typeof getBiometryStatusView>> | null>(null);

const redirectTo = computed(() => {
  const raw = route.query.redirect;
  return typeof raw === "string" && raw.trim() ? raw : "/editor";
});

const needsPin = computed(() => view.value?.hasUnlockPin ?? false);

async function refresh() {
  loading.value = true;
  try {
    view.value = await getAppLockSettings();
  } catch (error) {
    message.error(getErrorMessage(error, "加载应用锁状态失败"));
  } finally {
    loading.value = false;
  }
}

async function refreshBiometryStatus() {
  biometryStatus.value = await getBiometryStatusView();
}

async function unlockAndRedirect(pinValue: string) {
  unlocking.value = true;
  try {
    await unlockAppLockSession({ pin: pinValue });
    await bootstrapAfterUnlock().catch((error) => {
      console.warn("[bootstrap] post-unlock sync failed:", error);
    });
    await router.replace(redirectTo.value);
  } catch (error) {
    message.error(getErrorMessage(error, "解锁失败"));
  } finally {
    unlocking.value = false;
  }
}

async function onSubmitPin() {
  if (needsPin.value && !pin.value.trim()) {
    message.warning("请输入解锁 PIN");
    return;
  }
  await unlockAndRedirect(pin.value);
}

async function tryBiometry() {
  try {
    await authenticateBiometry("请验证以解锁 AppX");
    // 生物识别仅作身份确认，解锁仍须后端校验 PIN
    if (needsPin.value) {
      if (pin.value.trim()) {
        await unlockAndRedirect(pin.value);
      } else {
        message.info("生物识别通过，请输入解锁 PIN");
      }
      return;
    }
    await unlockAndRedirect("");
  } catch (error) {
    if (isBiometryUserDismissed(error)) return;
    message.error(getErrorMessage(error, "生物识别失败"));
  }
}

onMounted(async () => {
  await refresh();
  await refreshBiometryStatus();

  const v = view.value;
  if (!v || !v.enabled || !v.sessionLocked) {
    await router.replace(redirectTo.value);
    return;
  }

  if (biometryStatus.value?.available && !needsPin.value) {
    await tryBiometry();
  }
});
</script>

<template>
  <div class="unlock-page">
    <a-card class="unlock-card" :bordered="false" :loading="loading">
      <a-space direction="vertical" size="middle" class="w-full">
        <div>
          <a-typography-title :level="4" class="m-0!">解锁 AppX</a-typography-title>
          <a-typography-text type="secondary" class="text-[12px]">
            {{
              needsPin
                ? "请输入解锁 PIN；可先使用生物识别确认身份"
                : "可使用生物识别解锁（建议尽快在偏好设置中配置 PIN）"
            }}
          </a-typography-text>
        </div>

        <a-alert
          v-if="biometryStatus && !biometryStatus.available"
          type="info"
          :message="`生物识别不可用：${biometryStatus.reason}`"
          show-icon
        />
        <a-alert
          v-else-if="biometryStatus?.available"
          type="success"
          :message="`生物识别可用：${biometryStatus.typeLabel}`"
          show-icon
        />

        <a-input-password
          v-model:value="pin"
          size="large"
          placeholder="解锁 PIN"
          :maxlength="64"
          :disabled="unlocking"
          @press-enter="onSubmitPin"
        />

        <a-space>
          <a-button type="primary" :loading="unlocking" @click="onSubmitPin">
            解锁
          </a-button>
          <a-button
            v-if="biometryStatus?.available"
            :disabled="unlocking"
            @click="tryBiometry"
          >
            使用生物识别
          </a-button>
        </a-space>
      </a-space>
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.unlock-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: radial-gradient(80% 80% at 50% 20%, #ffffff 0%, #f5f5f5 60%, #eef2ff 100%);
}

.unlock-card {
  width: min(520px, 100%);
  border-radius: 12px;
}
</style>
