<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from "vue";
import { Icon } from "@iconify/vue";
import {
  getStatus,
  type ClipboardItemSummary,
} from "@/modules/clipboardAssistant";
import { isAppSessionLocked } from "@/modules/appLock";
import { useFloatingPaletteKeyboard } from "@/hooks/useFloatingPaletteKeyboard";
import type { PaletteSearchInputRef } from "@/hooks/focusPaletteSearch";
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
  confirmDeleteItem,
  openLink,
  revealFile,
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

const searchInputRef = ref<PaletteSearchInputRef | null>(null);
const virtualListRef = ref<{ scrollToActiveIndex: () => void } | null>(null);

function toggleActionMode() {
  actionMode.value = actionMode.value === "paste" ? "copy" : "paste";
}

const { onSearchKeydown, focusSearch } = useFloatingPaletteKeyboard<ClipboardItemSummary>({
  activeIndex,
  keyword,
  list: () => items.value,
  horizontal: () => isHorizontalList.value,
  scrollToActive: () => virtualListRef.value?.scrollToActiveIndex(),
  onApply: (item) => void runAction(item, actionMode.value),
  onEscape: () => void hidePalette(),
  onToggleActionMode: toggleActionMode,
  shortcutActions: {
    numberKeys: true,
    onPin: (item) => void togglePin(item),
    onDelete: (item) => confirmDeleteItem(item),
  },
});

const commandHandlers: Record<PaletteItemCommand, (item: ClipboardItemSummary) => void> = {
  pastePlain: (item) => void runAction(item, "paste", "plain"),
  pasteRich: (item) => void runAction(item, "paste", "rich"),
  copyPlain: (item) => void runAction(item, "copy", "plain"),
  copyRich: (item) => void runAction(item, "copy", "rich"),
  openLink: (item) => void openLink(item),
  revealFile: (item) => void revealFile(item),
  pin: (item) => void togglePin(item),
  delete: (item) => confirmDeleteItem(item),
};

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
    void nextTick(() => virtualListRef.value?.scrollToActiveIndex());
  })();
}

function onClipboardChanged() {
  void loadItems(true);
}

onMounted(() => {
  window.addEventListener("appx:clipboard-palette-open", onPaletteOpen);
  window.addEventListener("appx:clipboard-changed", onClipboardChanged);
  void bootstrapPalette().catch(() => {});
});

onUnmounted(() => {
  window.removeEventListener("appx:clipboard-palette-open", onPaletteOpen);
  window.removeEventListener("appx:clipboard-changed", onClipboardChanged);
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
            placeholder="搜索剪切历史"
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
            ref="virtualListRef"
            :items="items"
            :active-index="activeIndex"
            :show-source-app-icon="showSourceAppIcon"
            :horizontal="isHorizontalList"
            :item-width="PALETTE_ITEM_WIDTH"
            :item-height="PALETTE_ITEM_HEIGHT"
            @select="(item) => void runAction(item, actionMode)"
            @command="(item, command) => commandHandlers[command](item)"
            @active-index="activeIndex = $event"
          />

          <footer class="clipboard-palette__footer">
            <span class="clipboard-palette__footer-hint">
              {{ isHorizontalList ? "←→ 选择" : "↑↓ 选择" }}
            </span>
            <span class="clipboard-palette__footer-hint">Tab 切换操作</span>
            <div class="clipboard-palette__footer-mode">
              <a-button
                :type="actionMode === 'paste' ? 'primary' : 'text'"
                @click="actionMode = 'paste'"
              >
                <span class="clipboard-palette__footer-mode-label">
                  <Icon icon="mdi:content-paste" aria-hidden="true" />
                  <span>粘贴</span>
                </span>
              </a-button>
              <a-button
                :type="actionMode === 'copy' ? 'primary' : 'text'"
                @click="actionMode = 'copy'"
              >
                <span class="clipboard-palette__footer-mode-label">
                  <Icon icon="mdi:content-copy" aria-hidden="true" />
                  <span>复制</span>
                </span>
              </a-button>
            </div>
            <span class="clipboard-palette__footer-hint">右键更多</span>
            <span class="clipboard-palette__footer-hint">Esc 关闭</span>
          </footer>
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

.clipboard-palette__footer {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
  padding-top: 2px;
  font-size: 12px;
  color: var(--app-fg-muted);
  line-height: 1;
}

.clipboard-palette__footer-hint {
  display: inline-flex;
  align-items: center;
  min-height: 28px;
}

.clipboard-palette__footer-mode {
  display: inline-flex;
  align-items: center;
  gap: 4px;

  :deep(.ant-btn) {
    display: inline-flex;
    align-items: center;
    padding-inline: 8px;
    height: 28px;
  }
}

.clipboard-palette__footer-mode-label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  line-height: 1;
  font-size: 12px;
}
</style>
