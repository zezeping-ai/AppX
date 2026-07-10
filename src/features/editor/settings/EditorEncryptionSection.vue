<script setup lang="ts">
import { Modal, message } from "ant-design-vue";
import { computed, onMounted, ref } from "vue";
import {
  getEditorSettings,
  saveEditorEncryptionPassphrase,
} from "@/modules/editor/settings/client";

const SECURITY_PASSPHRASE_SOURCE = "偏好设置 › 安全 › 默认加密口令";

const loading = ref(false);
const saving = ref(false);
const passphrase = ref("");
const savedPassphrase = ref("");
const usesGlobalPassphrase = ref(true);

const canSave = computed(
  () => passphrase.value.trim() !== savedPassphrase.value.trim(),
);

const globalPassphraseHint = computed(
  () => `未单独配置，使用「${SECURITY_PASSPHRASE_SOURCE}」`,
);

async function refresh() {
  loading.value = true;
  try {
    const view = await getEditorSettings();
    passphrase.value = view.encryption.passphrase;
    savedPassphrase.value = view.encryption.passphrase;
    usesGlobalPassphrase.value = view.encryption.usesGlobalPassphrase;
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function onSave() {
  const trimmed = passphrase.value.trim();
  const hadCustomPassphrase = savedPassphrase.value.trim().length > 0;

  if (hadCustomPassphrase || trimmed.length > 0) {
    const confirmed = await new Promise<boolean>((resolve) => {
      Modal.confirm({
        title: trimmed ? "确认修改 Editor 加密口令" : "确认清除独立配置",
        content: trimmed
          ? "修改后，旧口令加密的 .x 文件将无法解密。请确认已备份并记住新口令。"
          : `清除后，Editor 的 .x 加密将改用「${SECURITY_PASSPHRASE_SOURCE}」。`,
        okText: "确认",
        okType: trimmed ? "danger" : "primary",
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
  }

  saving.value = true;
  try {
    const view = await saveEditorEncryptionPassphrase(passphrase.value);
    passphrase.value = view.encryption.passphrase;
    savedPassphrase.value = view.encryption.passphrase;
    usesGlobalPassphrase.value = view.encryption.usesGlobalPassphrase;
    message.success(
      usesGlobalPassphrase.value
        ? `已清除独立配置，改用「${SECURITY_PASSPHRASE_SOURCE}」`
        : "已保存独立加密口令",
    );
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

    <a-typography-text
      v-if="usesGlobalPassphrase"
      type="secondary"
      class="editor-encryption__status"
    >
      {{ globalPassphraseHint }}
    </a-typography-text>

    <div class="editor-encryption__body">
      <a-input-password
        v-model:value="passphrase"
        :placeholder="`留空则使用「${SECURITY_PASSPHRASE_SOURCE}」`"
        autocomplete="new-password"
        allow-clear
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
  margin-bottom: 8px;
}

.editor-encryption__title {
  margin: 0;
}

.editor-encryption__status {
  display: block;
  margin-bottom: 12px;
  font-size: 12px;
  line-height: 1.5;
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
