<script setup lang="ts">
import { computed, nextTick, ref } from "vue";
import { Icon } from "@/components/Icon";
import { useEventListener } from "@vueuse/core";
import { encryptionFileIcon } from "@/modules/editor/encryption";
import type { OpenedEditorFile } from "@/modules/editor/types";

type TabAction =
  | "close"
  | "close-others"
  | "close-right"
  | "close-saved"
  | "close-all"
  | "copy-path";

type MenuItem = {
  key: TabAction;
  label: string;
  danger?: boolean;
  divider?: boolean;
  disabled?: boolean;
};

const props = defineProps<{
  tabs: OpenedEditorFile[];
  activePath?: string | null;
}>();

const emit = defineEmits<{
  select: [path: string];
  close: [path: string];
  closeOthers: [path: string];
  closeRight: [path: string];
  closeSaved: [];
  closeAll: [];
  copyPath: [path: string];
}>();

const VIEWPORT_PADDING = 8;

const menuRef = ref<HTMLElement | null>(null);
const menu = ref({
  visible: false,
  x: 0,
  y: 0,
  path: null as string | null,
});

const menuItems = computed<MenuItem[]>(() => {
  const path = menu.value.path;
  if (!path) {
    return [];
  }
  const idx = props.tabs.findIndex((tab) => tab.path === path);
  const hasOthers = props.tabs.length > 1;
  const hasRight = idx >= 0 && idx < props.tabs.length - 1;
  const hasSaved = props.tabs.some((tab) => !tab.dirty);

  return [
    { key: "close", label: "关闭" },
    { key: "close-others", label: "关闭其他", disabled: !hasOthers, divider: true },
    { key: "close-right", label: "关闭右侧", disabled: !hasRight },
    { key: "close-saved", label: "关闭已保存", disabled: !hasSaved },
    { key: "close-all", label: "全部关闭", danger: true, divider: true },
    { key: "copy-path", label: "复制路径", divider: true },
  ];
});

function closeMenu() {
  menu.value.visible = false;
}

async function placeMenu(x: number, y: number) {
  menu.value = { ...menu.value, x, y, visible: true };
  await nextTick();
  const el = menuRef.value;
  if (!el) {
    return;
  }
  const { width, height } = el.getBoundingClientRect();
  menu.value = {
    ...menu.value,
    x: Math.min(Math.max(VIEWPORT_PADDING, x), window.innerWidth - width - VIEWPORT_PADDING),
    y: Math.min(Math.max(VIEWPORT_PADDING, y), window.innerHeight - height - VIEWPORT_PADDING),
  };
}

function openTabMenu(path: string, event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  emit("select", path);
  menu.value = { ...menu.value, path };
  void placeMenu(event.clientX, event.clientY);
}

function onMenuAction(key: TabAction) {
  const path = menu.value.path;
  closeMenu();
  if (!path) {
    return;
  }
  switch (key) {
    case "close":
      emit("close", path);
      return;
    case "close-others":
      emit("closeOthers", path);
      return;
    case "close-right":
      emit("closeRight", path);
      return;
    case "close-saved":
      emit("closeSaved");
      return;
    case "close-all":
      emit("closeAll");
      return;
    case "copy-path":
      emit("copyPath", path);
  }
}

useEventListener(window, "click", () => {
  if (menu.value.visible) {
    closeMenu();
  }
});
useEventListener(window, "keydown", (event: KeyboardEvent) => {
  if (event.key === "Escape") {
    closeMenu();
  }
});
</script>

<template>
  <div v-if="tabs.length" class="editor-tabs">
    <button
      v-for="tab in tabs"
      :key="tab.path"
      type="button"
      class="editor-tabs__item"
      :class="{ 'editor-tabs__item--active': tab.path === activePath }"
      :title="tab.path"
      @click="emit('select', tab.path)"
      @contextmenu="openTabMenu(tab.path, $event)"
    >
      <Icon :icon="encryptionFileIcon(tab)" :size="14" />
      <span class="editor-tabs__label">
        {{ tab.name }}<span v-if="tab.dirty" class="editor-tabs__dirty">*</span>
      </span>
    </button>

    <Teleport to="body">
      <div
        v-if="menu.visible"
        ref="menuRef"
        class="editor-tabs__context-menu"
        :style="{ left: `${menu.x}px`, top: `${menu.y}px` }"
        @click.stop
        @contextmenu.prevent
      >
        <template v-for="item in menuItems" :key="item.key">
          <div v-if="item.divider" class="editor-tabs__context-divider" />
          <button
            type="button"
            class="editor-tabs__context-item"
            :class="{ 'editor-tabs__context-item--danger': item.danger }"
            :disabled="item.disabled"
            @click="onMenuAction(item.key)"
          >
            {{ item.label }}
          </button>
        </template>
      </div>
    </Teleport>
  </div>
</template>

<style scoped lang="scss">
.editor-tabs {
  display: flex;
  align-items: stretch;
  gap: 2px;
  padding: 0 8px;
  min-height: 36px;
  flex-shrink: 0;
  overflow-x: auto;
  border-bottom: 1px solid var(--app-border);
  background: var(--app-surface);
}

.editor-tabs__item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 220px;
  padding: 0 12px;
  border: 0;
  border-bottom: 2px solid transparent;
  background: transparent;
  color: var(--app-fg-secondary);
  font-size: 12px;
  cursor: pointer;
  flex-shrink: 0;

  &:hover {
    background: var(--app-hover-bg);
  }

  &--active {
    color: var(--app-active-fg);
    border-bottom-color: var(--app-active-fg);
    background: var(--app-surface);
  }
}

.editor-tabs__label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.editor-tabs__dirty {
  color: var(--app-danger-fg);
}

.editor-tabs__context-menu {
  position: fixed;
  z-index: 3000;
  min-width: 160px;
  padding: 4px;
  border: 1px solid var(--app-border);
  border-radius: 6px;
  background: var(--app-surface);
  box-shadow: 0 8px 24px rgb(0 0 0 / 18%);
  color: var(--app-fg);
}

.editor-tabs__context-divider {
  height: 1px;
  margin: 4px 0;
  background: var(--app-divider);
}

.editor-tabs__context-item {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 6px 10px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--app-fg);
  font-size: 13px;
  text-align: left;
  cursor: pointer;

  &:hover:not(:disabled) {
    background: var(--app-hover-bg);
  }

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  &--danger {
    color: var(--app-danger-fg);

    &:hover:not(:disabled) {
      background: var(--app-danger-bg);
    }
  }
}
</style>
