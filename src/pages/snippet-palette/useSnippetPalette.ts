import { computed, ref, watch } from "vue";
import { Modal } from "ant-design-vue";
import { CodeSnippetRecord } from "@/models";
import {
  CODE_SNIPPET_GROUPS,
  copyPaletteItem,
  hidePalette,
  insertPaletteItem,
  listPaletteItems,
  getCodeSnippetSettings,
  setInlineExpansionTrigger,
  syncAllSnippetsToRuntime,
  type CodeSnippetGroup,
  type CodeSnippetPaletteItem,
  labelOfSnippetGroup,
} from "@/modules/codeSnippets";
import { getErrorMessage } from "@/shared/error";
import { isAppSessionLocked } from "@/modules/appLock";

export type SnippetPaletteAction = "insert" | "copy";

export function useSnippetPalette() {
  const keyword = ref("");
  const selectedGroup = ref<CodeSnippetGroup | undefined>(undefined);
  const loading = ref(true);
  const items = ref<CodeSnippetPaletteItem[]>([]);
  const activeIndex = ref(0);
  const actionMode = ref<SnippetPaletteAction>("insert");

  const groupFilters = computed(() => {
    const counts = new Map<CodeSnippetGroup, number>();
    for (const item of items.value) {
      counts.set(item.group, (counts.get(item.group) ?? 0) + 1);
    }
    return CODE_SNIPPET_GROUPS.filter((group) => counts.has(group.value)).map((group) => ({
      ...group,
      count: counts.get(group.value) ?? 0,
    }));
  });

  const filtered = computed(() => {
    const q = keyword.value.trim().toLowerCase();
    return items.value.filter((item) => {
      if (selectedGroup.value && item.group !== selectedGroup.value) return false;
      if (!q) return true;
      return (
        item.name.toLowerCase().includes(q) ||
        item.abbreviation.toLowerCase().includes(q) ||
        labelOfSnippetGroup(item.group).toLowerCase().includes(q)
      );
    });
  });

  watch([filtered, selectedGroup], () => {
    activeIndex.value = 0;
  });

  async function ensureUnlockedOrHide(): Promise<boolean> {
    try {
      if (await isAppSessionLocked()) {
        await hidePalette();
        return false;
      }
    } catch {
      // 忽略异常，避免面板卡死
    }
    return true;
  }

  async function loadItems() {
    loading.value = true;
    try {
      items.value = await listPaletteItems();
      activeIndex.value = 0;
    } catch (error) {
      console.error(getErrorMessage(error, "加载代码段失败"));
    } finally {
      loading.value = false;
    }
  }

  function selectGroup(group: CodeSnippetGroup | undefined) {
    selectedGroup.value = group;
  }

  function toggleActionMode() {
    actionMode.value = actionMode.value === "insert" ? "copy" : "insert";
  }

  async function runAction(item: CodeSnippetPaletteItem | undefined, action: SnippetPaletteAction) {
    if (!item) return;
    try {
      if (action === "copy") {
        await copyPaletteItem(item.id);
      } else {
        await insertPaletteItem(item.id);
      }
    } catch (error) {
      console.error(getErrorMessage(error, action === "copy" ? "拷贝失败" : "插入失败"));
    }
  }

  function selectItem(item: CodeSnippetPaletteItem | undefined) {
    void runAction(item, actionMode.value);
  }

  function onItemAction(event: MouseEvent, item: CodeSnippetPaletteItem, action: SnippetPaletteAction) {
    event.stopPropagation();
    void runAction(item, action);
  }

  function removeSnippet(item: CodeSnippetPaletteItem) {
    Modal.confirm({
      title: "删除代码段",
      content: `确认删除「${item.name}」？`,
      okText: "删除",
      okButtonProps: { danger: true },
      cancelText: "取消",
      onOk: async () => {
        try {
          await CodeSnippetRecord.destroy(item.id);
          await syncAllSnippetsToRuntime();
          await loadItems();
        } catch (error) {
          console.error(getErrorMessage(error, "删除失败"));
        }
      },
    });
  }

  async function resetForOpen() {
    if (!(await ensureUnlockedOrHide())) return false;
    keyword.value = "";
    selectedGroup.value = undefined;
    activeIndex.value = 0;
    actionMode.value = "insert";
    await loadItems();
    return true;
  }

  async function bootstrap() {
    if (!(await ensureUnlockedOrHide())) return false;
    try {
      const settings = await getCodeSnippetSettings();
      setInlineExpansionTrigger(settings.inlineExpansionTrigger);
    } catch {
      // 独立窗口读取失败时保留默认触发键
    }
    await loadItems();
    return true;
  }

  return {
    keyword,
    selectedGroup,
    loading,
    items,
    filtered,
    groupFilters,
    activeIndex,
    actionMode,
    loadItems,
    selectGroup,
    toggleActionMode,
    runAction,
    selectItem,
    onItemAction,
    removeSnippet,
    resetForOpen,
    bootstrap,
    hidePalette,
  };
}
