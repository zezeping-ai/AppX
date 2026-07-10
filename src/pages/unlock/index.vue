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

const router = useRouter();
const route = useRoute();

const loading = ref(false);
const view = ref<Awaited<ReturnType<typeof getAppLockSettings>> | null>(null);
const biometryStatus = ref<Awaited<ReturnType<typeof getBiometryStatusView>> | null>(null);

const redirectTo = computed(() => {
  const raw = route.query.redirect;
  return typeof raw === "string" && raw.trim() ? raw : "/editor";
});

async function refresh() {
  loading.value = true;
  try {
    view.value = await getAppLockSettings();
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function refreshBiometryStatus() {
  biometryStatus.value = await getBiometryStatusView();
}

async function unlockAndRedirect() {
  try {
    await unlockAppLockSession();
    await router.replace(redirectTo.value);
  } catch (error) {
    message.error(String(error));
  }
}

async function tryBiometry() {
  try {
    await authenticateBiometry("请验证以解锁 AppX");
    await unlockAndRedirect();
  } catch (error) {
    if (isBiometryUserDismissed(error)) return;
    message.error(String(error));
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

  if (biometryStatus.value?.available) {
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
            可使用生物识别或设备登录密码解锁
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

        <a-space>
          <a-button
            v-if="biometryStatus?.available"
            type="primary"
            @click="tryBiometry"
          >
            使用生物识别解锁
          </a-button>
        </a-space>
        <a-alert
          v-if="biometryStatus && !biometryStatus.available"
          type="warning"
          message="当前设备不可用生物识别，请在支持生物识别的设备上使用应用锁。"
          show-icon
        />
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
