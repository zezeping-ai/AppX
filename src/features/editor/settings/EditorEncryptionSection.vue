<script setup lang="ts">
import { Modal, message } from "ant-design-vue";
import { computed, onMounted, ref } from "vue";
import {
  getEditorSettings,
  saveEditorEncryptionPassphrase,
} from "@/modules/editor/settings/client";

const DEFAULT_PASSPHRASE = "zezeping";

const loading = ref(false);
const saving = ref(false);
const passphrase = ref(DEFAULT_PASSPHRASE);
const savedPassphrase = ref(DEFAULT_PASSPHRASE);

const canSave = computed(() => passphrase.value.trim().length > 0);

async function refresh() {
  loading.value = true;
  try {
    const view = await getEditorSettings();
    passphrase.value = view.encryption.passphrase;
    savedPassphrase.value = view.encryption.passphrase;
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function onSave() {
  if (!canSave.value) return;
  const confirmed = await new Promise<boolean>((resolve) => {
    Modal.confirm({
      title: "确认修改加密口令",
      content: "修改后，旧口令加密的 .x 文件将无法解密。请确认已备份并记住新口令。",
      okText: "确认修改",
      okType: "danger",
      cancelText: "取消",
      onOk: () => resolve(true),
      onCancel: () => {
        passphrase.value = savedPassphrase.value;
        resolve(false);
      },
    });
  });
  if (!confirmed) {
    return;
  }

  saving.value = true;
  try {
    const view = await saveEditorEncryptionPassphrase(passphrase.value);
    passphrase.value = view.encryption.passphrase;
    savedPassphrase.value = view.encryption.passphrase;
    message.success("已保存");
  } catch (error) {
    message.error(String(error));
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <a-card :bordered="false" :loading="loading" class="editor-encryption">
    <div class="editor-encryption__header">
      <a-typography-title :level="5" class="editor-encryption__title">加密（.x）</a-typography-title>
    </div>

    <div class="editor-encryption__body">
      <a-input-password
        v-model:value="passphrase"
        placeholder="输入加密口令"
        autocomplete="new-password"
      />
      <a-button type="primary" :loading="saving" :disabled="!canSave" @click="onSave">
        保存
      </a-button>
    </div>
  </a-card>
</template>

<style scoped lang="scss">
.editor-encryption {
  width: 100%;
  border-radius: 10px;

  :deep(.ant-card-body) {
    padding: 16px;
  }
}

.editor-encryption__header {
  margin-bottom: 12px;
}

.editor-encryption__title {
  margin: 0;
}

.editor-encryption__body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
}

@media (max-width: 640px) {
  .editor-encryption__body {
    grid-template-columns: 1fr;
  }
}
</style>
