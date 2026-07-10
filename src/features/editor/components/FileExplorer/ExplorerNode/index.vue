<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { computed } from "vue";
import { encryptionFileIcon } from "@/modules/editor/encryption";
import ExplorerInlineInput from "@/features/editor/components/FileExplorer/ExplorerInlineInput/index.vue";
import type { ExplorerTreeItem } from "@/features/editor/components/FileExplorer/normalizeTree";
import type { InlineEditState } from "@/features/editor/components/FileExplorer/types";
import ExplorerNode from "@/features/editor/components/FileExplorer/ExplorerNode/index.vue";

defineOptions({ name: "ExplorerNode" });

const props = defineProps<{
  node: ExplorerTreeItem;
  depth: number;
  expandedKeys: string[];
  activePath?: string | null;
  inlineEdit?: InlineEditState | null;
}>();

const emit = defineEmits<{
  toggle: [path: string];
  open: [path: string];
  contextmenu: [path: string, event: MouseEvent];
  inlineConfirm: [value: string];
  inlineCancel: [];
}>();

const isDirectory = computed(() => props.node.kind === "directory");
const isExpanded = computed(() => props.expandedKeys.includes(props.node.path));
const isActive = computed(() => props.activePath === props.node.path);
const isRenaming = computed(
  () => props.inlineEdit?.mode === "rename" && props.inlineEdit.targetPath === props.node.path,
);
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

const iconName = computed(() => {
  if (isDirectory.value) {
    return isExpanded.value ? "mdi:folder-open-outline" : "mdi:folder-outline";
  }
  if (props.node.customEncrypted || props.node.encrypted) {
    return encryptionFileIcon({
      encrypted: Boolean(props.node.encrypted),
      customEncrypted: Boolean(props.node.customEncrypted),
    });
  }
  return "mdi:file-document-outline";
});

function onRowClick() {
  if (isRenaming.value) {
    return;
  }
  if (isDirectory.value) {
    emit("toggle", props.node.path);
    return;
  }
  emit("open", props.node.path);
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
      }"
      :style="{ paddingLeft: `${depth * 12 + 8}px` }"
      @click="onRowClick"
      @contextmenu="onContextMenu"
    >
      <button
        v-if="isDirectory"
        type="button"
        class="explorer-chevron"
        @click="onChevronClick"
      >
        <Icon :icon="isExpanded ? 'mdi:chevron-down' : 'mdi:chevron-right'" width="14" height="14" />
      </button>
      <span v-else class="explorer-chevron explorer-chevron--placeholder" />

      <Icon :icon="iconName" width="15" height="15" class="explorer-icon" />
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
        @toggle="emit('toggle', $event)"
        @open="emit('open', $event)"
        @contextmenu="(path, event) => emit('contextmenu', path, event)"
        @inline-confirm="emit('inlineConfirm', $event)"
        @inline-cancel="emit('inlineCancel')"
      />
    </template>
  </div>
</template>

<style scoped lang="scss">
.explorer-row {
  display: flex;
  align-items: center;
  gap: 4px;
  min-height: 22px;
  padding-right: 8px;
  border-radius: 4px;
  cursor: pointer;
  user-select: none;
  color: #374151;
  font-size: 13px;

  &:hover {
    background: #f3f4f6;
  }

  &--active {
    background: #e8f0fe;
    color: #1d4ed8;
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
  color: #6b7280;
  flex-shrink: 0;
  cursor: pointer;

  &--placeholder {
    cursor: default;
  }
}

.explorer-icon {
  flex-shrink: 0;
  color: #6b7280;
}

.explorer-row--active .explorer-icon {
  color: #1d4ed8;
}

.explorer-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
