import { Icon } from "@iconify/vue";
import { App as AntApp, Button, Tag } from "ant-design-vue";
import type { ColumnsType } from "ant-design-vue/es/table";
import { computed, onMounted, ref } from "vue";
import { useDrawer, useList, useListPagination } from "@/hooks";
import {
  CLIPBOARD_CONTENT_TYPES,
  getContent,
  getSettings,
  getStatus,
  iconOfContentType,
  labelOfContentType,
  listItems,
  mutateItems,
  type ClipboardItemSummary,
  type ContentType,
} from "@/modules/clipboardAssistant";
import { isAppSessionLocked } from "@/modules/appLock";
import { getErrorMessage } from "@/shared/error";

export function useClipboardHistory() {
  const { message, modal } = AntApp.useApp();
  const status = ref<Awaited<ReturnType<typeof getStatus>> | null>(null);
  const featureEnabled = ref(true);
  const sessionLocked = ref(false);
  const pageBlocked = computed(
    () => sessionLocked.value || !featureEnabled.value,
  );

  const list = useList<ClipboardItemSummary>({
    query: {
      keyword: "",
      contentType: undefined as ContentType | undefined,
    },
    pagination: { pageSize: 20 },
    async onLoad() {
      if (pageBlocked.value) {
        list.records = [];
        list.pagination.total = 0;
        return;
      }
      const keyword = String(list.query.keyword ?? "").trim();
      const contentType = list.query.contentType as ContentType | undefined;
      const ret = await listItems({
        keyword: keyword || undefined,
        contentType,
        limit: list.pagination.pageSize,
        offset: (list.pagination.page - 1) * list.pagination.pageSize,
        preferCache: false,
      });
      list.records = ret.items;
      list.pagination.total = ret.total;
    },
  });

  const { pagination: listPagination } = useListPagination(list, {
    showSizeChanger: true,
    pageSizeOptions: ["20", "50"],
  });

  const drawer = useDrawer({ width: 720, placement: "right", destroyOnClose: true });

  const columns = computed<ColumnsType<ClipboardItemSummary>>(() => [
    {
      title: "类型",
      dataIndex: "contentType",
      width: 96,
      customRender: ({ record }) => (
        <span class="inline-flex items-center gap-1">
          <Icon icon={iconOfContentType(record.contentType)} />
          {labelOfContentType(record.contentType)}
        </span>
      ),
    },
    {
      title: "预览",
      dataIndex: "preview",
      ellipsis: true,
    },
    {
      title: "来源",
      dataIndex: "sourceAppName",
      width: 140,
      customRender: ({ record }) => record.sourceAppName ?? "—",
    },
    {
      title: "时间",
      dataIndex: "relativeTime",
      width: 100,
    },
    {
      title: "固定",
      dataIndex: "pinned",
      width: 72,
      customRender: ({ record }) =>
        record.pinned ? <Tag color="gold">已固定</Tag> : <Tag>—</Tag>,
    },
    {
      title: "操作",
      key: "actions",
      width: 180,
      customRender: ({ record }) => (
        <div class="flex gap-2">
          <Button type="link" size="small" onClick={() => openDetail(record)}>
            详情
          </Button>
          <Button type="link" size="small" onClick={() => togglePin(record)}>
            {record.pinned ? "取消固定" : "固定"}
          </Button>
          <Button type="link" size="small" danger onClick={() => removeItem(record)}>
            删除
          </Button>
        </div>
      ),
    },
  ]);

  function rowKey(row: ClipboardItemSummary) {
    return row.id;
  }

  function onSearch() {
    void list.onLoad({ page: 1, pageSize: list.pagination.pageSize });
  }

  async function openDetail(record: ClipboardItemSummary) {
    let detail: Awaited<ReturnType<typeof getContent>> | null = null;
    try {
      detail = await getContent(record.id);
    } catch (error) {
      message.error(getErrorMessage(error, "加载详情失败"));
      return;
    }

    drawer.create({
      title: `条目 #${record.id}`,
      slots: {
        default: () => (
          <div>
            {detail?.text ? (
              <pre class="clipboard-assistant-page__detail-text">{detail.text}</pre>
            ) : null}
            {detail?.filePaths?.length ? (
              <ul class="clipboard-assistant-page__detail-files">
                {detail.filePaths.map((path) => (
                  <li key={path}>{path}</li>
                ))}
              </ul>
            ) : null}
            {detail?.hasBlob && !detail.text ? <p>二进制内容（图片或大文本）</p> : null}
          </div>
        ),
      },
    });
  }

  async function togglePin(record: ClipboardItemSummary) {
    try {
      await mutateItems(record.pinned ? "unpin" : "pin", [record.id]);
      message.success(record.pinned ? "已取消固定" : "已固定");
      await list.onLoad();
    } catch (error) {
      message.error(getErrorMessage(error, "操作失败"));
    }
  }

  async function removeItem(record: ClipboardItemSummary) {
    modal.confirm({
      title: "删除此条目？",
      onOk: async () => {
        await mutateItems("delete", [record.id]);
        message.success("已删除");
        await list.onLoad();
      },
    });
  }

  async function clearUnpinned() {
    modal.confirm({
      title: "清空所有未固定条目？",
      onOk: async () => {
        await mutateItems("clearUnpinned");
        message.success("已清空");
        await list.onLoad();
      },
    });
  }

  onMounted(() => {
    void (async () => {
      try {
        sessionLocked.value = await isAppSessionLocked();
        const settings = await getSettings();
        featureEnabled.value = settings.enabled;
        status.value = await getStatus();
      } catch (error) {
        message.error(getErrorMessage(error, "加载剪切助手状态失败"));
      }
      if (!pageBlocked.value) {
        await list.onLoad();
      }
    })();
  });

  return {
    status,
    pageBlocked,
    sessionLocked,
    featureEnabled,
    list,
    listPagination,
    columns,
    rowKey,
    onSearch,
    clearUnpinned,
    contentTypes: CLIPBOARD_CONTENT_TYPES,
  };
}
