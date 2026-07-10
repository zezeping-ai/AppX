<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { Icon } from "@iconify/vue";
import {
  copyPaletteItem,
  hidePalette,
  insertPaletteItem,
  listPaletteItems,
  type CodeSnippetPaletteItem,
} from "@/modules/palette";
import { formatAbbreviationTrigger } from "@/shared/abbreviation";
import { getErrorMessage } from "@/shared/error";

type PaletteAction = "insert" | "copy";

const keyword = ref("");
const loading = ref(true);
const items = ref<CodeSnippetPaletteItem[]>([]);
const activeIndex = ref(0);
const actionMode = ref<PaletteAction>("insert");
const searchInputRef = ref<HTMLInputElement | null>(null);

const filtered = computed(() => {
  const q = keyword.value.trim().toLowerCase();
  if (!q) return items.value;
  return items.value.filter(
    (item) =>
      item.name.toLowerCase().includes(q) ||
      item.abbreviation.toLowerCase().includes(q),
  );
});

watch(filtered, () => {
  activeIndex.value = 0;
});

function focusSearch() {
  void nextTick(() => {
    requestAnimationFrame(() => {
      searchInputRef.value?.focus();
      searchInputRef.value?.select();
    });
  });
}

function onPaletteOpen() {
  keyword.value = "";
  activeIndex.value = 0;
  actionMode.value = "insert";
  void loadItems();
  focusSearch();
}

async function loadItems() {
  loading.value = true;
  try {
    items.value = await listPaletteItems();
    activeIndex.value = 0;
  } catch (error) {
    console.error(getErrorMessage(error, "加载代码段失败"));
  } finally {
    loading.value = false;
  }
}

async function runAction(item: CodeSnippetPaletteItem | undefined, action: PaletteAction) {
  if (!item) return;
  try {
    if (action === "copy") {
      await copyPaletteItem(item.id);
    } else {
      await insertPaletteItem(item.id);
    }
  } catch (error) {
    console.error(getErrorMessage(error, action === "copy" ? "拷贝失败" : "插入失败"));
  }
}

function selectItem(item: CodeSnippetPaletteItem | undefined) {
  void runAction(item, actionMode.value);
}

function onItemAction(event: MouseEvent, item: CodeSnippetPaletteItem, action: PaletteAction) {
  event.stopPropagation();
  void runAction(item, action);
}

function toggleActionMode() {
  actionMode.value = actionMode.value === "insert" ? "copy" : "insert";
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    void hidePalette();
  }
}

function onSearchKeydown(event: KeyboardEvent) {
  const list = filtered.value;

  if (event.key === "Tab") {
    event.preventDefault();
    toggleActionMode();
    return;
  }

  if (event.key === "ArrowDown" && list.length) {
    event.preventDefault();
    activeIndex.value = (activeIndex.value + 1) % list.length;
    return;
  }
  if (event.key === "ArrowUp" && list.length) {
    event.preventDefault();
    activeIndex.value = (activeIndex.value - 1 + list.length) % list.length;
    return;
  }
  if (event.key === "Enter" && list.length) {
    event.preventDefault();
    void runAction(list[activeIndex.value], actionMode.value);
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    void hidePalette();
  }
}

onMounted(() => {
  window.addEventListener("appx:palette-open", onPaletteOpen);
  void loadItems();
  focusSearch();
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("appx:palette-open", onPaletteOpen);
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <div class="snippet-palette">
    <div class="snippet-palette__search">
      <Icon icon="mdi:magnify" class="snippet-palette__search-icon" aria-hidden="true" />
      <input
        ref="searchInputRef"
        v-model="keyword"
        class="snippet-palette__input"
        placeholder="搜索代码段名称或缩写"
        @keydown="onSearchKeydown"
      />
    </div>

    <div v-if="loading" class="snippet-palette__empty">加载中…</div>
    <div v-else-if="!filtered.length" class="snippet-palette__empty">没有匹配的代码段</div>
    <ul v-else class="snippet-palette__list">
      <li
        v-for="(item, index) in filtered"
        :key="item.id"
        class="snippet-palette__item"
        :class="{ 'snippet-palette__item--active': index === activeIndex }"
        @mouseenter="activeIndex = index"
        @click="selectItem(item)"
      >
        <div class="snippet-palette__item-main">
          <span class="snippet-palette__name">{{ item.name }}</span>
          <span class="snippet-palette__abbr">{{ formatAbbreviationTrigger(item.abbreviation) }}</span>
        </div>
        <div class="snippet-palette__actions">
          <button
            type="button"
            class="snippet-palette__action"
            :class="{ 'snippet-palette__action--primary': actionMode === 'insert' }"
            title="直接插入"
            @click="onItemAction($event, item, 'insert')"
          >
            <Icon icon="mdi:keyboard-return" aria-hidden="true" />
            <span>插入</span>
          </button>
          <button
            type="button"
            class="snippet-palette__action"
            :class="{ 'snippet-palette__action--primary': actionMode === 'copy' }"
            title="拷贝到剪贴板"
            @click="onItemAction($event, item, 'copy')"
          >
            <Icon icon="mdi:content-copy" aria-hidden="true" />
            <span>拷贝</span>
          </button>
        </div>
      </li>
    </ul>

    <footer class="snippet-palette__footer">
      <span>↑↓ 选择</span>
      <span>Tab 切换操作</span>
      <span class="snippet-palette__footer-mode">
        ↵
        <button
          type="button"
          class="snippet-palette__mode"
          :class="{ 'snippet-palette__mode--active': actionMode === 'insert' }"
          @click="actionMode = 'insert'"
        >
          直接插入
        </button>
        <button
          type="button"
          class="snippet-palette__mode"
          :class="{ 'snippet-palette__mode--active': actionMode === 'copy' }"
          @click="actionMode = 'copy'"
        >
          拷贝
        </button>
      </span>
      <span>Esc 关闭</span>
    </footer>
  </div>
</template>

<style scoped lang="scss">
.snippet-palette {
  display: flex;
  flex-direction: column;
  height: 100vh;
  padding: 12px;
  background: var(--app-bg, #fff);
  color: var(--app-fg, rgba(0, 0, 0, 0.88));
}

.snippet-palette__search {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: 1px solid rgb(0 0 0 / 12%);
  border-radius: 8px;
  margin-bottom: 10px;
}

.snippet-palette__search-icon {
  color: rgb(0 0 0 / 45%);
  font-size: 18px;
}

.snippet-palette__input {
  flex: 1;
  border: 0;
  outline: none;
  background: transparent;
  font-size: 14px;
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
  gap: 12px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;

  &--active,
  &:hover {
    background: rgb(22 119 255 / 10%);
  }
}

.snippet-palette__item-main {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex: 1;
  min-width: 0;
}

.snippet-palette__name {
  font-size: 13px;
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
  gap: 4px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.12s ease;

  .snippet-palette__item--active &,
  .snippet-palette__item:hover & {
    opacity: 1;
  }
}

.snippet-palette__action {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border: 1px solid rgb(0 0 0 / 12%);
  border-radius: 4px;
  background: rgb(255 255 255 / 80%);
  color: rgb(0 0 0 / 65%);
  font-size: 11px;
  cursor: pointer;

  &:hover {
    border-color: rgb(22 119 255 / 45%);
    color: #1677ff;
  }

  &--primary {
    border-color: rgb(22 119 255 / 45%);
    background: rgb(22 119 255 / 12%);
    color: #1677ff;
  }
}

.snippet-palette__empty {
  flex: 1;
  display: grid;
  place-items: center;
  color: rgb(0 0 0 / 45%);
  font-size: 13px;
}

.snippet-palette__footer {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding-top: 10px;
  font-size: 11px;
  color: rgb(0 0 0 / 45%);
}

.snippet-palette__footer-mode {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.snippet-palette__mode {
  padding: 2px 6px;
  border: 1px solid transparent;
  border-radius: 4px;
  background: transparent;
  color: rgb(0 0 0 / 45%);
  font-size: 11px;
  cursor: pointer;

  &--active {
    border-color: rgb(22 119 255 / 35%);
    background: rgb(22 119 255 / 10%);
    color: #1677ff;
  }

  &:hover {
    color: #1677ff;
  }
}
</style>
