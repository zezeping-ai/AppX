<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { Icon } from "@iconify/vue";
import {
  CODE_SNIPPET_GROUPS,
  iconOfSnippetGroup,
  labelOfSnippetGroup,
  copyPaletteItem,
  hidePalette,
  insertPaletteItem,
  listPaletteItems,
  formatAbbreviationTrigger,
  getCodeSnippetSettings,
  inlineExpansionTrigger,
  setInlineExpansionTrigger,
  type CodeSnippetGroup,
  type CodeSnippetPaletteItem,
} from "@/modules/codeSnippets";
import { getErrorMessage } from "@/shared/error";
import { isAppSessionLocked } from "@/modules/appLock";

type PaletteAction = "insert" | "copy";

const paletteSize = "middle" as const;

const keyword = ref("");
const selectedGroup = ref<CodeSnippetGroup | undefined>(undefined);
const loading = ref(true);
const items = ref<CodeSnippetPaletteItem[]>([]);
const activeIndex = ref(0);
const actionMode = ref<PaletteAction>("insert");
type SearchInputRef = {
  focus: () => void;
};

const searchInputRef = ref<SearchInputRef | null>(null);

const groupFilters = computed(() => {
  const counts = new Map<CodeSnippetGroup, number>();
  for (const item of items.value) {
    counts.set(item.group, (counts.get(item.group) ?? 0) + 1);
  }
  return CODE_SNIPPET_GROUPS.filter((group) => counts.has(group.value)).map((group) => ({
    ...group,
    count: counts.get(group.value) ?? 0,
  }));
});

const filtered = computed(() => {
  const q = keyword.value.trim().toLowerCase();
  return items.value.filter((item) => {
    if (selectedGroup.value && item.group !== selectedGroup.value) return false;
    if (!q) return true;
    return (
      item.name.toLowerCase().includes(q) ||
      item.abbreviation.toLowerCase().includes(q) ||
      labelOfSnippetGroup(item.group).toLowerCase().includes(q)
    );
  });
});

watch([filtered, selectedGroup], () => {
  activeIndex.value = 0;
});

function focusSearch() {
  void nextTick(() => {
    requestAnimationFrame(() => {
      searchInputRef.value?.focus();
    });
  });
}

async function ensureUnlockedOrHide(): Promise<boolean> {
  try {
    if (await isAppSessionLocked()) {
      await hidePalette();
      return false;
    }
  } catch {
    // 忽略异常，避免面板卡死
  }
  return true;
}

function onPaletteOpen() {
  void (async () => {
    if (!(await ensureUnlockedOrHide())) return;
    keyword.value = "";
    selectedGroup.value = undefined;
    activeIndex.value = 0;
    actionMode.value = "insert";
    await loadItems();
    focusSearch();
  })();
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

function selectGroup(group: CodeSnippetGroup | undefined) {
  selectedGroup.value = group;
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
  void (async () => {
    if (!(await ensureUnlockedOrHide())) return;
    try {
      const settings = await getCodeSnippetSettings();
      setInlineExpansionTrigger(settings.inlineExpansionTrigger);
    } catch {
      // 独立窗口读取失败时保留默认触发键
    }
    await loadItems();
    focusSearch();
  })();
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("appx:palette-open", onPaletteOpen);
  window.removeEventListener("keydown", onKeydown);
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
