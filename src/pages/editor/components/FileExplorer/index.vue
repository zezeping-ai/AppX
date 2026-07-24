<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { computed, nextTick, ref, toRef, watch } from "vue";
import ExplorerInlineInput from "@/pages/editor/components/FileExplorer/ExplorerInlineInput/index.vue";
import ExplorerNode from "@/pages/editor/components/FileExplorer/ExplorerNode/index.vue";
import type { ExplorerTreeItem } from "@/pages/editor/components/FileExplorer/normalizeTree";
import type { ContextMenuAction, InlineEditState } from "@/pages/editor/components/FileExplorer/types";
import { useExplorerExpandedKeys } from "@/pages/editor/components/FileExplorer/useExplorerExpandedKeys";
import {
  findNode,
  isDirectoryUnloaded,
} from "@/pages/editor/components/FileExplorer/treeHelpers";
import {
  DEFAULT_AXDOC_FILE_NAME,
  DEFAULT_TEXT_FILE_NAME,
} from "@/modules/editor/axdoc";

const props = defineProps<{
  treeData: ExplorerTreeItem[];
  workspaceRoot?: string | null;
  activePath?: string | null;
  loading?: boolean;
  /** 按需加载某目录子节点（懒加载） */
  loadChildren?: (path: string) => Promise<void>;
}>();

const emit = defineEmits<{
  open: [path: string];
  refresh: [];
  createFile: [directory: string, fileName: string];
  createFolder: [directory: string, folderName: string];
  rename: [path: string, newName: string];
  delete: [path: string];
  convertToEncrypted: [path: string];
  convertToCustomEncrypted: [path: string];
  convertCustomToDefaultEncrypted: [path: string];
  convertToPlain: [path: string];
}>();

const VIEWPORT_PADDING = 8;
const DEFAULT_FOLDER_NAME = "新建文件夹";

const CREATE_CONTEXT_ACTIONS: ContextMenuAction[] = [
  {
    key: "new-file",
    label: "新建文本",
    hint: ".txt.x",
    icon: "mdi:file-code-outline",
  },
  {
    key: "new-doc",
    label: "新建富文本",
    hint: ".axdoc.x",
    icon: "mdi:file-document-edit-outline",
  },
  { key: "new-folder", label: "新建文件夹", icon: "mdi:folder-plus-outline" },
];

const { expandedKeys, toggleExpanded, expandForSession, collapseAll, pruneExpandedKeys } =
  useExplorerExpandedKeys(toRef(props, "workspaceRoot"));
const inlineEdit = ref<InlineEditState | null>(null);
const contextMenuRef = ref<HTMLElement | null>(null);
const contextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  node: null as ExplorerTreeItem | null,
  rootTarget: false,
});

function findAncestorKeys(
  nodes: ExplorerTreeItem[],
  targetPath: string,
  ancestors: string[] = [],
): string[] | null {
  for (const node of nodes) {
    if (node.path === targetPath) {
      return ancestors;
    }
    if (node.children) {
      const nextAncestors =
        node.kind === "directory" ? [...ancestors, node.path] : ancestors;
      const matched = findAncestorKeys(node.children, targetPath, nextAncestors);
      if (matched) {
        return matched;
      }
    }
  }
  return null;
}

function depthOfPath(path: string): number {
  return path.split(/[/\\]/).filter(Boolean).length;
}

async function ensureChildrenLoaded(path: string) {
  if (!props.loadChildren || path === props.workspaceRoot) {
    return;
  }
  const node = findNode(props.treeData, path);
  if (node && isDirectoryUnloaded(node)) {
    await props.loadChildren(path);
  }
}

function resolveParentDirectory(node: ExplorerTreeItem | null): string | null {
  if (node?.kind === "directory") {
    return node.path;
  }
  if (node?.kind === "file") {
    const index = Math.max(node.path.lastIndexOf("/"), node.path.lastIndexOf("\\"));
    if (index > 0) {
      return node.path.slice(0, index);
    }
    return props.workspaceRoot ?? null;
  }
  return props.workspaceRoot ?? null;
}

async function toggleExpand(path: string) {
  const expanding = !expandedKeys.value.includes(path);
  if (expanding) {
    try {
      await ensureChildrenLoaded(path);
    } catch {
      return;
    }
  }
  toggleExpanded(path);
}

async function startCreateEntry(parentPath: string, defaultName: string) {
  if (!parentPath) {
    return;
  }
  try {
    await ensureChildrenLoaded(parentPath);
  } catch {
    return;
  }
  expandForSession([parentPath]);
  inlineEdit.value = {
    mode: "create-file",
    parentPath,
    targetPath: parentPath,
    value: defaultName,
  };
}

function startCreateFile(parentPath: string) {
  void startCreateEntry(parentPath, DEFAULT_TEXT_FILE_NAME);
}

function startCreateDoc(parentPath: string) {
  void startCreateEntry(parentPath, DEFAULT_AXDOC_FILE_NAME);
}

async function startCreateFolder(parentPath: string) {
  if (!parentPath) {
    return;
  }
  try {
    await ensureChildrenLoaded(parentPath);
  } catch {
    return;
  }
  expandForSession([parentPath]);
  inlineEdit.value = {
    mode: "create-folder",
    parentPath,
    targetPath: parentPath,
    value: DEFAULT_FOLDER_NAME,
  };
}

function startRename(path: string) {
  const node = findNode(props.treeData, path);
  if (!node) {
    return;
  }
  const parentPath = resolveParentDirectory(node);
  if (!parentPath) {
    return;
  }
  if (node.kind === "directory") {
    expandForSession([node.path]);
  }
  inlineEdit.value = {
    mode: "rename",
    parentPath,
    targetPath: node.path,
    value: node.title,
  };
}

function cancelInline() {
  inlineEdit.value = null;
}

function onInlineConfirm(value: string) {
  const state = inlineEdit.value;
  inlineEdit.value = null;
  if (!state) {
    return;
  }
  const trimmed = value.trim();
  if (!trimmed) {
    return;
  }
  if (state.mode === "create-file") {
    emit("createFile", state.parentPath, trimmed);
    return;
  }
  if (state.mode === "create-folder") {
    emit("createFolder", state.parentPath, trimmed);
    return;
  }
  emit("rename", state.targetPath, trimmed);
}

function onHeaderNewFile() {
  if (!props.workspaceRoot) {
    return;
  }
  const parent = resolveParentDirectory(
    props.activePath ? findNode(props.treeData, props.activePath) : null,
  );
  startCreateFile(parent ?? props.workspaceRoot);
}

function onHeaderNewDoc() {
  if (!props.workspaceRoot) {
    return;
  }
  const parent = resolveParentDirectory(
    props.activePath ? findNode(props.treeData, props.activePath) : null,
  );
  startCreateDoc(parent ?? props.workspaceRoot);
}

function onHeaderNewFolder() {
  if (!props.workspaceRoot) {
    return;
  }
  const parent = resolveParentDirectory(
    props.activePath ? findNode(props.treeData, props.activePath) : null,
  );
  startCreateFolder(parent ?? props.workspaceRoot);
}

watch(
  () => props.treeData,
  (nodes) => {
    if (!props.workspaceRoot) {
      return;
    }
    pruneExpandedKeys(nodes);
  },
  { deep: true },
);

/** 恢复持久化展开时，按深度顺序补齐未加载的子目录 */
watch(
  [expandedKeys, () => props.treeData],
  async () => {
    if (!props.loadChildren || !props.workspaceRoot) {
      return;
    }
    const sorted = [...expandedKeys.value].sort((a, b) => depthOfPath(a) - depthOfPath(b));
    for (const path of sorted) {
      try {
        await ensureChildrenLoaded(path);
      } catch {
        // 单个目录失败不影响其余展开恢复
      }
    }
  },
);

watch(
  () => props.activePath,
  (path) => {
    if (!path) {
      return;
    }
    const ancestors = findAncestorKeys(props.treeData, path);
    if (ancestors?.length) {
      expandForSession(ancestors);
    }
  },
);

const contextActions = computed<ContextMenuAction[]>(() => {
  const node = contextMenu.value.node;
  const rootTarget = contextMenu.value.rootTarget;

  if (rootTarget || !node) {
    return [
      ...CREATE_CONTEXT_ACTIONS,
      { key: "refresh", label: "刷新", icon: "mdi:refresh", divider: true },
    ];
  }

  if (node.kind === "directory") {
    return [
      ...CREATE_CONTEXT_ACTIONS,
      { key: "rename", label: "重命名", icon: "mdi:pencil-outline", divider: true },
      { key: "delete", label: "删除", icon: "mdi:delete-outline", danger: true },
    ];
  }

  const actions: ContextMenuAction[] = [
    { key: "open", label: "打开", icon: "mdi:file-eye-outline" },
    { key: "rename", label: "重命名", icon: "mdi:pencil-outline" },
  ];

  // 转换项优先级：转为普通 > 转为加密 (.x) > 独立口令加密 (.x0)
  const convertActions: ContextMenuAction[] = [];
  if (node.encrypted) {
    convertActions.push({ key: "to-plain", label: "转为普通" });
  }
  if (!node.encrypted) {
    convertActions.push({ key: "to-encrypted", label: "转为加密 (.x)" });
  } else if (node.customEncrypted) {
    convertActions.push({ key: "to-default-encrypted", label: "转为默认口令加密" });
  }
  if (!node.encrypted) {
    convertActions.push({ key: "to-custom-encrypted", label: "独立口令加密 (.x0)" });
  } else if (!node.customEncrypted) {
    convertActions.push({ key: "to-custom-encrypted", label: "转为独立口令加密 (.x0)" });
  }
  if (convertActions.length) {
    convertActions[0] = { ...convertActions[0], divider: true };
    actions.push(...convertActions);
  }

  actions.push({
    key: "delete",
    label: "删除",
    icon: "mdi:delete-outline",
    danger: true,
    divider: true,
  });
  return actions;
});

function openContextMenu(path: string, event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  const node = findNode(props.treeData, path);
  if (!node) {
    return;
  }
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    node,
    rootTarget: false,
  };
  void nextTick(adjustContextMenuPosition);
}

function openRootContextMenu(event: MouseEvent) {
  if (!props.workspaceRoot) {
    return;
  }
  event.preventDefault();
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    node: null,
    rootTarget: true,
  };
  void nextTick(adjustContextMenuPosition);
}

function adjustContextMenuPosition() {
  const menu = contextMenuRef.value;
  if (!menu) {
    return;
  }

  const { width, height } = menu.getBoundingClientRect();
  const maxX = window.innerWidth - width - VIEWPORT_PADDING;
  const maxY = window.innerHeight - height - VIEWPORT_PADDING;

  contextMenu.value = {
    ...contextMenu.value,
    x: Math.min(Math.max(VIEWPORT_PADDING, contextMenu.value.x), maxX),
    y: Math.min(Math.max(VIEWPORT_PADDING, contextMenu.value.y), maxY),
  };
}

function closeContextMenu() {
  contextMenu.value.visible = false;
}

function onContextAction(key: string) {
  const node = contextMenu.value.node;
  const rootTarget = contextMenu.value.rootTarget;
  closeContextMenu();

  const parentPath = rootTarget
    ? props.workspaceRoot
    : resolveParentDirectory(node);

  if (key === "new-file") {
    if (parentPath) startCreateFile(parentPath);
    return;
  }
  if (key === "new-doc") {
    if (parentPath) startCreateDoc(parentPath);
    return;
  }
  if (key === "new-folder") {
    if (parentPath) startCreateFolder(parentPath);
    return;
  }
  if (key === "refresh") {
    emit("refresh");
    return;
  }
  if (!node) {
    return;
  }

  if (key === "open") {
    emit("open", node.path);
    return;
  }
  if (key === "rename") {
    startRename(node.path);
    return;
  }
  if (key === "delete") {
    emit("delete", node.path);
    return;
  }
  if (key === "to-encrypted") {
    emit("convertToEncrypted", node.path);
    return;
  }
  if (key === "to-custom-encrypted") {
    emit("convertToCustomEncrypted", node.path);
    return;
  }
  if (key === "to-default-encrypted") {
    emit("convertCustomToDefaultEncrypted", node.path);
    return;
  }
  if (key === "to-plain") {
    emit("convertToPlain", node.path);
  }
}

function onOpenFile(path: string) {
  emit("open", path);
}

const showRootInline = computed(() => {
  const edit = inlineEdit.value;
  return (
    Boolean(edit) &&
    (edit!.mode === "create-file" || edit!.mode === "create-folder") &&
    edit!.parentPath === props.workspaceRoot
  );
});
</script>

<template>
  <div class="file-explorer" @click="closeContextMenu" @contextmenu.prevent>
    <div class="file-explorer__header">
      <div class="file-explorer__actions">
        <button
          type="button"
          class="file-explorer__action"
          title="新建文本 (.txt.x)"
          :disabled="!workspaceRoot"
          @click="onHeaderNewFile"
        >
          <Icon icon="mdi:file-code-outline" width="15" height="15" />
        </button>
        <button
          type="button"
          class="file-explorer__action"
          title="新建富文本 (.axdoc.x)"
          :disabled="!workspaceRoot"
          @click="onHeaderNewDoc"
        >
          <Icon icon="mdi:file-document-edit-outline" width="15" height="15" />
        </button>
        <button
          type="button"
          class="file-explorer__action"
          title="新建文件夹"
          :disabled="!workspaceRoot"
          @click="onHeaderNewFolder"
        >
          <Icon icon="mdi:folder-plus-outline" width="15" height="15" />
        </button>
        <button
          type="button"
          class="file-explorer__action"
          title="刷新"
          :disabled="!workspaceRoot"
          @click="emit('refresh')"
        >
          <Icon icon="mdi:refresh" width="15" height="15" />
        </button>
        <button
          type="button"
          class="file-explorer__action"
          title="全部折叠"
          :disabled="!workspaceRoot"
          @click="collapseAll"
        >
          <Icon icon="mdi:unfold-less-horizontal" width="15" height="15" />
        </button>
      </div>
    </div>

    <div v-if="loading" class="file-explorer__empty" />
    <div v-else-if="!workspaceRoot" class="file-explorer__empty" />
    <div
      v-else
      class="file-explorer__tree"
      @scroll="closeContextMenu"
      @contextmenu="openRootContextMenu"
    >
      <ExplorerInlineInput
        v-if="showRootInline && inlineEdit"
        :depth="0"
        :value="inlineEdit.value"
        :kind="inlineEdit.mode === 'create-folder' ? 'folder' : 'file'"
        @confirm="onInlineConfirm"
        @cancel="cancelInline"
      />
      <div
        v-if="!showRootInline && treeData.length === 0"
        class="file-explorer__empty file-explorer__empty--in-tree"
      />
      <ExplorerNode
        v-for="node in treeData"
        :key="node.path"
        :node="node"
        :depth="0"
        :expanded-keys="expandedKeys"
        :active-path="activePath"
        :inline-edit="inlineEdit"
        @toggle="toggleExpand"
        @open="onOpenFile"
        @contextmenu="openContextMenu"
        @inline-confirm="onInlineConfirm"
        @inline-cancel="cancelInline"
      />
    </div>

    <Teleport to="body">
      <div
        v-if="contextMenu.visible"
        ref="contextMenuRef"
        class="file-explorer__context-menu"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
        @click.stop
      >
        <template v-for="action in contextActions" :key="action.key">
          <div v-if="action.divider" class="file-explorer__context-divider" />
          <button
            type="button"
            class="file-explorer__context-item"
            :class="{ 'file-explorer__context-item--danger': action.danger }"
            @click="onContextAction(action.key)"
          >
            <Icon
              v-if="action.icon"
              :icon="action.icon"
              width="15"
              height="15"
              class="file-explorer__context-icon"
            />
            <span class="file-explorer__context-label">{{ action.label }}</span>
            <span v-if="action.hint" class="file-explorer__context-hint">{{ action.hint }}</span>
          </button>
        </template>
      </div>
    </Teleport>
  </div>
</template>

<style scoped lang="scss">
.file-explorer {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: #fff;
}

.file-explorer__header {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 4px 6px;
  border-bottom: 1px solid #f1f5f9;
}

.file-explorer__actions {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

.file-explorer__action {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: #6b7280;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: #f3f4f6;
    color: #111827;
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
}

.file-explorer__tree {
  flex: 1;
  overflow: auto;
  padding: 4px 0 8px;
}

.file-explorer__empty {
  flex: 1;
  min-height: 0;

  &--in-tree {
    display: none;
  }
}

.file-explorer__context-menu {
  position: fixed;
  z-index: 3000;
  min-width: 200px;
  padding: 4px;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  background: #fff;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.12);
}

.file-explorer__context-divider {
  height: 1px;
  margin: 4px 0;
  background: #e5e7eb;
}

.file-explorer__context-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 10px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: #111827;
  font-size: 13px;
  text-align: left;
  cursor: pointer;

  &:hover {
    background: #f3f4f6;
  }

  &--danger {
    color: #dc2626;

    &:hover {
      background: #fef2f2;
    }
  }
}

.file-explorer__context-icon {
  flex-shrink: 0;
  color: #64748b;
}

.file-explorer__context-item--danger .file-explorer__context-icon {
  color: inherit;
}

.file-explorer__context-label {
  flex: 1;
  min-width: 0;
}

.file-explorer__context-hint {
  flex-shrink: 0;
  color: #94a3b8;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}
</style>
