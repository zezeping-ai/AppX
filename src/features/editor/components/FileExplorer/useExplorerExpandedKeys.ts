import { useLocalStorage } from "@vueuse/core";
import { computed, ref, watch, type Ref } from "vue";
import type { ExplorerTreeItem } from "@/features/editor/components/FileExplorer/normalizeTree";

const STORAGE_KEY = "appx.explorer.expanded";

export function collectDirectoryPaths(nodes: ExplorerTreeItem[]): Set<string> {
  const paths = new Set<string>();
  const walk = (items: ExplorerTreeItem[]) => {
    for (const node of items) {
      if (node.kind !== "directory") {
        continue;
      }
      paths.add(node.path);
      if (node.children?.length) {
        walk(node.children);
      }
    }
  };
  walk(nodes);
  return paths;
}

/** 按工作区根目录持久化用户手动展开状态；程序性展开仅当前会话生效 */
export function useExplorerExpandedKeys(workspaceRoot: Ref<string | null | undefined>) {
  const store = useLocalStorage<Record<string, string[]>>(STORAGE_KEY, {});
  const manualExpandedKeys = ref<string[]>([]);
  const sessionExpandedKeys = ref<string[]>([]);
  let syncingFromStore = false;

  const expandedKeys = computed(() => [
    ...new Set([...manualExpandedKeys.value, ...sessionExpandedKeys.value]),
  ]);

  function loadForRoot(root: string | null | undefined) {
    syncingFromStore = true;
    manualExpandedKeys.value = root ? [...(store.value[root] ?? [])] : [];
    sessionExpandedKeys.value = [];
    syncingFromStore = false;
  }

  watch(workspaceRoot, loadForRoot, { immediate: true });

  watch(
    manualExpandedKeys,
    (keys) => {
      if (syncingFromStore) {
        return;
      }
      const root = workspaceRoot.value;
      if (!root) {
        return;
      }
      store.value = { ...store.value, [root]: keys };
    },
    { deep: true },
  );

  function toggleExpanded(path: string) {
    const manual = new Set(manualExpandedKeys.value);
    const session = new Set(sessionExpandedKeys.value);
    const expanded = manual.has(path) || session.has(path);

    session.delete(path);
    if (expanded) {
      manual.delete(path);
    } else {
      manual.add(path);
    }

    sessionExpandedKeys.value = [...session];
    manualExpandedKeys.value = [...manual];
  }

  /** 打开文件、新建等触发的展开，不写入缓存 */
  function expandForSession(paths: string[]) {
    sessionExpandedKeys.value = [...new Set([...sessionExpandedKeys.value, ...paths])];
  }

  function collapseAll() {
    manualExpandedKeys.value = [];
    sessionExpandedKeys.value = [];
  }

  function pruneExpandedKeys(validPaths: Set<string>) {
    const prune = (keys: string[]) => keys.filter((key) => validPaths.has(key));
    const nextManual = prune(manualExpandedKeys.value);
    const nextSession = prune(sessionExpandedKeys.value);

    if (
      nextManual.length !== manualExpandedKeys.value.length ||
      nextSession.length !== sessionExpandedKeys.value.length
    ) {
      manualExpandedKeys.value = nextManual;
      sessionExpandedKeys.value = nextSession;
    }
  }

  return {
    expandedKeys,
    toggleExpanded,
    expandForSession,
    collapseAll,
    pruneExpandedKeys,
  };
}
