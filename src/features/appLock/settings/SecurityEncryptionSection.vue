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
  <a-card title="默认加密口令" size="small" :bordered="false" :loading="loading">
    <a-space direction="vertical" size="small" class="w-full">
      <a-input-password
        :value="passphrase"
        size="small"
        :disabled="!editable"
        autocomplete="new-password"
        readonly
      />
      <a-tag v-if="!editable" color="default">暂不可修改</a-tag>
    </a-space>
  </a-card>
</template>
