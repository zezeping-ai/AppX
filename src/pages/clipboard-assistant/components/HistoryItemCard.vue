<script setup lang="ts">
import { computed } from "vue";
import { Icon } from "@/components/Icon";
import {
  contrastTextOnFill,
  resolveClipboardColor,
  tintVarOfContentType,
  type ClipboardItemSummary,
} from "@/modules/clipboardAssistant";

const props = defineProps<{
  item: ClipboardItemSummary;
  showSourceAppIcon: boolean;
}>();

const emit = defineEmits<{
  detail: [];
  pin: [];
  delete: [];
}>();

const isImagePreview = computed(
  () => props.item.contentType === "image" && !!props.item.thumbUrl,
);

const useNeutralBackground = computed(
  () => props.item.contentType === "text" || props.item.contentType === "image",
);

const colorFill = computed(() => resolveClipboardColor(props.item));
const colorOnFill = computed(() =>
  colorFill.value ? contrastTextOnFill(colorFill.value) : undefined,
);
</script>

<template>
  <article
    class="history-card"
    :class="{
      'history-card--image': isImagePreview,
      'history-card--neutral-bg': useNeutralBackground,
      'history-card--color': !!colorFill,
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
    @click="emit('detail')"
  >
    <div class="history-card__accent" />
    <div class="history-card__body">
      <div class="history-card__head">
        <div
          v-if="(showSourceAppIcon && item.sourceAppIconUrl) || item.sourceAppName"
          class="history-card__source"
        >
          <img
            v-if="showSourceAppIcon && item.sourceAppIconUrl"
            :src="item.sourceAppIconUrl"
            class="history-card__app-icon"
            alt=""
          />
          <span v-if="item.sourceAppName" class="history-card__app-name">{{
            item.sourceAppName
          }}</span>
        </div>
        <span v-if="item.pinned" class="history-card__pin" title="已固定">
          <Icon icon="mdi:pin" />
        </span>
        <span v-for="badge in item.badges" :key="badge.kind" class="history-card__badge">
          {{ badge.label }}
        </span>
        <span class="history-card__time">{{ item.relativeTime }}</span>
      </div>

      <div v-if="isImagePreview" class="history-card__image-preview">
        <img :src="item.thumbUrl" class="history-card__image" alt="" loading="lazy" />
      </div>
      <div v-else class="history-card__preview-row">
        <img
          v-if="item.thumbUrl"
          :src="item.thumbUrl"
          class="history-card__thumb"
          alt=""
          loading="lazy"
        />
        <div v-if="colorFill" class="history-card__color-swatch" :style="{ background: colorFill }" />
        <p class="history-card__preview">{{ item.preview }}</p>
      </div>

      <div class="history-card__actions" @click.stop>
        <a-button type="link" size="small" @click="emit('detail')">详情</a-button>
        <a-button type="link" size="small" @click="emit('pin')">
          {{ item.pinned ? "取消固定" : "固定" }}
        </a-button>
        <a-button type="link" size="small" danger @click="emit('delete')">删除</a-button>
      </div>
    </div>
  </article>
</template>

<style scoped lang="scss">
.history-card {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 148px;
  padding: 10px 10px 8px 14px;
  border: 1px solid var(--app-border);
  border-radius: 10px;
  background: color-mix(in srgb, var(--card-tint) 12%, var(--app-surface));
  cursor: pointer;
  overflow: hidden;
  transition: border-color 0.15s ease;

  &:hover {
    border-color: color-mix(in srgb, var(--app-active-fg) 45%, var(--app-border));
  }

  &--neutral-bg {
    background: var(--app-surface);
  }

  &--color {
    .history-card__head,
    .history-card__preview {
      color: var(--color-on-fill);
    }

    .history-card__badge {
      color: var(--color-on-fill);
      background: color-mix(in srgb, var(--color-on-fill) 14%, transparent);
    }
  }
}

.history-card__accent {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: var(--source-accent, var(--card-tint));
}

.history-card__body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  flex: 1;
}

.history-card__head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--app-fg-muted);
}

.history-card__source {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}

.history-card__app-icon {
  width: 14px;
  height: 14px;
  border-radius: 3px;
  object-fit: cover;
}

.history-card__app-name {
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-card__pin {
  display: inline-flex;
  color: #d48806;
}

.history-card__badge {
  padding: 0 5px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--card-tint) 22%, transparent);
  font-size: 10px;
}

.history-card__time {
  margin-left: auto;
}

.history-card__preview-row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
  min-width: 0;
  flex: 1;
}

.history-card__thumb {
  width: 48px;
  height: 48px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
}

.history-card__color-swatch {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 1px solid rgba(0, 0, 0, 0.08);
  flex-shrink: 0;
}

.history-card__preview {
  margin: 0;
  flex: 1;
  min-width: 0;
  font-size: 13px;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 3;
  line-clamp: 3;
  overflow: hidden;
  word-break: break-word;
  color: var(--app-fg);
}

.history-card__image-preview {
  flex: 1;
  min-height: 72px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.history-card__image {
  max-width: 100%;
  max-height: 120px;
  border-radius: 6px;
  object-fit: contain;
}

.history-card__actions {
  display: flex;
  justify-content: flex-end;
  gap: 2px;
  margin-top: auto;
  border-top: 1px solid color-mix(in srgb, var(--app-border) 80%, transparent);
  padding-top: 4px;
}
</style>
