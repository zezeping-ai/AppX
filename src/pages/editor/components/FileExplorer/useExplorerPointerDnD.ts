import { onUnmounted, ref, type Ref } from "vue";
import type { ExplorerTreeItem } from "@/pages/editor/components/FileExplorer/normalizeTree";
import {
  canDropIntoDirectory,
  parentDirectory,
} from "@/pages/editor/components/FileExplorer/treeHelpers";

const DRAG_THRESHOLD_PX = 3;
const EXPAND_ON_DRAG_MS = 500;
const SCROLL_EDGE_PX = 36;
const SCROLL_SPEED_PX = 14;

export type ExplorerDropPlacement = "before" | "after" | "inside";

/** VS Code 风格落点：插入线 / 目录高亮 + 实际目标目录 */
export type ExplorerDropIndicator = {
  anchorPath: string;
  placement: ExplorerDropPlacement;
  targetDirectory: string;
};

export type ExplorerDragGhost = {
  title: string;
  icon: string;
  iconColor?: string;
  x: number;
  y: number;
  valid: boolean;
};

type DragSession = {
  path: string;
  title: string;
  icon: string;
  iconColor?: string;
  startX: number;
  startY: number;
  pointerId: number;
  activated: boolean;
};

/**
 * VS Code 风格文件树拖放（pointer 实现）。
 * 成熟库（antd Tree / he-tree）底层多为 HTML5 DnD，在 Tauri WKWebView 上不稳定。
 */
export function useExplorerPointerDnD(options: {
  workspaceRoot: Ref<string | null | undefined>;
  scrollEl: Ref<HTMLElement | null | undefined>;
  isDirectoryExpanded: (path: string) => boolean;
  expandDirectory: (path: string) => void | Promise<void>;
  onMove: (sourcePath: string, targetDirectory: string) => void;
}) {
  const draggingPath = ref<string | null>(null);
  const dropIndicator = ref<ExplorerDropIndicator | null>(null);
  const ghost = ref<ExplorerDragGhost | null>(null);

  let session: DragSession | null = null;
  let expandTimer: ReturnType<typeof setTimeout> | null = null;
  let expandPath: string | null = null;
  let scrollRaf = 0;
  let lastClientX = 0;
  let lastClientY = 0;
  let suppressClickPath: string | null = null;

  function clearExpandTimer() {
    if (expandTimer) {
      clearTimeout(expandTimer);
      expandTimer = null;
    }
    expandPath = null;
  }

  function stopAutoScroll() {
    if (scrollRaf) {
      cancelAnimationFrame(scrollRaf);
      scrollRaf = 0;
    }
  }

  function resetDragVisual() {
    draggingPath.value = null;
    dropIndicator.value = null;
    ghost.value = null;
    document.body.style.removeProperty("cursor");
    clearExpandTimer();
    stopAutoScroll();
  }

  function detachWindowListeners() {
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
    window.removeEventListener("pointercancel", onPointerUp);
  }

  function endSession(opts?: { suppressClick?: boolean }) {
    if (opts?.suppressClick && session) {
      suppressClickPath = session.path;
    }
    session = null;
    detachWindowListeners();
    resetDragVisual();
  }

  function scheduleExpand(directoryPath: string) {
    if (expandPath === directoryPath) {
      return;
    }
    clearExpandTimer();
    if (
      !directoryPath ||
      directoryPath === options.workspaceRoot.value ||
      options.isDirectoryExpanded(directoryPath)
    ) {
      return;
    }
    expandPath = directoryPath;
    expandTimer = setTimeout(() => {
      expandTimer = null;
      expandPath = null;
      void options.expandDirectory(directoryPath);
    }, EXPAND_ON_DRAG_MS);
  }

  /**
   * VS Code 命中区：
   * - 目录上/下 25% → 插入线（移入其父目录）
   * - 目录中间 → 拖入该目录（高亮）
   * - 文件上/下半 → 插入线（移入其父目录）
   */
  function resolveDropFromRow(
    source: string,
    row: HTMLElement,
    clientY: number,
    root: string,
  ): ExplorerDropIndicator | null {
    const path = row.dataset.explorerPath;
    const kind = row.dataset.explorerKind as ExplorerTreeItem["kind"] | undefined;
    if (!path || !kind || path === source) {
      return null;
    }

    const rect = row.getBoundingClientRect();
    const ratio = rect.height > 0 ? (clientY - rect.top) / rect.height : 0.5;

    let placement: ExplorerDropPlacement;
    let targetDirectory: string | null;

    if (kind === "directory") {
      if (ratio < 0.25) {
        placement = "before";
        targetDirectory = parentDirectory(path, root);
      } else if (ratio > 0.75) {
        placement = "after";
        targetDirectory = parentDirectory(path, root);
      } else {
        placement = "inside";
        targetDirectory = path;
      }
    } else if (ratio < 0.5) {
      placement = "before";
      targetDirectory = parentDirectory(path, root);
    } else {
      placement = "after";
      targetDirectory = parentDirectory(path, root);
    }

    if (!targetDirectory || !canDropIntoDirectory(source, targetDirectory)) {
      return null;
    }
    return { anchorPath: path, placement, targetDirectory };
  }

  function resolveDropFromPoint(
    clientX: number,
    clientY: number,
  ): ExplorerDropIndicator | null {
    const source = session?.path ?? draggingPath.value;
    const root = options.workspaceRoot.value;
    if (!source || !root) {
      return null;
    }

    const el = document.elementFromPoint(clientX, clientY);
    const row = el?.closest<HTMLElement>("[data-explorer-path]");
    if (row) {
      return resolveDropFromRow(source, row, clientY, root);
    }

    if (el?.closest("[data-explorer-tree]") && canDropIntoDirectory(source, root)) {
      return {
        anchorPath: root,
        placement: "inside",
        targetDirectory: root,
      };
    }
    return null;
  }

  function tickAutoScroll() {
    scrollRaf = 0;
    if (!session?.activated) {
      return;
    }
    const scroller = options.scrollEl.value;
    if (!scroller) {
      return;
    }
    const rect = scroller.getBoundingClientRect();
    let delta = 0;
    if (lastClientY < rect.top + SCROLL_EDGE_PX) {
      delta = -SCROLL_SPEED_PX;
    } else if (lastClientY > rect.bottom - SCROLL_EDGE_PX) {
      delta = SCROLL_SPEED_PX;
    }
    if (delta !== 0) {
      scroller.scrollTop += delta;
      dropIndicator.value = resolveDropFromPoint(lastClientX, lastClientY);
      scrollRaf = requestAnimationFrame(tickAutoScroll);
    }
  }

  function ensureAutoScroll(clientX: number, clientY: number) {
    lastClientX = clientX;
    lastClientY = clientY;
    const scroller = options.scrollEl.value;
    if (!scroller) {
      return;
    }
    const rect = scroller.getBoundingClientRect();
    const nearEdge =
      clientY < rect.top + SCROLL_EDGE_PX || clientY > rect.bottom - SCROLL_EDGE_PX;
    if (nearEdge && !scrollRaf) {
      scrollRaf = requestAnimationFrame(tickAutoScroll);
    } else if (!nearEdge) {
      stopAutoScroll();
    }
  }

  function onPointerMove(event: PointerEvent) {
    if (!session || event.pointerId !== session.pointerId) {
      return;
    }
    if (!session.activated) {
      const dx = event.clientX - session.startX;
      const dy = event.clientY - session.startY;
      if (dx * dx + dy * dy < DRAG_THRESHOLD_PX * DRAG_THRESHOLD_PX) {
        return;
      }
      session.activated = true;
      draggingPath.value = session.path;
      document.body.style.cursor = "grabbing";
    }

    event.preventDefault();
    const indicator = resolveDropFromPoint(event.clientX, event.clientY);
    dropIndicator.value = indicator;
    ghost.value = {
      title: session.title,
      icon: session.icon,
      iconColor: session.iconColor,
      x: event.clientX,
      y: event.clientY,
      valid: Boolean(indicator),
    };

    if (indicator?.placement === "inside") {
      scheduleExpand(indicator.targetDirectory);
    } else {
      clearExpandTimer();
    }
    ensureAutoScroll(event.clientX, event.clientY);
  }

  function onPointerUp(event: PointerEvent) {
    if (!session || event.pointerId !== session.pointerId) {
      return;
    }
    const source = session.path;
    const activated = session.activated;
    const indicator = activated
      ? resolveDropFromPoint(event.clientX, event.clientY)
      : null;
    endSession({ suppressClick: activated });
    if (
      activated &&
      indicator &&
      canDropIntoDirectory(source, indicator.targetDirectory)
    ) {
      options.onMove(source, indicator.targetDirectory);
    }
  }

  function onRowPointerDown(
    event: PointerEvent,
    node: Pick<ExplorerTreeItem, "path" | "title"> & {
      icon: string;
      iconColor?: string;
    },
  ) {
    if (event.button !== 0 || event.ctrlKey || event.metaKey || event.altKey) {
      return;
    }
    if ((event.target as HTMLElement | null)?.closest(".explorer-chevron")) {
      return;
    }
    if (session) {
      endSession();
    }
    session = {
      path: node.path,
      title: node.title,
      icon: node.icon,
      iconColor: node.iconColor,
      startX: event.clientX,
      startY: event.clientY,
      pointerId: event.pointerId,
      activated: false,
    };
    window.addEventListener("pointermove", onPointerMove, { passive: false });
    window.addEventListener("pointerup", onPointerUp);
    window.addEventListener("pointercancel", onPointerUp);
  }

  function consumeClickSuppression(path: string): boolean {
    if (suppressClickPath === path) {
      suppressClickPath = null;
      return true;
    }
    return false;
  }

  onUnmounted(() => {
    endSession();
  });

  return {
    draggingPath,
    dropIndicator,
    ghost,
    onRowPointerDown,
    consumeClickSuppression,
  };
}
