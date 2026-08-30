<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { Icon } from "@iconify/vue";
import type { MenuProps } from "ant-design-vue";
import {
  iconOfSnippetGroup,
  labelOfSnippetGroup,
  formatAbbreviationTrigger,
  inlineExpansionTrigger,
  type CodeSnippetPaletteItem,
} from "@/modules/codeSnippets";
import { useFloatingPaletteKeyboard } from "@/hooks/useFloatingPaletteKeyboard";
import type { PaletteSearchInputRef } from "@/hooks/focusPaletteSearch";
import {
  useSnippetPalette,
  type SnippetPaletteAction,
} from "@/pages/snippet-palette/useSnippetPalette";

type SnippetContextCommand = SnippetPaletteAction | "delete";

const paletteSize = "middle" as const;

const {
  keyword,
  selectedGroup,
  loading,
  items,
  filtered,
  groupFilters,
  activeIndex,
  actionMode,
  selectGroup,
  toggleActionMode,
  selectItem,
  onItemAction,
  removeSnippet,
  runAction,
  resetForOpen,
  bootstrap,
  hidePalette,
} = useSnippetPalette();

const searchInputRef = ref<PaletteSearchInputRef | null>(null);
const listRef = ref<HTMLElement | null>(null);

const snippetContextMenuItems: MenuProps["items"] = [
  { key: "insert", label: "插入" },
  { key: "copy", label: "拷贝" },
  { type: "divider" },
  { key: "delete", label: "删除", danger: true },
];

const { onSearchKeydown, focusSearch } = useFloatingPaletteKeyboard<CodeSnippetPaletteItem>({
  activeIndex,
  keyword,
  list: () => filtered.value,
  listRef,
  onApply: (item) => void runAction(item, actionMode.value),
  onEscape: () => void hidePalette(),
  onToggleActionMode: toggleActionMode,
});

function onSnippetContextAction(item: CodeSnippetPaletteItem, command: SnippetContextCommand) {
  if (command === "delete") {
    removeSnippet(item);
    return;
  }
  void runAction(item, command);
}

function createSnippetContextMenuClick(item: CodeSnippetPaletteItem) {
  return ({ key }: { key: string | number }) => {
    onSnippetContextAction(item, String(key) as SnippetContextCommand);
  };
}

function onPaletteOpen() {
  void (async () => {
    if (!(await resetForOpen())) return;
    focusSearch(searchInputRef.value);
  })();
}

onMounted(() => {
  window.addEventListener("appx:palette-open", onPaletteOpen);
  void (async () => {
    if (!(await bootstrap())) return;
    focusSearch(searchInputRef.value);
  })();
});

onUnmounted(() => {
  window.removeEventListener("appx:palette-open", onPaletteOpen);
});
</script>

<template>
  <a-config-provider :component-size="paletteSize">
    <div class="snippet-palette">
      <a-input
        ref="searchInputRef"
        v-model:value="keyword"
        allow-clear
        class="snippet-palette__search"
        placeholder="搜索代码段名称或缩写"
        @keydown="onSearchKeydown"
      >
        <template #prefix>
          <Icon icon="mdi:magnify" class="snippet-palette__search-icon" aria-hidden="true" />
        </template>
      </a-input>

      <div v-if="!loading && groupFilters.length" class="snippet-palette__groups">
        <button
          type="button"
          class="snippet-palette__group"
          :class="{ 'snippet-palette__group--active': !selectedGroup }"
          @click="selectGroup(undefined)"
        >
          全部
          <span class="snippet-palette__group-count">{{ items.length }}</span>
        </button>
        <button
          v-for="group in groupFilters"
          :key="group.value"
          type="button"
          class="snippet-palette__group"
          :class="{ 'snippet-palette__group--active': selectedGroup === group.value }"
          @click="selectGroup(group.value)"
        >
          <Icon :icon="group.icon" class="snippet-palette__group-icon" aria-hidden="true" />
          {{ group.label }}
          <span class="snippet-palette__group-count">{{ group.count }}</span>
        </button>
      </div>

      <div v-if="loading" class="snippet-palette__empty">加载中…</div>
      <div v-else-if="!filtered.length" class="snippet-palette__empty">没有匹配的代码段</div>
      <ul v-else ref="listRef" class="snippet-palette__list">
        <li
          v-for="(item, index) in filtered"
          :key="item.id"
          :data-index="index"
          class="snippet-palette__item"
          :class="{ 'snippet-palette__item--active': index === activeIndex }"
          @mouseenter="activeIndex = index"
        >
          <a-dropdown :trigger="['contextmenu']" class="snippet-palette__item-dropdown">
            <div class="snippet-palette__item-body" @click="selectItem(item)">
              <div class="snippet-palette__item-main">
                <div class="snippet-palette__item-title">
                  <span class="snippet-palette__group-tag">
                    <Icon :icon="iconOfSnippetGroup(item.group)" aria-hidden="true" />
                    {{ labelOfSnippetGroup(item.group) }}
                  </span>
                  <span class="snippet-palette__name">{{ item.name }}</span>
                </div>
                <span class="snippet-palette__abbr">{{
                  formatAbbreviationTrigger(item.abbreviation, inlineExpansionTrigger)
                }}</span>
              </div>
              <div class="snippet-palette__actions">
                <a-button
                  :type="actionMode === 'insert' ? 'primary' : 'default'"
                  @click="onItemAction($event, item, 'insert')"
                >
                  <template #icon>
                    <Icon icon="mdi:keyboard-return" aria-hidden="true" />
                  </template>
                  插入
                </a-button>
                <a-button
                  :type="actionMode === 'copy' ? 'primary' : 'default'"
                  @click="onItemAction($event, item, 'copy')"
                >
                  <template #icon>
                    <Icon icon="mdi:content-copy" aria-hidden="true" />
                  </template>
                  拷贝
                </a-button>
              </div>
            </div>
            <template #overlay>
              <a-menu
                :items="snippetContextMenuItems"
                @click="createSnippetContextMenuClick(item)"
              />
            </template>
          </a-dropdown>
        </li>
      </ul>

      <footer class="snippet-palette__footer">
        <span class="snippet-palette__footer-hint">↑↓ 选择</span>
        <span class="snippet-palette__footer-hint">Tab 切换操作</span>
        <div class="snippet-palette__footer-mode">
          <a-button
            :type="actionMode === 'insert' ? 'primary' : 'text'"
            @click="actionMode = 'insert'"
          >
            <span class="snippet-palette__footer-mode-label">
              <Icon icon="mdi:keyboard-return" aria-hidden="true" />
              <span>直接插入</span>
            </span>
          </a-button>
          <a-button
            :type="actionMode === 'copy' ? 'primary' : 'text'"
            @click="actionMode = 'copy'"
          >
            拷贝
          </a-button>
        </div>
        <span class="snippet-palette__footer-hint">Esc 关闭</span>
      </footer>
    </div>
  </a-config-provider>
</template>

<style scoped lang="scss">
.snippet-palette {
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 12px;
  background: var(--app-bg, #fff);
  color: var(--app-fg, rgba(0, 0, 0, 0.88));
  font-size: 14px;
}

.snippet-palette__search {
  margin-bottom: 10px;

  :deep(.ant-input-affix-wrapper) {
    min-height: 36px;
    padding-inline: 12px;
    font-size: 15px;
  }

  :deep(.ant-input) {
    font-size: 15px;
  }
}

.snippet-palette__search-icon {
  color: rgb(0 0 0 / 45%);
  font-size: 17px;
}

.snippet-palette__groups {
  display: flex;
  gap: 6px;
  margin-bottom: 8px;
  overflow-x: auto;
  padding-bottom: 2px;

  &::-webkit-scrollbar {
    height: 3px;
  }

  &::-webkit-scrollbar-thumb {
    background: rgb(0 0 0 / 12%);
    border-radius: 2px;
  }
}

.snippet-palette__group {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  height: 24px;
  padding: 0 8px;
  border: 1px solid rgb(0 0 0 / 10%);
  border-radius: 999px;
  background: transparent;
  color: rgb(0 0 0 / 55%);
  font-size: 11px;
  line-height: 1;
  cursor: pointer;

  &--active,
  &:hover {
    border-color: rgb(22 119 255 / 40%);
    background: rgb(22 119 255 / 8%);
    color: #1677ff;
  }
}

.snippet-palette__group-icon {
  font-size: 12px;
}

.snippet-palette__group-count {
  color: rgb(0 0 0 / 40%);
  font-size: 10px;

  .snippet-palette__group--active & {
    color: rgb(22 119 255 / 65%);
  }
}

.snippet-palette__list {
  flex: 1;
  margin: 0;
  padding: 0;
  list-style: none;
  overflow: auto;
}

.snippet-palette__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;

  &--active,
  &:hover {
    background: rgb(22 119 255 / 10%);
  }
}

.snippet-palette__item-dropdown {
  display: block;
  width: 100%;
}

.snippet-palette__item-body {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
}

.snippet-palette__item-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex: 1;
  min-width: 0;
}

.snippet-palette__item-title {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.snippet-palette__group-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
  padding: 2px 6px;
  border-radius: 4px;
  background: rgb(0 0 0 / 5%);
  color: rgb(0 0 0 / 55%);
  font-size: 11px;
  line-height: 20px;
}

.snippet-palette__name {
  font-size: 14px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.snippet-palette__abbr {
  flex-shrink: 0;
  font-size: 12px;
  color: rgb(0 0 0 / 55%);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.snippet-palette__actions {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.12s ease;

  .snippet-palette__item--active &,
  .snippet-palette__item:hover & {
    opacity: 1;
  }
}

.snippet-palette__empty {
  flex: 1;
  display: grid;
  place-items: center;
  color: rgb(0 0 0 / 45%);
  font-size: 14px;
}

.snippet-palette__footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 12px;
  padding-top: 10px;
  font-size: 12px;
  color: rgb(0 0 0 / 45%);
  line-height: 1;
}

.snippet-palette__footer-hint {
  display: inline-flex;
  align-items: center;
  min-height: 32px;
}

.snippet-palette__footer-mode {
  display: inline-flex;
  align-items: center;
  gap: 4px;

  :deep(.ant-btn) {
    display: inline-flex;
    align-items: center;
    padding-inline: 8px;
  }
}

.snippet-palette__footer-mode-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  line-height: 1;
  font-size: 12px;
}
</style>
