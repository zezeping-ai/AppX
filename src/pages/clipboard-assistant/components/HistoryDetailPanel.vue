<script setup lang="ts">
import { computed, ref } from "vue";
import { Icon } from "@/components/Icon";
import { message } from "ant-design-vue";
import {
  applyItem,
  contrastTextOnFill,
  extraBadgesOf,
  formatItemCreatedAt,
  iconOfContentType,
  itemHasRichFormat,
  labelOfContentType,
  resolveClipboardColor,
  tintVarOfContentType,
  type ClipboardItemSummary,
  type GetContentResult,
} from "@/modules/clipboardAssistant";
import { getErrorMessage } from "@/shared/error";

const props = defineProps<{
  item: ClipboardItemSummary;
  detail: GetContentResult;
  showSourceAppIcon: boolean;
}>();

const emit = defineEmits<{
  pin: [];
  delete: [];
}>();

const copying = ref(false);

const typeLabel = computed(() => labelOfContentType(props.item.contentType));
const typeIcon = computed(() => iconOfContentType(props.item.contentType));
const colorFill = computed(() => resolveClipboardColor(props.item));
const colorOnFill = computed(() =>
  colorFill.value ? contrastTextOnFill(colorFill.value) : undefined,
);

const createdLabel = computed(() => formatItemCreatedAt(props.item.createdAt));
const canCopyRich = computed(() => itemHasRichFormat(props.item));
const extraBadges = computed(() => extraBadgesOf(props.item));

const isImage = computed(
  () => props.item.contentType === "image" && !!props.item.thumbUrl,
);

const isFile = computed(() => props.item.contentType === "file");

async function onCopy(format: "plain" | "rich") {
  copying.value = true;
  try {
    await applyItem(props.item.id, "copy", format);
    message.success(format === "rich" ? "已复制富文本" : "已复制");
  } catch (error) {
    message.error(getErrorMessage(error, "复制失败"));
  } finally {
    copying.value = false;
  }
}
</script>

<template>
  <div
    class="history-detail"
    :style="{
      '--card-tint': `var(${tintVarOfContentType(item.contentType)})`,
      '--source-accent': item.accentColor,
    }"
  >
    <header class="history-detail__meta">
      <div class="history-detail__source">
        <img
          v-if="showSourceAppIcon && item.sourceAppIconUrl"
          :src="item.sourceAppIconUrl"
          class="history-detail__app-icon"
          alt=""
        />
        <span class="history-detail__app-name">
          {{ item.sourceAppName || "未知来源" }}
        </span>
      </div>
      <div class="history-detail__time" :title="createdLabel || undefined">
        <span>{{ item.relativeTime }}</span>
        <template v-if="createdLabel">
          <span class="history-detail__time-sep">·</span>
          <span class="history-detail__time-abs">{{ createdLabel }}</span>
        </template>
      </div>
    </header>

    <div class="history-detail__tags">
      <a-tag class="history-detail__type">
        <span class="history-detail__type-inner">
          <Icon :icon="typeIcon" :size="14" />
          {{ typeLabel }}
        </span>
      </a-tag>
      <a-tag v-if="item.pinned" color="gold">已固定</a-tag>
      <a-tag v-for="badge in extraBadges" :key="badge.kind">{{ badge.label }}</a-tag>
      <a-tag v-if="item.charCount">{{ item.charCount }} 字</a-tag>
      <a-tag class="history-detail__id">#{{ item.id }}</a-tag>
    </div>

    <div class="history-detail__actions">
      <a-button type="primary" size="small" :loading="copying" @click="onCopy('plain')">
        <template #icon><Icon icon="mdi:content-copy" /></template>
        复制
      </a-button>
      <a-button
        v-if="canCopyRich"
        size="small"
        :loading="copying"
        @click="onCopy('rich')"
      >
        <template #icon><Icon icon="mdi:file-document-outline" /></template>
        复制富文本
      </a-button>
      <a-button size="small" @click="emit('pin')">
        <template #icon>
          <Icon :icon="item.pinned ? 'mdi:pin-off' : 'mdi:pin'" />
        </template>
        {{ item.pinned ? "取消固定" : "固定" }}
      </a-button>
      <a-button size="small" danger @click="emit('delete')">
        <template #icon><Icon icon="mdi:delete-outline" /></template>
        删除
      </a-button>
    </div>

    <section class="history-detail__body">
      <div
        v-if="colorFill"
        class="history-detail__color"
        :style="{ background: colorFill, color: colorOnFill }"
      >
        <span class="history-detail__color-value">{{ item.preview }}</span>
      </div>

      <div v-else-if="isImage" class="history-detail__image-wrap">
        <img :src="item.thumbUrl" class="history-detail__image" alt="" />
      </div>

      <ul v-else-if="isFile && detail.filePaths?.length" class="history-detail__files">
        <li v-for="path in detail.filePaths" :key="path">
          <Icon icon="mdi:file-outline" class="history-detail__file-icon" />
          <span>{{ path }}</span>
        </li>
      </ul>

      <pre v-else-if="detail.text" class="history-detail__text">{{ detail.text }}</pre>

      <a-empty
        v-else-if="detail.hasBlob"
        description="二进制内容（无法以文本预览）"
        :image-style="{ height: '48px' }"
      />
      <a-empty v-else description="无内容" :image-style="{ height: '48px' }" />
    </section>
  </div>
</template>

<style scoped lang="scss">
.history-detail {
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 100%;
}

.history-detail__meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 8px 16px;
}

.history-detail__source {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.history-detail__app-icon {
  width: 22px;
  height: 22px;
  border-radius: 5px;
  object-fit: cover;
  flex-shrink: 0;
}

.history-detail__app-name {
  font-size: 14px;
  font-weight: 520;
  color: var(--app-fg);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-detail__time {
  display: inline-flex;
  align-items: center;
  flex-shrink: 0;
  font-size: 12px;
  color: var(--app-fg-muted);
  white-space: nowrap;
}

.history-detail__time-sep {
  margin: 0 4px;
}

.history-detail__time-abs {
  font-variant-numeric: tabular-nums;
}

.history-detail__tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  align-items: center;
}

.history-detail__type-inner {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.history-detail__id {
  margin-left: auto;
  opacity: 0.75;
}

.history-detail__actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--app-border, rgba(0, 0, 0, 0.06));
}

.history-detail__body {
  flex: 1;
  min-height: 0;
}

.history-detail__color {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 120px;
  border-radius: 10px;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 18px;
  font-weight: 600;
}

.history-detail__image-wrap {
  display: flex;
  justify-content: center;
  padding: 8px;
  border-radius: 10px;
  background: color-mix(in srgb, var(--card-tint) 10%, var(--app-surface-muted, #f5f5f5));
}

.history-detail__image {
  max-width: 100%;
  max-height: min(420px, 55vh);
  object-fit: contain;
  border-radius: 8px;
}

.history-detail__files {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.history-detail__files li {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 10px 12px;
  border-radius: 8px;
  background: var(--app-surface-muted, rgba(0, 0, 0, 0.03));
  font-size: 12px;
  word-break: break-all;
  line-height: 1.45;
}

.history-detail__file-icon {
  flex-shrink: 0;
  margin-top: 1px;
  color: var(--app-fg-muted);
}

.history-detail__text {
  margin: 0;
  padding: 14px 16px;
  border-radius: 10px;
  background: var(--app-surface-muted, rgba(0, 0, 0, 0.03));
  border: 1px solid var(--app-border, rgba(0, 0, 0, 0.06));
  white-space: pre-wrap;
  word-break: break-word;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 13px;
  line-height: 1.55;
  color: var(--app-fg);
  max-height: min(60vh, 560px);
  overflow: auto;
}
</style>
