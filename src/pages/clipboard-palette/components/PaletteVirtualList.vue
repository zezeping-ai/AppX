<script setup lang="ts">
import { useResizeObserver, useVirtualList } from "@vueuse/core";
import { computed, nextTick, ref, watch, type ComponentPublicInstance } from "vue";
import type { ClipboardItemSummary } from "@/modules/clipboardAssistant";
import {
  PALETTE_ITEM_GAP,
  PALETTE_ITEM_HEIGHT,
  PALETTE_ITEM_WIDTH,
} from "@/pages/clipboard-palette/paletteItemMetrics";
import PaletteItemCard, {
  type PaletteItemCommand,
} from "@/pages/clipboard-palette/components/PaletteItemCard.vue";

const props = withDefaults(
  defineProps<{
    items: ClipboardItemSummary[];
    activeIndex: number;
    showSourceAppIcon: boolean;
    horizontal?: boolean;
    itemHeight?: number;
    itemWidth?: number;
  }>(),
  {
    itemHeight: PALETTE_ITEM_HEIGHT,
    itemWidth: PALETTE_ITEM_WIDTH,
  },
);

const emit = defineEmits<{
  select: [ClipboardItemSummary];
  command: [ClipboardItemSummary, PaletteItemCommand];
  activeIndex: [number];
}>();

/** 条目底部与滚动条/面板底边的间距 */
const ITEM_BOTTOM_GAP = 6;
/** 无横向滚动条时模拟滚动条占用高度（macOS overlay 滚动条不占位） */
const SCROLLBAR_RESERVE = 10;

const scrollEl = ref<HTMLElement | null>(null);
const viewportHeight = ref(0);
const horizontalContentOverflows = ref(false);

function updateViewportMetrics() {
  const el = scrollEl.value;
  if (!el) return;
  viewportHeight.value = el.clientHeight;
  horizontalContentOverflows.value = el.scrollWidth > el.clientWidth + 1;
}

useResizeObserver(scrollEl, updateViewportMetrics);
watch(scrollEl, updateViewportMetrics);
watch(
  () => props.items.length,
  () => void nextTick(updateViewportMetrics),
);

const source = computed(() => props.items);

/** 侧栏纵向列表：虚拟滚动 */
const { list, containerProps, wrapperProps, scrollTo } = useVirtualList(source, {
  itemHeight: () => props.itemHeight + PALETTE_ITEM_GAP,
  overscan: 6,
});

/** 顶部/底部横条：useVirtualList 只认 scrollTop，横向滚动无法正确虚拟化，直接全量渲染 */
const horizontalItemStride = computed(() => props.itemWidth + PALETTE_ITEM_GAP);

const horizontalBottomInset = computed(() =>
  horizontalContentOverflows.value ? ITEM_BOTTOM_GAP : SCROLLBAR_RESERVE + ITEM_BOTTOM_GAP,
);

const resolvedItemHeight = computed(() => {
  if (!props.horizontal) return props.itemHeight;
  if (viewportHeight.value <= 0) return props.itemHeight;
  return Math.max(props.itemHeight, viewportHeight.value - horizontalBottomInset.value);
});

const containerBind = computed(() => {
  const { ref: _ref, ...rest } = containerProps;
  return rest;
});

function setScrollContainerRef(el: Element | ComponentPublicInstance | null) {
  const node = el instanceof Element ? (el as HTMLElement) : null;
  scrollEl.value = node;
  updateViewportMetrics();
  if (!props.horizontal) {
    containerProps.ref.value = node;
  }
}

const verticalWrapperStyle = computed(
  () => (wrapperProps.value.style ?? {}) as Record<string, string>,
);

function scrollHorizontalTo(index: number) {
  const el = scrollEl.value;
  if (!el || index < 0) return;
  const stride = horizontalItemStride.value;
  const targetLeft = index * stride;
  const maxLeft = Math.max(0, el.scrollWidth - el.clientWidth);
  el.scrollLeft = Math.min(targetLeft, maxLeft);
}

/** 仅由键盘导航 / 打开面板时显式调用，勿在 activeIndex 变化时自动滚（会与 mouseenter 形成反馈环） */
function scrollToActiveIndex() {
  const index = props.activeIndex;
  if (index < 0) return;
  if (props.horizontal) {
    scrollHorizontalTo(index);
    return;
  }
  scrollTo(index);
}

defineExpose({ scrollToActiveIndex });

/** 底部横条：滚轮纵向增量转为横向滚动 */
function onHorizontalWheel(event: WheelEvent) {
  if (!props.horizontal) return;
  const el = event.currentTarget as HTMLElement;
  if ((event.target as HTMLElement).closest("input, textarea")) return;
  const vertical = Math.abs(event.deltaY) >= Math.abs(event.deltaX);
  if (!vertical) return;
  event.preventDefault();
  el.scrollLeft += event.deltaY;
}
</script>

<template>
  <div class="palette-virtual-host">
    <div
      v-if="horizontal"
      ref="scrollEl"
      class="palette-virtual palette-virtual--horizontal"
      @wheel="onHorizontalWheel"
    >
      <div
        class="palette-virtual__wrapper"
        :style="{ height: `${resolvedItemHeight}px` }"
      >
        <div
          v-for="(item, index) in items"
          :key="item.id"
          class="palette-virtual__item"
          :style="{
            width: `${itemWidth}px`,
            height: `${resolvedItemHeight}px`,
          }"
          @mouseenter="emit('activeIndex', index)"
          @contextmenu="emit('activeIndex', index)"
        >
          <PaletteItemCard
            :item="item"
            :active="index === activeIndex"
            :show-source-app-icon="showSourceAppIcon"
            @select="emit('select', item)"
            @command="emit('command', item, $event)"
          />
        </div>
      </div>
    </div>

    <div
      v-else
      :ref="setScrollContainerRef"
      v-bind="containerBind"
      class="palette-virtual"
    >
      <div :style="verticalWrapperStyle" class="palette-virtual__wrapper">
        <div
          v-for="{ data: item, index } in list"
          :key="item.id"
          class="palette-virtual__item"
          :style="{ height: `${itemHeight}px` }"
          @mouseenter="emit('activeIndex', index)"
          @contextmenu="emit('activeIndex', index)"
        >
          <PaletteItemCard
            :item="item"
            :active="index === activeIndex"
            :show-source-app-icon="showSourceAppIcon"
            @select="emit('select', item)"
            @command="emit('command', item, $event)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
.palette-virtual-host {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.palette-virtual {
  flex: 1;
  min-height: 0;
  height: 100%;
  overflow: auto;

  &--horizontal {
    overflow-x: auto;
    overflow-y: hidden;

    .palette-virtual__wrapper {
      display: flex;
      flex-direction: row;
      gap: 8px;
      align-items: stretch;
      width: max-content;
    }
  }
}

.palette-virtual__wrapper {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.palette-virtual__item {
  flex-shrink: 0;
  min-height: 0;
  overflow: hidden;
  width: 100%;
}
</style>
