<script lang="ts" setup>
import { Empty as AEmpty, Pagination as APagination } from "ant-design-vue";
import HistoryGrid from "@/pages/clipboard-assistant/components/HistoryGrid.vue";
import { useClipboardHistory } from "@/pages/clipboard-assistant/useClipboardHistory";

const {
  status,
  pageBlocked,
  sessionLocked,
  showSourceAppIcon,
  list,
  listPagination,
  onSearch,
  openDetail,
  togglePin,
  removeItem,
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

    <div v-else class="clipboard-assistant-page__panel">
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

      <div class="clipboard-assistant-page__list">
        <a-spin :spinning="list.isLoading">
          <HistoryGrid
            v-if="list.records.length"
            :items="list.records"
            :show-source-app-icon="showSourceAppIcon"
            @detail="openDetail"
            @pin="togglePin"
            @delete="removeItem"
          />
          <div v-else class="clipboard-assistant-page__empty">
            <AEmpty description="暂无历史记录" />
          </div>
        </a-spin>
      </div>

      <div class="clipboard-assistant-page__pager">
        <APagination v-bind="listPagination" />
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.clipboard-assistant-page {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 16px 20px 24px;
  box-sizing: border-box;
}

.clipboard-assistant-page__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
  flex-shrink: 0;
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
  flex-shrink: 0;
}

.clipboard-assistant-page__panel {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--app-surface);
  border-radius: 8px;
  padding: 12px 16px 16px;
  box-sizing: border-box;
}

.clipboard-assistant-page__toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.clipboard-assistant-page__list {
  flex: 1;
  min-height: 0;
  overflow: auto;
}

.clipboard-assistant-page__empty {
  min-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.clipboard-assistant-page__pager {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
  flex-shrink: 0;
}
</style>
