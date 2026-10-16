<script setup lang="ts">
import { Icon } from "@/components/Icon";
import { computed } from "vue";
import ExplorerInlineInput from "@/pages/editor/components/FileExplorer/ExplorerInlineInput/index.vue";
import type { ExplorerTreeItem } from "@/pages/editor/components/FileExplorer/normalizeTree";
import type { InlineEditState } from "@/pages/editor/components/FileExplorer/types";
import {
  resolveExplorerFileIcon,
  resolveExplorerFolderIcon,
} from "@/pages/editor/components/FileExplorer/fileIcon";
import type { ExplorerDropIndicator } from "@/pages/editor/components/FileExplorer/useExplorerPointerDnD";
import ExplorerNode from "@/pages/editor/components/FileExplorer/ExplorerNode/index.vue";

defineOptions({ name: "ExplorerNode" });

const props = defineProps<{
  node: ExplorerTreeItem;
  depth: number;
  expandedKeys: string[];
  activePath?: string | null;
  inlineEdit?: InlineEditState | null;
  draggingPath?: string | null;
  dropIndicator?: ExplorerDropIndicator | null;
  workspaceRoot?: string | null;
}>();

const emit = defineEmits<{
  toggle: [path: string];
  contextmenu: [path: string, event: MouseEvent];
  inlineConfirm: [value: string];
  inlineCancel: [];
  pointerDownRow: [
    event: PointerEvent,
    node: ExplorerTreeItem & { icon: string; iconColor?: string },
  ];
  rowClick: [path: string, kind: ExplorerTreeItem["kind"]];
}>();

const isDirectory = computed(() => props.node.kind === "directory");
const isExpanded = computed(() => props.expandedKeys.includes(props.node.path));
const isActive = computed(() => props.activePath === props.node.path);
const isDimmed = computed(() => Boolean(props.node.hidden || props.node.ignored));
const isRenaming = computed(
  () => props.inlineEdit?.mode === "rename" && props.inlineEdit.targetPath === props.node.path,
);
const isDragging = computed(() => props.draggingPath === props.node.path);
const showCreateInline = computed(() => {
  if (!props.inlineEdit || !isDirectory.value || !isExpanded.value) {
    return false;
  }
  return (
    (props.inlineEdit.mode === "create-file" || props.inlineEdit.mode === "create-folder") &&
    props.inlineEdit.parentPath === props.node.path
  );
});
const createInlineKind = computed(() =>
  props.inlineEdit?.mode === "create-folder" ? "folder" : "file",
);

const dropPlacement = computed(() => {
  if (props.dropIndicator?.anchorPath !== props.node.path) {
    return null;
  }
  return props.dropIndicator.placement;
});

const fileIcon = computed(() =>
  isDirectory.value
    ? resolveExplorerFolderIcon(isExpanded.value)
    : resolveExplorerFileIcon(props.node),
);

function onRowClick() {
  if (isRenaming.value) {
    return;
  }
  emit("rowClick", props.node.path, props.node.kind);
}

function onChevronClick(event: MouseEvent) {
  event.stopPropagation();
  emit("toggle", props.node.path);
}

function onContextMenu(event: MouseEvent) {
  if (isRenaming.value) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  emit("contextmenu", props.node.path, event);
}

function onPointerDown(event: PointerEvent) {
  if (isRenaming.value) {
    return;
  }
  emit("pointerDownRow", event, {
    ...props.node,
    icon: fileIcon.value.icon,
    iconColor: fileIcon.value.color,
  });
}
</script>

<template>
  <div class="explorer-node">
    <ExplorerInlineInput
      v-if="isRenaming && inlineEdit"
      :depth="depth"
      :value="inlineEdit.value"
      :kind="node.kind === 'directory' ? 'folder' : 'file'"
      @confirm="emit('inlineConfirm', $event)"
      @cancel="emit('inlineCancel')"
    />
    <div
      v-else
      class="explorer-row"
      :class="{
        'explorer-row--active': isActive,
        'explorer-row--folder': isDirectory,
        'explorer-row--dimmed': isDimmed,
        'explorer-row--dragging': isDragging,
        'explorer-row--drop-inside': dropPlacement === 'inside',
        'explorer-row--drop-before': dropPlacement === 'before',
        'explorer-row--drop-after': dropPlacement === 'after',
      }"
      :style="{
        paddingLeft: `${depth * 12 + 8}px`,
        '--explorer-indent': `${depth * 12 + 8}px`,
      }"
      :data-explorer-path="node.path"
      :data-explorer-kind="node.kind"
      @click="onRowClick"
      @contextmenu="onContextMenu"
      @pointerdown="onPointerDown"
    >
      <button
        v-if="isDirectory"
        type="button"
        class="explorer-chevron"
        @click="onChevronClick"
      >
        <Icon :icon="isExpanded ? 'mdi:chevron-down' : 'mdi:chevron-right'" :size="14" />
      </button>
      <span v-else class="explorer-chevron explorer-chevron--placeholder" />

      <Icon
        :icon="fileIcon.icon"
        class="explorer-icon"
        :style="!isDimmed && fileIcon.color ? { color: fileIcon.color } : undefined"
        :size="15"
      />
      <span class="explorer-label">{{ node.title }}</span>
    </div>

    <template v-if="isDirectory && isExpanded">
      <ExplorerInlineInput
        v-if="showCreateInline && inlineEdit"
        :depth="depth + 1"
        :value="inlineEdit.value"
        :kind="createInlineKind"
        @confirm="emit('inlineConfirm', $event)"
        @cancel="emit('inlineCancel')"
      />
      <ExplorerNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        :expanded-keys="expandedKeys"
        :active-path="activePath"
        :inline-edit="inlineEdit"
        :dragging-path="draggingPath"
        :drop-indicator="dropIndicator"
        :workspace-root="workspaceRoot"
        @toggle="emit('toggle', $event)"
        @contextmenu="(path, event) => emit('contextmenu', path, event)"
        @inline-confirm="emit('inlineConfirm', $event)"
        @inline-cancel="emit('inlineCancel')"
        @pointer-down-row="(event, node) => emit('pointerDownRow', event, node)"
        @row-click="(path, kind) => emit('rowClick', path, kind)"
      />
    </template>
  </div>
</template>

<style scoped lang="scss">
.explorer-row {
  position: relative;
  display: flex;
  align-items: center;
  gap: 4px;
  min-height: 22px;
  padding-right: 8px;
  border-radius: 4px;
  cursor: default;
  user-select: none;
  color: var(--app-fg);
  font-size: 13px;
  touch-action: none;

  &:hover {
    background: var(--app-hover-bg);
  }

  &--active {
    background: var(--app-active-bg);
    color: var(--app-active-fg);
  }

  &--dimmed {
    color: var(--app-fg-muted);

    .explorer-icon,
    .explorer-chevron {
      color: var(--app-fg-muted);
    }
  }

  &--dimmed.explorer-row--active {
    color: var(--app-active-fg);

    .explorer-icon {
      color: var(--app-active-fg);
    }
  }

  &--dragging {
    opacity: 0.35;
  }

  /* 拖入目录：整行高亮（VS Code） */
  &--drop-inside {
    background: var(--app-active-bg);
    outline: 1px solid var(--app-active-fg);
    outline-offset: -1px;
  }

  /* 行间插入线 */
  &--drop-before::before,
  &--drop-after::after {
    content: "";
    position: absolute;
    left: var(--explorer-indent, 8px);
    right: 8px;
    height: 2px;
    border-radius: 1px;
    background: var(--app-active-fg);
    pointer-events: none;
    z-index: 2;
  }

  &--drop-before::before {
    top: -1px;
  }

  &--drop-after::after {
    bottom: -1px;
  }
}

.explorer-chevron {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--app-fg-muted);
  flex-shrink: 0;
  cursor: pointer;
  touch-action: manipulation;

  &--placeholder {
    cursor: default;
  }
}

.explorer-icon {
  flex-shrink: 0;
  color: var(--app-fg-secondary);
  pointer-events: none;
}

.explorer-row--active:not(.explorer-row--dimmed) .explorer-icon {
  color: var(--app-active-fg);
}

.explorer-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  pointer-events: none;
}
</style>
