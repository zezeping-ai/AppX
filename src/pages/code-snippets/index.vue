<script lang="tsx" setup>
import { Icon } from "@iconify/vue";
import { App as AntApp, Button, Tag } from "ant-design-vue";
import type { ColumnsType } from "ant-design-vue/es/table";
import { computed, h, onMounted } from "vue";
import ListQueryBar from "@/pages/code-snippets/components/ListQueryBar/index.vue";
import ShortcutRecorder from "@/components/ShortcutRecorder/index.vue";
import { useDrawer, useList, useListPagination } from "@/hooks";
import { CodeSnippetRecord } from "@/models";
import {
  CODE_SNIPPET_GROUPS,
  isPasswordSnippetGroup,
  labelOfSnippetGroup,
  syncAllSnippetsToRuntime,
  formatAbbreviationTrigger,
  inlineExpansionTrigger,
  inlineExpansionTriggerLabel,
  type CodeSnippetGroup,
} from "@/modules/codeSnippets";
import { decryptText } from "@/modules/crypto";
import { setGlobalShortcutsPaused } from "@/modules/globalShortcut";
import SnippetForm from "@/pages/code-snippets/Form.vue";
import { getErrorMessage } from "@/shared/error";
import { formatShortcutLabel, normalizeGlobalShortcut } from "@/shared/shortcut";

const { message, modal: antdModal } = AntApp.useApp();

const list = useList<CodeSnippetRecord>({
  query: {
    keyword: "",
    shortcut: "",
    group: undefined as CodeSnippetGroup | undefined,
  },
  pagination: { pageSize: 20 },
  async onLoad() {
    const where: Record<string, unknown> = {};
    const keyword = String(list.query.keyword ?? "").trim();
    if (keyword) where.keyword = keyword;
    const shortcut = normalizeGlobalShortcut(String(list.query.shortcut ?? ""));
    if (shortcut) where.shortcut = shortcut;
    if (list.query.group) where.group = list.query.group;

    const ret = await CodeSnippetRecord.query({
      where,
      pagination: { page: list.pagination.page, pageSize: list.pagination.pageSize },
    });
    list.records = ret.data;
    list.pagination.total = ret.total;
    await syncAllSnippetsToRuntime();
  },
});

const { pagination: listPagination } = useListPagination(list, {
  showSizeChanger: false,
});

const drawer = useDrawer({ width: 760, placement: "right", destroyOnClose: true });

function rowKey(row: CodeSnippetRecord) {
  return row.id as number;
}

function onSearch() {
  void list.onLoad({ page: 1, pageSize: list.pagination.pageSize });
}

function onShortcutFilter(value: string) {
  list.query.shortcut = value ?? "";
  onSearch();
}

function onReset() {
  void list.onReset();
}

async function onForm(record: CodeSnippetRecord | null) {
  let plainContent = "";
  if (record) {
    try {
      plainContent = await decryptText(record.content);
    } catch (error) {
      message.error(getErrorMessage(error, "读取内容失败"));
      return;
    }
  }

  const handle = drawer.create({
    title: record ? "编辑 Snippet" : "新建 Snippet",
    // 覆盖可拖拽 drawer 默认 flex，由 body 整体滚动
    bodyStyle: {
      display: "block",
      overflow: "auto",
      paddingBottom: 0,
    },
    slots: {
      default: () =>
        h(SnippetForm, {
          record,
          plainContent,
          onSubmitted: () => {
            handle.close();
            void list.onLoad();
          },
        }),
    },
  });
}

function removeRecord(row: CodeSnippetRecord) {
  antdModal.confirm({
    title: "删除 Snippet",
    content: `确认删除「${row.name}」？`,
    okText: "删除",
    okButtonProps: { danger: true },
    cancelText: "取消",
    onOk: async () => {
      try {
        await CodeSnippetRecord.destroy(row.id as number);
        message.success("已删除");
        await list.onLoad();
      } catch (error) {
        message.error(getErrorMessage(error, "删除失败"));
      }
    },
  });
}

const columns = computed<ColumnsType<CodeSnippetRecord>>(() => [
  {
    title: "代码段名称",
    dataIndex: "name",
    key: "name",
    ellipsis: true,
  },
  {
    title: "分组",
    key: "group",
    width: 100,
    customRender: ({ record }) => labelOfSnippetGroup(record.meta.group),
  },
  {
    title: "缩写",
    key: "abbreviation",
    width: 140,
    customRender: ({ record }) => (
      <Tag>{formatAbbreviationTrigger(record.abbreviation, inlineExpansionTrigger.value)}</Tag>
    ),
  },
  {
    title: "快捷键",
    key: "shortcut",
    width: 140,
    ellipsis: true,
    customRender: ({ record }) =>
      record.shortcut ? formatShortcutLabel(record.shortcut) : "—",
  },
  {
    title: "内容",
    key: "content",
    width: 100,
    customRender: ({ record }) =>
      isPasswordSnippetGroup(record.meta.group) ? "••••••" : "已加密",
  },
  {
    title: "备注",
    key: "note",
    dataIndex: ["meta", "note"],
    ellipsis: true,
  },
  {
    title: "操作",
    key: "actions",
    width: 120,
    align: "right",
    customRender: ({ record }) => (
      <div class="flex flex-wrap justify-end gap-1">
        <Button size="small" onClick={() => onForm(record)}>
          编辑
        </Button>
        <Button size="small" danger onClick={() => removeRecord(record)}>
          删除
        </Button>
      </div>
    ),
  },
]);

onMounted(() => {
  void list.onLoad();
});
</script>

<template>
  <div class="code-snippets-page">
    <header class="code-snippets-page__header">
      <div>
        <h1 class="code-snippets-page__title">代码段</h1>
        <p class="code-snippets-page__desc">
          跨软件文本展开：输入 <code>:缩写</code> 后按
          <strong>{{ inlineExpansionTriggerLabel() }}</strong>，或使用快捷键命令面板 / 全局快捷键。内容加密存储。
        </p>
      </div>
    </header>

    <ListQueryBar @search="onSearch">
      <a-form-item class="min-w-0">
        <a-input
          :value="String(list.query.keyword ?? '')"
          allow-clear
          placeholder="搜索名称 / 缩写"
          class="w-72 max-w-full"
          @update:value="(v: string) => (list.query.keyword = v ?? '')"
          @press-enter="onSearch"
        >
          <template #prefix>
            <Icon icon="mdi:magnify" class="text-black/45 dark:text-white/45" />
          </template>
        </a-input>
      </a-form-item>

      <a-form-item label="快捷键" class="min-w-0">
        <ShortcutRecorder
          :value="String(list.query.shortcut ?? '')"
          :on-recording-change="setGlobalShortcutsPaused"
          @update:value="onShortcutFilter"
        />
      </a-form-item>

      <a-form-item label="分组" class="min-w-0">
        <a-select
          :value="list.query.group"
          allow-clear
          placeholder="全部分组"
          class="w-36"
          @update:value="(v: CodeSnippetGroup | undefined) => (list.query.group = v)"
        >
          <a-select-option
            v-for="item in CODE_SNIPPET_GROUPS"
            :key="item.value"
            :value="item.value"
          >
            {{ item.label }}
          </a-select-option>
        </a-select>
      </a-form-item>

      <a-form-item class="min-w-0">
        <a-space size="small" align="center">
          <a-button
            size="small"
            type="primary"
            html-type="submit"
            :loading="list.isLoading"
            @click="onSearch"
          >
            <template #icon>
              <Icon icon="mdi:magnify" />
            </template>
            搜索
          </a-button>
          <a-button size="small" :loading="list.isLoading" @click="onReset">
            <template #icon>
              <Icon icon="mdi:refresh" />
            </template>
            重置
          </a-button>
        </a-space>
      </a-form-item>

      <template #extra>
        <a-button type="primary" size="small" @click="onForm(null)">
          <template #icon>
            <Icon icon="mdi:plus" />
          </template>
          新建
        </a-button>
      </template>
    </ListQueryBar>

    <a-card :bordered="false" class="code-snippets-page__table-card">
      <a-table
        class="code-snippets-page__table"
        :columns="columns"
        :data-source="list.records"
        :loading="list.isLoading"
        :pagination="listPagination"
        :row-key="rowKey"
        size="small"
        :scroll="{ x: 'max-content' }"
      />
    </a-card>
  </div>
</template>

<style scoped lang="scss">
.code-snippets-page {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 20px 24px;
  height: 100%;
  overflow: auto;
}

.code-snippets-page__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}

.code-snippets-page__title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.code-snippets-page__desc {
  margin: 6px 0 0;
  color: #6b7280;
  font-size: 13px;

  code {
    padding: 0 4px;
    border-radius: 4px;
    background: #f3f4f6;
  }
}

.code-snippets-page__table-card {
  min-width: 0;
  flex: 1;
}

.code-snippets-page__table {
  min-width: 0;
  width: 100%;
}
</style>
