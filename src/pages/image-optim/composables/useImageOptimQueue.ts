import { computed, onUnmounted, ref, watch } from "vue";
import { Modal, message } from "ant-design-vue";
import {
  expandPaths,
  optimizeFile,
  pickFiles,
  pickFolder,
  useImageOptimSettings,
  type OptimizeResult,
  type OptimizeStatus,
} from "@/modules/imageOptim";
import { fileNameFromPath } from "@/pages/image-optim/format";
import { getErrorMessage } from "@/shared/error";

export type QueueItemStatus = "queued" | "running" | OptimizeStatus;

export type QueueItem = {
  id: string;
  path: string;
  name: string;
  beforeBytes: number;
  afterBytes: number;
  savedBytes: number;
  status: QueueItemStatus;
  error: string | null;
};

const CONCURRENCY = 2;

function toQueueItem(path: string): QueueItem {
  return {
    id: `${path}::${Date.now()}::${Math.random().toString(36).slice(2, 8)}`,
    path,
    name: fileNameFromPath(path),
    beforeBytes: 0,
    afterBytes: 0,
    savedBytes: 0,
    status: "queued",
    error: null,
  };
}

export function useImageOptimQueue() {
  const { settings, loaded, refresh, patch } = useImageOptimSettings();
  const items = ref<QueueItem[]>([]);
  let activeWorkers = 0;
  let pumpScheduled = false;
  let disposed = false;
  /** 已确认过的覆盖模式；切换有损开关后需重新确认 */
  let confirmedLossyMode: boolean | null = null;

  const running = computed(
    () => items.value.some((i) => i.status === "running") || activeWorkers > 0,
  );

  const totals = computed(() => {
    const optimized = items.value.filter(
      (i) => i.status === "optimized" || i.status === "alreadyOptimal",
    );
    const before = optimized.reduce((s, i) => s + i.beforeBytes, 0);
    const after = optimized.reduce((s, i) => s + i.afterBytes, 0);
    const saved = optimized.reduce((s, i) => s + Math.max(0, i.savedBytes), 0);
    const done = items.value.filter(
      (i) =>
        i.status === "optimized" ||
        i.status === "alreadyOptimal" ||
        i.status === "error",
    ).length;
    return {
      count: items.value.length,
      done,
      pending: items.value.filter((i) => i.status === "queued" || i.status === "running").length,
      before,
      after,
      saved,
    };
  });

  watch(
    () => settings.value.lossyEnabled,
    () => {
      confirmedLossyMode = null;
    },
  );

  async function setLossyEnabled(checked: boolean) {
    try {
      await patch({ lossyEnabled: checked });
    } catch (error) {
      message.error(getErrorMessage(error, "保存有损开关失败"));
      await refresh();
    }
  }

  function enqueuePaths(paths: string[]) {
    const busy = new Set(
      items.value
        .filter((i) => i.status === "queued" || i.status === "running")
        .map((i) => i.path),
    );
    const next = paths.filter((p) => !busy.has(p)).map(toQueueItem);
    if (!next.length) {
      return;
    }
    items.value = [...items.value, ...next];
    schedulePump();
  }

  async function ensureOverwriteConsent(): Promise<boolean> {
    const lossy = settings.value.lossyEnabled;
    if (confirmedLossyMode === lossy) {
      return true;
    }
    return new Promise((resolve) => {
      Modal.confirm({
        title: "确认优化图片？",
        content: lossy
          ? "将原地覆盖原文件（已开启有损压缩，画质可能下降）。无法自动撤销。"
          : "将原地覆盖原文件（默认无损）。仅当体积更小时才会写入。无法自动撤销。",
        okText: "开始优化",
        cancelText: "取消",
        onOk: () => {
          confirmedLossyMode = lossy;
          resolve(true);
        },
        onCancel: () => resolve(false),
      });
    });
  }

  async function addDroppedOrPicked(rawPaths: string[]) {
    if (!rawPaths.length || disposed) {
      return;
    }
    try {
      const expanded = await expandPaths(rawPaths);
      if (!expanded.length) {
        message.info("未找到可优化的图片（png / jpg / jpeg / gif / svg）");
        return;
      }
      const ok = await ensureOverwriteConsent();
      if (!ok || disposed) {
        return;
      }
      enqueuePaths(expanded);
    } catch (error) {
      message.error(getErrorMessage(error, "展开路径失败"));
    }
  }

  async function onPickFiles() {
    try {
      const paths = await pickFiles();
      await addDroppedOrPicked(paths);
    } catch (error) {
      message.error(getErrorMessage(error, "选择文件失败"));
    }
  }

  async function onPickFolder() {
    try {
      const folder = await pickFolder();
      if (folder) {
        await addDroppedOrPicked([folder]);
      }
    } catch (error) {
      message.error(getErrorMessage(error, "选择文件夹失败"));
    }
  }

  function removeItem(id: string) {
    const target = items.value.find((i) => i.id === id);
    if (target?.status === "running") {
      return;
    }
    items.value = items.value.filter((i) => i.id !== id);
  }

  function clearFinished() {
    items.value = items.value.filter(
      (i) => i.status === "queued" || i.status === "running",
    );
  }

  function clearAll() {
    Modal.confirm({
      title: "清空列表？",
      content: "进行中的任务仍会写完当前文件，排队中的任务将被取消。",
      okText: "清空",
      okType: "danger",
      cancelText: "取消",
      onOk: () => {
        items.value = items.value.filter((i) => i.status === "running");
      },
    });
  }

  function stopQueued() {
    items.value = items.value.filter((i) => i.status !== "queued");
  }

  function schedulePump() {
    if (disposed || pumpScheduled) {
      return;
    }
    pumpScheduled = true;
    queueMicrotask(() => {
      pumpScheduled = false;
      void pump();
    });
  }

  function patchItem(id: string, patchFields: Partial<QueueItem>) {
    if (disposed) {
      return;
    }
    items.value = items.value.map((i) => (i.id === id ? { ...i, ...patchFields } : i));
  }

  async function pump() {
    if (disposed) {
      return;
    }
    while (activeWorkers < CONCURRENCY) {
      const next = items.value.find((i) => i.status === "queued");
      if (!next) {
        break;
      }
      const { id, path } = next;
      patchItem(id, { status: "running" });
      activeWorkers += 1;
      void (async () => {
        try {
          const result: OptimizeResult = await optimizeFile({ path });
          if (disposed) {
            return;
          }
          patchItem(id, {
            beforeBytes: result.beforeBytes,
            afterBytes: result.afterBytes,
            savedBytes: result.savedBytes,
            status: result.status,
            error: result.error,
          });
        } catch (error) {
          patchItem(id, {
            status: "error",
            error: getErrorMessage(error, "优化失败"),
          });
        } finally {
          activeWorkers -= 1;
          if (!disposed) {
            schedulePump();
          }
        }
      })();
    }
  }

  onUnmounted(() => {
    disposed = true;
    items.value = items.value.filter((i) => i.status === "running");
  });

  return {
    items,
    settings,
    settingsLoaded: loaded,
    running,
    totals,
    setLossyEnabled,
    addDroppedOrPicked,
    onPickFiles,
    onPickFolder,
    removeItem,
    clearAll,
    clearFinished,
    stopQueued,
  };
}
