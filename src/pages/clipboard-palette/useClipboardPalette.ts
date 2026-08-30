import { useDebounceFn } from "@vueuse/core";
import { Modal } from "ant-design-vue";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { computed, ref, watch } from "vue";
import {
  applyItem,
  getContent,
  getSettings,
  hidePalette,
  listItems,
  mutateItems,
  type ApplyAction,
  type ApplyFormat,
  type ClipboardItemSummary,
  normalizePaletteLayout,
  type PaletteLayout,
} from "@/modules/clipboardAssistant";
import { getErrorMessage } from "@/shared/error";

export type PaletteSourceTab = {
  bundle: string;
  name: string;
  iconUrl?: string;
  count: number;
};

function buildSourceTabs(list: ClipboardItemSummary[]): PaletteSourceTab[] {
  const map = new Map<string, PaletteSourceTab>();
  for (const item of list) {
    if (!item.sourceAppBundle) continue;
    const prev = map.get(item.sourceAppBundle);
    map.set(item.sourceAppBundle, {
      bundle: item.sourceAppBundle,
      name: item.sourceAppName ?? item.sourceAppBundle,
      iconUrl: item.sourceAppIconUrl,
      count: (prev?.count ?? 0) + 1,
    });
  }
  return [...map.values()].sort((a, b) => b.count - a.count);
}

export function useClipboardPalette() {
  const keyword = ref("");
  const selectedSource = ref<string | undefined>(undefined);
  const sourceCatalog = ref<PaletteSourceTab[]>([]);
  const loading = ref(true);
  const items = ref<ClipboardItemSummary[]>([]);
  const activeIndex = ref(0);
  const actionMode = ref<ApplyAction>("paste");
  const maxItems = ref(80);
  const paletteLayout = ref<PaletteLayout>("bottomPanel");
  const showSourceAppIcon = ref(true);
  const rememberWindowPosition = ref(true);
  const autoHideOnClickOutside = ref(true);
  const openSearchOnShow = ref(true);

  // 搜索由 Rust 端 FTS + SQL 完成，items 即为当前列表

  const sourceTabs = computed(() => sourceCatalog.value);

  watch([items, selectedSource], () => {
    activeIndex.value = 0;
  });

  async function refreshSourceCatalog() {
    try {
      const result = await listItems({
        limit: maxItems.value,
        preferCache: true,
        keyword: keyword.value.trim() || undefined,
      });
      sourceCatalog.value = buildSourceTabs(result.items);
    } catch {
      // 分组 Tab 失败时不阻塞列表
    }
  }

  async function loadItems(preferCache = true) {
    loading.value = true;
    try {
      const result = await listItems({
        limit: maxItems.value,
        preferCache,
        keyword: keyword.value.trim() || undefined,
        sourceAppBundle: selectedSource.value,
      });
      items.value = result.items;
      activeIndex.value = 0;
      if (!selectedSource.value) {
        sourceCatalog.value = buildSourceTabs(result.items);
      } else if (!sourceCatalog.value.length) {
        await refreshSourceCatalog();
      }
    } catch (error) {
      console.error(getErrorMessage(error, "加载剪切历史失败"));
    } finally {
      loading.value = false;
    }
  }

  const debouncedSearch = useDebounceFn(() => {
    void loadItems(false);
  }, 200);

  watch(keyword, () => {
    if (keyword.value.trim()) {
      void debouncedSearch();
    } else {
      void loadItems(true);
    }
  });

  function resolveFormat(format?: ApplyFormat): ApplyFormat {
    return format ?? "plain";
  }

  async function runAction(
    item: ClipboardItemSummary | undefined,
    action: ApplyAction,
    format?: ApplyFormat,
  ) {
    if (!item) return;
    try {
      // 音效由 Rust apply / ingest 播放
      await applyItem(item.id, action, resolveFormat(format));
    } catch (error) {
      console.error(getErrorMessage(error, action === "copy" ? "复制失败" : "粘贴失败"));
    }
  }

  async function togglePin(item: ClipboardItemSummary) {
    try {
      await mutateItems(item.pinned ? "unpin" : "pin", [item.id]);
      await loadItems(true);
    } catch (error) {
      console.error(getErrorMessage(error, "更新固定状态失败"));
    }
  }

  async function deleteItem(item: ClipboardItemSummary) {
    try {
      await mutateItems("delete", [item.id]);
      await loadItems(true);
    } catch (error) {
      console.error(getErrorMessage(error, "删除失败"));
    }
  }

  /** 与代码段面板一致：删除前二次确认 */
  function confirmDeleteItem(item: ClipboardItemSummary) {
    const preview = item.preview.trim().slice(0, 40);
    Modal.confirm({
      title: "删除此条目？",
      content: preview ? `确认删除「${preview}${item.preview.trim().length > 40 ? "…" : ""}」？` : undefined,
      okText: "删除",
      okButtonProps: { danger: true },
      cancelText: "取消",
      onOk: () => deleteItem(item),
    });
  }

  async function openLink(item: ClipboardItemSummary) {
    try {
      const content = await getContent(item.id);
      const url = (content.text ?? item.preview).trim();
      if (!url) return;
      await openUrl(url);
      await hidePalette();
    } catch (error) {
      console.error(getErrorMessage(error, "打开链接失败"));
    }
  }

  async function revealFile(item: ClipboardItemSummary) {
    try {
      const content = await getContent(item.id);
      const path = content.filePaths?.[0];
      if (!path) {
        console.error("无文件路径可显示");
        return;
      }
      await revealItemInDir(path);
      await hidePalette();
    } catch (error) {
      console.error(getErrorMessage(error, "在文件夹中显示失败"));
    }
  }

  async function selectSource(bundle: string | undefined) {
    selectedSource.value = bundle;
    await loadItems(!keyword.value.trim());
  }

  async function bootstrapPalette() {
    try {
      const settings = await getSettings();
      maxItems.value = settings.paletteMaxItems || 80;
      paletteLayout.value = normalizePaletteLayout(settings.paletteLayout);
      showSourceAppIcon.value = settings.showSourceAppIcon;
      rememberWindowPosition.value = settings.rememberWindowPosition;
      autoHideOnClickOutside.value = settings.autoHideOnClickOutside;
      openSearchOnShow.value = settings.openSearchOnShow;
    } catch {
      // 使用默认配置
    }
    await loadItems(true);
  }

  return {
    keyword,
    selectedSource,
    sourceTabs,
    loading,
    items,
    activeIndex,
    actionMode,
    paletteLayout,
    showSourceAppIcon,
    rememberWindowPosition,
    autoHideOnClickOutside,
    openSearchOnShow,
    loadItems,
    selectSource,
    runAction,
    togglePin,
    deleteItem,
    confirmDeleteItem,
    openLink,
    revealFile,
    bootstrapPalette,
    hidePalette,
  };
}
