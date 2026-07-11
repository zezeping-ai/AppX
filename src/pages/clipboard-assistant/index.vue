<script lang="tsx" setup>
import { useClipboardHistory } from "@/pages/clipboard-assistant/useClipboardHistory";

const {
  status,
  pageBlocked,
  sessionLocked,
  list,
  listPagination,
  columns,
  rowKey,
  onSearch,
  clearUnpinned,
  contentTypes,
} = useClipboardHistory();
</script>

<template>
  <div class="clipboard-assistant-page">
    <div class="clipboard-assistant-page__header">
      <div>
        <h1 class="clipboard-assistant-page__title">剪切助手</h1>
        <p v-if="status" class="clipboard-assistant-page__subtitle">
          共 {{ status.totalCount }} 条 · 未固定 {{ status.unpinnedCount }} · 存储
          {{ Math.round(status.blobBytes / 1024 / 1024) }} MB
        </p>
      </div>
      <a-space>
        <a-button @click="clearUnpinned">清空未固定</a-button>
        <a-button type="primary" @click="onSearch">刷新</a-button>
      </a-space>
    </div>

    <a-alert
      v-if="pageBlocked"
      type="warning"
      show-icon
      class="clipboard-assistant-page__alert"
      :message="
        sessionLocked
          ? '应用已锁定，请先解锁后再使用剪切助手。'
          : '剪切助手已在偏好设置中停用，请前往偏好设置 → 剪切助手开启。'
      "
    />

    <a-card v-else :bordered="false" class="clipboard-assistant-page__card">
      <div class="clipboard-assistant-page__toolbar">
        <a-input
          v-model:value="list.query.keyword"
          allow-clear
          placeholder="搜索预览内容"
          style="max-width: 280px"
          @press-enter="onSearch"
        />
        <a-select
          v-model:value="list.query.contentType"
          allow-clear
          placeholder="类型"
          style="width: 120px"
          @change="onSearch"
        >
          <a-select-option
            v-for="type in contentTypes"
            :key="type.value"
            :value="type.value"
          >
            {{ type.label }}
          </a-select-option>
        </a-select>
        <a-button @click="onSearch">搜索</a-button>
      </div>

      <a-table
        :columns="columns"
        :data-source="list.records"
        :loading="list.isLoading"
        :pagination="listPagination"
        :row-key="rowKey"
        size="small"
      />
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.clipboard-assistant-page {
  padding: 16px 20px 24px;
}

.clipboard-assistant-page__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}

.clipboard-assistant-page__title {
  margin: 0 0 4px;
  font-size: 20px;
  font-weight: 600;
}

.clipboard-assistant-page__subtitle {
  margin: 0;
  color: var(--app-fg-muted);
  font-size: 13px;
}

.clipboard-assistant-page__alert {
  margin-bottom: 16px;
}

.clipboard-assistant-page__card {
  background: var(--app-surface);
}

.clipboard-assistant-page__toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

:deep(.clipboard-assistant-page__detail-text) {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12px;
}

:deep(.clipboard-assistant-page__detail-files) {
  margin: 0;
  padding-left: 18px;
  font-size: 12px;
}
</style>
