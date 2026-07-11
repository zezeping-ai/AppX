<script setup lang="ts">
import { computed, ref, watchEffect } from "vue";
import { useElementSize } from "@vueuse/core";
import { Icon } from "@iconify/vue";
import type { MenuProps } from "ant-design-vue";
import {
  contrastTextOnFill,
  resolveClipboardColor,
  tintVarOfContentType,
  type ClipboardItemSummary,
} from "@/modules/clipboardAssistant";

export type PaletteItemCommand = "paste" | "copy" | "pin" | "delete";

const props = defineProps<{
  item: ClipboardItemSummary;
  active: boolean;
  showSourceAppIcon: boolean;
}>();

const emit = defineEmits<{
  select: [];
  command: [PaletteItemCommand];
}>();

const contextMenuItems = computed<MenuProps["items"]>(() => [
  { key: "paste", label: "粘贴" },
  { key: "copy", label: "拷贝" },
  { type: "divider" },
  { key: "pin", label: props.item.pinned ? "取消固定" : "固定" },
  { type: "divider" },
  { key: "delete", label: "删除", danger: true },
]);

function onContextMenuClick({ key }: { key: string | number }) {
  emit("command", String(key) as PaletteItemCommand);
}

/** 头部已有类型徽章时，内容区只展示大图预览 */
const isImagePreview = computed(
  () => props.item.contentType === "image" && !!props.item.thumbUrl,
);

/** 仅颜色类条目保留类型着色背景，文本/图片等用中性底色 */
const useNeutralBackground = computed(
  () => props.item.contentType === "text" || props.item.contentType === "image",
);

const colorFill = computed(() => resolveClipboardColor(props.item));
const colorOnFill = computed(() =>
  colorFill.value ? contrastTextOnFill(colorFill.value) : undefined,
);

const previewClipRef = ref<HTMLElement | null>(null);
const previewRef = ref<HTMLElement | null>(null);
const { height: clipHeight } = useElementSize(previewClipRef);

/** line-clamp 需匹配实际可用高度，flex 容器里固定行数常无法触发省略号。 */
watchEffect(() => {
  if (isImagePreview.value) return;
  const preview = previewRef.value;
  if (!preview || clipHeight.value <= 0) return;
  const lineHeight = parseFloat(getComputedStyle(preview).lineHeight) || 18;
  const lines = Math.max(1, Math.floor(clipHeight.value / lineHeight));
  preview.style.setProperty("-webkit-line-clamp", String(lines));
  preview.style.setProperty("line-clamp", String(lines));
});
</script>

<template>
  <a-dropdown :trigger="['contextmenu']" class="palette-card-dropdown">
    <article
      class="palette-card"
      :class="{
        'palette-card--active': active,
        'palette-card--image': isImagePreview,
        'palette-card--neutral-bg': useNeutralBackground,
        'palette-card--color': !!colorFill,
      }"
      :style="{
        '--card-tint': `var(${tintVarOfContentType(item.contentType)})`,
        '--source-accent': item.accentColor,
        ...(colorFill
          ? {
              '--color-fill': colorFill,
              '--color-on-fill': colorOnFill,
              background: colorFill,
            }
          : {}),
      }"
      @click="emit('select')"
    >
    <div class="palette-card__accent" />
    <div class="palette-card__body">
      <div class="palette-card__head">
        <div v-if="showSourceAppIcon && item.sourceAppIconUrl || item.sourceAppName" class="palette-card__source">
          <img
            v-if="showSourceAppIcon && item.sourceAppIconUrl"
            :src="item.sourceAppIconUrl"
            class="palette-card__app-icon"
            alt=""
          />
          <span v-if="item.sourceAppName" class="palette-card__app-name">{{ item.sourceAppName }}</span>
        </div>
        <span v-if="item.pinned" class="palette-card__pin" title="已固定">
          <Icon icon="mdi:pin" aria-hidden="true" />
        </span>
        <span v-for="badge in item.badges" :key="badge.kind" class="palette-card__badge">
          {{ badge.label }}
        </span>
        <span v-if="item.charCount" class="palette-card__chars">{{ item.charCount }} 字</span>
        <span class="palette-card__time">{{ item.relativeTime }}</span>
      </div>
      <div v-if="isImagePreview" class="palette-card__image-preview">
        <img
          :src="item.thumbUrl"
          class="palette-card__image"
          alt=""
        />
      </div>
      <div v-else class="palette-card__preview-row">
        <img
          v-if="item.thumbUrl"
          :src="item.thumbUrl"
          class="palette-card__thumb"
          alt=""
        />
        <div class="palette-card__text">
          <div ref="previewClipRef" class="palette-card__preview-clip">
            <p ref="previewRef" class="palette-card__preview">{{ item.preview }}</p>
          </div>
        </div>
      </div>
    </div>
    </article>
    <template #overlay>
      <a-menu :items="contextMenuItems" @click="onContextMenuClick" />
    </template>
  </a-dropdown>
</template>

<style scoped lang="scss">
.palette-card-dropdown {
  display: block;
  height: 100%;
  min-height: 0;
}

.palette-card {
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  padding: 10px 10px 10px 14px;
  border: 1px solid var(--app-border);
  border-radius: 10px;
  background: color-mix(in srgb, var(--card-tint) 12%, var(--app-surface));
  cursor: pointer;
  overflow: hidden;

  &--active {
    border-color: var(--app-active-fg);
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--app-active-fg) 35%, transparent);
  }

  &--neutral-bg {
    background: var(--app-surface);
  }

  &--color {
    .palette-card__head,
    .palette-card__preview {
      color: var(--color-on-fill);
    }

    .palette-card__badge {
      color: var(--color-on-fill);
      background: color-mix(in srgb, var(--color-on-fill) 14%, transparent);
    }
  }

  &--image {
    padding-top: 6px;
    padding-bottom: 6px;
    padding-right: 8px;

    .palette-card__head {
      margin-bottom: 2px;
      flex-shrink: 0;
    }
  }
}

.palette-card__accent {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: var(--source-accent, var(--card-tint));
}

.palette-card__body {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  height: 100%;
}

.palette-card__head {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
  font-size: 11px;
  color: var(--app-fg-muted);
}

.palette-card__source {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  max-width: 45%;
  flex-shrink: 1;
}

.palette-card__app-icon {
  width: 14px;
  height: 14px;
  border-radius: 3px;
  object-fit: contain;
  flex-shrink: 0;
}

.palette-card__app-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.palette-card__pin {
  color: #f59e0b;
}

.palette-card__badge {
  padding: 0 6px;
  border-radius: 999px;
  background: var(--app-surface-muted);
}

.palette-card__time {
  margin-left: auto;
  flex-shrink: 0;
}

.palette-card__chars {
  flex-shrink: 0;
}

.palette-card__preview-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
  flex: 1;
  min-height: 0;
}

.palette-card__image-preview {
  flex: 1;
  min-height: 0;
  min-width: 0;
  width: 100%;
  overflow: hidden;
}

.palette-card__image {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: contain;
  object-position: center;
  border-radius: 4px;
}

.palette-card__thumb {
  width: 40px;
  height: 40px;
  align-self: flex-start;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
  background: var(--app-surface-muted);
}

.palette-card__text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  flex: 1;
  overflow: hidden;
}

.palette-card__preview-clip {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.palette-card__preview {
  margin: 0;
  overflow: hidden;
  line-height: 1.35;
  font-size: 13px;
  word-break: break-word;
  white-space: pre-line;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  text-overflow: ellipsis;
}
</style>
