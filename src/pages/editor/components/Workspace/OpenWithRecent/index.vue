<script setup lang="ts">
import { Icon } from "@iconify/vue";
import { computed } from "vue";

const props = defineProps<{
  recentFolders: string[];
  recentFiles: string[];
}>();

const emit = defineEmits<{
  openFolder: [];
  openFile: [];
  selectRecentFolder: [path: string];
  selectRecentFile: [path: string];
  removeRecentFolder: [path: string];
  removeRecentFile: [path: string];
}>();

type RecentKind = "folder" | "file";

type RecentEntry = {
  path: string;
  name: string;
  parent: string;
};

function toEntries(paths: string[]): RecentEntry[] {
  return paths.map((path) => {
    const name = path.split(/[/\\]/).pop() ?? path;
    const parent = path
      .slice(0, Math.max(0, path.length - name.length))
      .replace(/[/\\]$/, "");
    return { path, name, parent };
  });
}

const folderEntries = computed(() => toEntries(props.recentFolders));
const fileEntries = computed(() => toEntries(props.recentFiles));
const hasRecent = computed(
  () => folderEntries.value.length > 0 || fileEntries.value.length > 0,
);

function onSelectRecent(kind: RecentKind, path: string) {
  if (kind === "folder") {
    emit("selectRecentFolder", path);
    return;
  }
  emit("selectRecentFile", path);
}

function onRemoveRecent(kind: RecentKind, path: string) {
  if (kind === "folder") {
    emit("removeRecentFolder", path);
    return;
  }
  emit("removeRecentFile", path);
}
</script>

<template>
  <a-dropdown-button
    trigger="click"
    placement="bottomLeft"
    class="open-with-recent"
    @click="emit('openFolder')"
  >
    <span class="open-with-recent__main">
      <Icon icon="mdi:folder-open-outline" width="16" height="16" />
      打开文件夹
    </span>
    <template #icon>
      <Icon icon="mdi:chevron-down" width="14" height="14" />
    </template>
    <template #overlay>
      <a-menu class="open-with-recent__menu">
        <a-menu-item key="__open_file__" @click="emit('openFile')">
          <div class="open-with-recent__action">
            <Icon icon="mdi:file-document-outline" width="15" height="15" />
            <span>打开文件</span>
          </div>
        </a-menu-item>

        <a-menu-divider />

        <template v-if="folderEntries.length">
          <a-menu-item-group>
            <template #title>
              <span class="open-with-recent__group-title">
                <Icon icon="mdi:folder-outline" width="14" height="14" />
                最近文件夹
              </span>
            </template>
            <a-menu-item
              v-for="entry in folderEntries"
              :key="`folder:${entry.path}`"
              @click="onSelectRecent('folder', entry.path)"
            >
              <div class="open-with-recent__item">
                <Icon
                  icon="mdi:folder-outline"
                  width="16"
                  height="16"
                  class="open-with-recent__item-icon open-with-recent__item-icon--folder"
                />
                <div class="open-with-recent__item-content">
                  <span class="open-with-recent__item-name">{{ entry.name }}</span>
                  <span v-if="entry.parent" class="open-with-recent__item-path">
                    {{ entry.parent }}
                  </span>
                </div>
                <button
                  type="button"
                  class="open-with-recent__remove-btn"
                  aria-label="删除历史记录"
                  title="删除"
                  @click.stop="onRemoveRecent('folder', entry.path)"
                >
                  <Icon icon="mdi:close" width="14" height="14" />
                </button>
              </div>
            </a-menu-item>
          </a-menu-item-group>
        </template>

        <template v-if="fileEntries.length">
          <a-menu-item-group>
            <template #title>
              <span class="open-with-recent__group-title">
                <Icon icon="mdi:file-document-outline" width="14" height="14" />
                最近文件
              </span>
            </template>
            <a-menu-item
              v-for="entry in fileEntries"
              :key="`file:${entry.path}`"
              @click="onSelectRecent('file', entry.path)"
            >
              <div class="open-with-recent__item">
                <Icon
                  icon="mdi:file-document-outline"
                  width="16"
                  height="16"
                  class="open-with-recent__item-icon open-with-recent__item-icon--file"
                />
                <div class="open-with-recent__item-content">
                  <span class="open-with-recent__item-name">{{ entry.name }}</span>
                  <span v-if="entry.parent" class="open-with-recent__item-path">
                    {{ entry.parent }}
                  </span>
                </div>
                <button
                  type="button"
                  class="open-with-recent__remove-btn"
                  aria-label="删除历史记录"
                  title="删除"
                  @click.stop="onRemoveRecent('file', entry.path)"
                >
                  <Icon icon="mdi:close" width="14" height="14" />
                </button>
              </div>
            </a-menu-item>
          </a-menu-item-group>
        </template>

        <a-menu-item v-if="!hasRecent" key="__empty__" disabled>无最近记录</a-menu-item>
      </a-menu>
    </template>
  </a-dropdown-button>
</template>

<style scoped lang="scss">
.open-with-recent {
  :deep(.ant-btn) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  :deep(.ant-btn-icon) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    line-height: 0;
  }

  :deep(.ant-btn svg) {
    display: block;
    vertical-align: middle;
  }
}

.open-with-recent__main {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  line-height: 1;

  :deep(svg) {
    display: block;
    flex-shrink: 0;
  }
}

.open-with-recent__menu {
  min-width: 280px;
  max-width: 420px;

  :deep(.ant-menu-item-group-title) {
    padding: 8px 12px 4px;
  }
}

.open-with-recent__group-title {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: #64748b;
  font-size: 12px;
  font-weight: 600;
}

.open-with-recent__action {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.open-with-recent__item {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.open-with-recent__item-icon {
  flex-shrink: 0;

  &--folder {
    color: #ca8a04;
  }

  &--file {
    color: #64748b;
  }
}

.open-with-recent__item-content {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
  line-height: 1.3;
}

.open-with-recent__item-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.open-with-recent__item-path {
  font-size: 11px;
  color: #6b7280;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.open-with-recent__remove-btn {
  border: 0;
  background: transparent;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  color: #9ca3af;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;

  &:hover {
    color: #374151;
    background: rgba(0, 0, 0, 0.06);
  }
}

[data-theme="dark"] .open-with-recent__remove-btn {
  color: #9ca3af;

  &:hover {
    color: #e5e7eb;
    background: rgba(255, 255, 255, 0.12);
  }
}
</style>
