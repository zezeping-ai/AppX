import { App as AntApp } from "ant-design-vue";
import { computed, createVNode, onMounted, ref } from "vue";
import { useDrawer, useList, useListPagination } from "@/hooks";
import {
  CLIPBOARD_CONTENT_TYPES,
  getContent,
  getSettings,
  getStatus,
  labelOfContentType,
  listItems,
  mutateItems,
  type ClipboardAssistantStatus,
  type ClipboardItemSummary,
  type ContentType,
} from "@/modules/clipboardAssistant";
import { isAppSessionLocked } from "@/modules/appLock";
import { getErrorMessage } from "@/shared/error";
import HistoryDetailPanel from "@/pages/clipboard-assistant/components/HistoryDetailPanel.vue";

export function useClipboardHistory() {
  const { message, modal } = AntApp.useApp();
  const status = ref<ClipboardAssistantStatus | null>(null);
  const featureEnabled = ref(true);
  const showSourceAppIcon = ref(true);
  const sessionLocked = ref(false);
  const pageBlocked = computed(
    () => sessionLocked.value || !featureEnabled.value,
  );

  const list = useList<ClipboardItemSummary>({
    query: {
      keyword: "",
      contentType: undefined as ContentType | undefined,
    },
    pagination: { pageSize: 48 },
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
    pageSizeOptions: ["48", "100", "200"],
  });

  const drawer = useDrawer({ width: 720, placement: "right", destroyOnClose: true });

  async function refreshStatus() {
    status.value = await getStatus();
  }

  async function reloadList() {
    await list.onLoad();
    await refreshStatus().catch(() => undefined);
  }

  function onSearch() {
    void list.onLoad({ page: 1, pageSize: list.pagination.pageSize });
  }

  async function togglePin(record: ClipboardItemSummary): Promise<boolean> {
    try {
      await mutateItems(record.pinned ? "unpin" : "pin", [record.id]);
      message.success(record.pinned ? "已取消固定" : "已固定");
      await reloadList();
      return true;
    } catch (error) {
      message.error(getErrorMessage(error, "操作失败"));
      return false;
    }
  }

  function removeItem(record: ClipboardItemSummary, options?: { onDone?: () => void }) {
    modal.confirm({
      title: "删除此条目？",
      onOk: async () => {
        await mutateItems("delete", [record.id]);
        message.success("已删除");
        options?.onDone?.();
        await reloadList();
      },
    });
  }

  async function openDetail(record: ClipboardItemSummary) {
    let detail: Awaited<ReturnType<typeof getContent>> | null = null;
    try {
      detail = await getContent(record.id);
    } catch (error) {
      message.error(getErrorMessage(error, "加载详情失败"));
      return;
    }

    const title = `${labelOfContentType(record.contentType)} · ${record.sourceAppName || "未知来源"}`;

    const host = drawer.create({
      title,
      width: 720,
      slots: {
        default: () =>
          createVNode(HistoryDetailPanel, {
            item: record,
            detail,
            showSourceAppIcon: showSourceAppIcon.value,
            onPin: async () => {
              if (await togglePin(record)) host.close();
            },
            onDelete: () => {
              removeItem(record, { onDone: () => host.close() });
            },
          }),
      },
    });
  }

  function clearUnpinned() {
    modal.confirm({
      title: "清空所有未固定条目？",
      onOk: async () => {
        await mutateItems("clearUnpinned");
        message.success("已清空");
        await reloadList();
      },
    });
  }

  onMounted(() => {
    void (async () => {
      try {
        sessionLocked.value = await isAppSessionLocked();
        const settings = await getSettings();
        featureEnabled.value = settings.enabled;
        showSourceAppIcon.value = settings.showSourceAppIcon;
        await refreshStatus();
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
    showSourceAppIcon,
    list,
    listPagination,
    onSearch,
    openDetail,
    togglePin,
    removeItem,
    clearUnpinned,
    contentTypes: CLIPBOARD_CONTENT_TYPES,
  };
}
