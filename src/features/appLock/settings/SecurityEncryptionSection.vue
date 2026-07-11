<script setup lang="ts">
import { onMounted, ref } from "vue";
import { message } from "ant-design-vue";
import { getSecuritySettings } from "@/modules/appLock";

const loading = ref(false);
const passphrase = ref("");
const editable = ref(false);

async function refresh() {
  loading.value = true;
  try {
    const view = await getSecuritySettings();
    passphrase.value = view.defaultEncryptionPassphrase;
    editable.value = view.defaultEncryptionPassphraseEditable;
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <a-card title="默认加密口令" :bordered="false" :loading="loading">
    <a-space direction="vertical" size="small" class="w-full">
      <a-input-password
        :value="passphrase"
        :disabled="!editable"
        autocomplete="new-password"
        readonly
      />
      <a-typography-text type="secondary" class="block text-[12px] leading-snug">
        应用内统一使用的默认加密口令；各功能未单独配置时，将使用此口令。
      </a-typography-text>
      <a-tag color="default">暂不可修改</a-tag>
    </a-space>
  </a-card>
</template>
