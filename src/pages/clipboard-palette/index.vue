<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { Icon } from "@iconify/vue";
import {
  getStatus,
  type ApplyAction,
  type ClipboardItemSummary,
} from "@/modules/clipboardAssistant";
import { isAppSessionLocked } from "@/modules/appLock";
import PaletteVirtualList from "@/pages/clipboard-palette/components/PaletteVirtualList.vue";
import PaletteSourceTabs from "@/pages/clipboard-palette/components/PaletteSourceTabs.vue";
import type { PaletteItemCommand } from "@/pages/clipboard-palette/components/PaletteItemCard.vue";
import { useClipboardPalette } from "@/pages/clipboard-palette/useClipboardPalette";
import { usePaletteGeometry } from "@/pages/clipboard-palette/usePaletteGeometry";
import { PALETTE_ITEM_HEIGHT, PALETTE_ITEM_WIDTH } from "@/pages/clipboard-palette/paletteItemMetrics";

const paletteSize = "middle" as const;

const {
  keyword,
  selectedSource,
  sourceTabs,
  loading,
  activeIndex,
  actionMode,
  items,
  paletteLayout,
  showSourceAppIcon,
  rememberWindowPosition,
  openSearchOnShow,
  loadItems,
  selectSource,
  runAction,
  togglePin,
  deleteItem,
  focusSearch,
  bootstrapPalette,
  hidePalette,
} = useClipboardPalette();

usePaletteGeometry(() => rememberWindowPosition.value);

const layoutClass = computed(() => `clipboard-palette--${paletteLayout.value}`);
const isHorizontalList = computed(
  () => paletteLayout.value === "topPanel" || paletteLayout.value === "bottomPanel",
);
const isSideLayout = computed(
  () => paletteLayout.value === "leftPanel" || paletteLayout.value === "rightPanel",
);
const tabOrientation = computed(() => (isSideLayout.value ? "vertical" : "horizontal"));
const showSourceTabs = computed(() => sourceTabs.value.length > 0);

type SearchInputRef = { focus?: () => void; input?: HTMLInputElement };
const searchInputRef = ref<SearchInputRef | null>(null);
const hoveredIndex = ref<number | null>(null);

async function ensureUsableOrHide(): Promise<boolean> {
  try {
    if (await isAppSessionLocked()) {
      await hidePalette();
      return false;
    }
    const stat = await getStatus();
    if (!stat.paletteActive) {
      await hidePalette();
      return false;
    }
  } catch {
    await hidePalette();
    return false;
  }
  return true;
}

function onPaletteOpen() {
  void (async () => {
    if (!(await ensureUsableOrHide())) return;
    keyword.value = "";
    selectedSource.value = undefined;
    activeIndex.value = 0;
    actionMode.value = "paste";
    await bootstrapPalette();
    focusSearch(searchInputRef.value, openSearchOnShow.value);
  })();
}

function selectItem(item: ClipboardItemSummary | undefined) {
  void runAction(item, actionMode.value);
}

function onItemAction(event: MouseEvent, item: ClipboardItemSummary, action: ApplyAction) {
  event.stopPropagation();
  void runAction(item, action);
}

function onItemCommand(item: ClipboardItemSummary, command: PaletteItemCommand) {
  if (command === "pastePlain") {
    void runAction(item, "paste", "plain");
    return;
  }
  if (command === "pasteRich") {
    void runAction(item, "paste", "rich");
    return;
  }
  if (command === "copyPlain") {
    void runAction(item, "copy", "plain");
    return;
  }
  if (command === "copyRich") {
    void runAction(item, "copy", "rich");
    return;
  }
  if (command === "pin") {
    void togglePin(item);
    return;
  }
  if (command === "delete") {
    void deleteItem(item);
  }
}

function toggleActionMode() {
  actionMode.value = actionMode.value === "paste" ? "copy" : "paste";
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.preventDefault();
    void hidePalette();
  }
}

function onSearchKeydown(event: KeyboardEvent) {
  const list = items.value;

  if (event.key === "Tab") {
    event.preventDefault();
    toggleActionMode();
    return;
  }

  if (/^[1-9]$/.test(event.key) && list.length) {
    const index = Number(event.key) - 1;
    if (index < list.length) {
      event.preventDefault();
      void runAction(list[index], actionMode.value);
    }
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
  if (event.key === "ArrowRight" && list.length && isHorizontalList.value) {
    event.preventDefault();
    activeIndex.value = (activeIndex.value + 1) % list.length;
    return;
  }
  if (event.key === "ArrowLeft" && list.length && isHorizontalList.value) {
    event.preventDefault();
    activeIndex.value = (activeIndex.value - 1 + list.length) % list.length;
    return;
  }
  if (event.key === "Enter" && list.length) {
    event.preventDefault();
    void runAction(list[activeIndex.value], actionMode.value);
    return;
  }
  if (event.key.toLowerCase() === "p" && list[activeIndex.value]) {
    event.preventDefault();
    void togglePin(list[activeIndex.value]);
    return;
  }
  if (event.key === "Delete" && list[activeIndex.value]) {
    event.preventDefault();
    void deleteItem(list[activeIndex.value]);
    return;
  }
  if (event.key === "Escape") {
    event.preventDefault();
    void hidePalette();
  }
}

function onClipboardChanged() {
  void loadItems(true);
}

onMounted(() => {
  window.addEventListener("appx:clipboard-palette-open", onPaletteOpen);
  window.addEventListener("appx:clipboard-changed", onClipboardChanged);
  void (async () => {
    if (!(await ensureUsableOrHide())) return;
    await bootstrapPalette();
    focusSearch(searchInputRef.value, openSearchOnShow.value);
  })();
  window.addEventListener("keydown", onKeydown);
});

onUnmounted(() => {
  window.removeEventListener("appx:clipboard-palette-open", onPaletteOpen);
  window.removeEventListener("appx:clipboard-changed", onClipboardChanged);
  window.removeEventListener("keydown", onKeydown);
});
</script>

<template>
  <a-config-provider :component-size="paletteSize">
    <div
      class="clipboard-palette"
      :class="layoutClass"
    >
      <div
        class="clipboard-palette__shell"
        :class="{ 'clipboard-palette__shell--side': isSideLayout }"
      >
        <PaletteSourceTabs
          v-if="showSourceTabs"
          :sources="sourceTabs"
          :active-bundle="selectedSource"
          :orientation="tabOrientation"
          @select="selectSource"
        />

        <div class="clipboard-palette__main">
          <a-input
            ref="searchInputRef"
            v-model:value="keyword"
            allow-clear
            class="clipboard-palette__search"
            placeholder="搜索剪贴板历史"
            @keydown="onSearchKeydown"
          >
            <template #prefix>
              <Icon icon="mdi:magnify" class="clipboard-palette__search-icon" aria-hidden="true" />
            </template>
          </a-input>

          <div v-if="loading" class="clipboard-palette__empty">加载中…</div>
          <div v-else-if="!items.length" class="clipboard-palette__empty">没有匹配的历史</div>

          <PaletteVirtualList
            v-else
            :items="items"
            :active-index="activeIndex"
            :hover-index="hoveredIndex"
            :action-mode="actionMode"
            :show-source-app-icon="showSourceAppIcon"
            :horizontal="isHorizontalList"
            :item-width="PALETTE_ITEM_WIDTH"
            :item-height="PALETTE_ITEM_HEIGHT"
            @select="selectItem"
            @action="onItemAction"
            @command="onItemCommand"
            @hover="hoveredIndex = $event"
          />
        </div>
      </div>
    </div>
  </a-config-provider>
</template>

<style scoped lang="scss">
.clipboard-palette {
  display: flex;
  flex-direction: column;
  height: 100vh;
  min-height: 0;
  padding: 0;
  box-sizing: border-box;
  background: var(--app-bg);
  color: var(--app-fg);

  &--topPanel,
  &--bottomPanel {
    .clipboard-palette__main {
      padding-bottom: 0;
    }
  }

  &--topPanel {
    .clipboard-palette__main {
      padding-top: 0;
    }
  }

  &--leftPanel,
  &--rightPanel,
  &--topPanel,
  &--bottomPanel {
    min-height: 0;
  }
}

.clipboard-palette__shell {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;

  &--side {
    flex-direction: row;
  }
}

.clipboard-palette__main {
  display: flex;
  flex-direction: column;
  flex: 1;
  gap: 10px;
  min-height: 0;
  min-width: 0;
  padding: 12px;

  .clipboard-palette--bottomPanel & {
    padding-top: 8px;
  }

  .clipboard-palette--topPanel & {
    padding-bottom: 8px;
  }

  .clipboard-palette--leftPanel & {
    padding: 12px 8px 12px 0;
  }

  .clipboard-palette--rightPanel & {
    padding: 12px 0 12px 8px;
  }
}

.clipboard-palette__search {
  flex-shrink: 0;
}

.clipboard-palette__search-icon {
  opacity: 0.55;
}

.clipboard-palette__empty {
  padding: 24px;
  text-align: center;
  color: var(--app-fg-muted);
}
</style>
