<script setup lang="ts">
import { Icon } from "@iconify/vue";
import type { MenuProps } from "ant-design-vue";

defineProps<{
  label: string;
  icon: string;
  recentPaths: string[];
}>();

const emit = defineEmits<{
  primary: [];
  selectRecent: [path: string];
  removeRecent: [path: string];
}>();

function splitPath(path: string) {
  const name = path.split(/[/\\]/).pop() ?? path;
  const parent = path.slice(0, Math.max(0, path.length - name.length)).replace(/[/\\]$/, "");
  return { name, parent };
}

const onMenuClick: MenuProps["onClick"] = ({ key }) => {
  if (key === "__empty__") {
    return;
  }
  emit("selectRecent", String(key));
};
</script>

<template>
  <a-dropdown-button
    trigger="click"
    placement="bottomLeft"
    class="open-with-recent"
    @click="emit('primary')"
  >
    <span class="open-with-recent__main">
      <Icon :icon="icon" width="16" height="16" />
      {{ label }}
    </span>
    <template #icon>
      <Icon icon="mdi:chevron-down" width="14" height="14" />
    </template>
    <template #overlay>
      <a-menu class="open-with-recent__menu" @click="onMenuClick">
        <template v-if="recentPaths.length">
          <a-menu-item v-for="path in recentPaths" :key="path">
            <div class="open-with-recent__item">
              <div class="open-with-recent__item-content">
                <span class="open-with-recent__item-name">{{ splitPath(path).name }}</span>
                <span v-if="splitPath(path).parent" class="open-with-recent__item-path">
                  {{ splitPath(path).parent }}
                </span>
              </div>
              <button
                type="button"
                class="open-with-recent__remove-btn"
                aria-label="删除历史记录"
                title="删除"
                @click.stop="emit('removeRecent', path)"
              >
                <Icon icon="mdi:close" width="14" height="14" />
              </button>
            </div>
          </a-menu-item>
        </template>
        <a-menu-item v-else key="__empty__" disabled>无最近记录</a-menu-item>
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
  min-width: 240px;
  max-width: 420px;
}

.open-with-recent__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  min-width: 0;
}

.open-with-recent__item-content {
  display: flex;
  flex: 1;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
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
