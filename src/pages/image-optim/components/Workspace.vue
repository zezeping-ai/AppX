<script setup lang="ts">
import { computed, h, onMounted, onUnmounted, ref } from "vue";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Icon } from "@iconify/vue";
import { Button as AButton, message } from "ant-design-vue";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  useImageOptimQueue,
  type QueueItem,
} from "@/pages/image-optim/composables/useImageOptimQueue";
import { formatBytes, formatSaved } from "@/pages/image-optim/format";
import { getErrorMessage } from "@/shared/error";

const {
  items,
  settings,
  settingsLoaded,
  running,
  totals,
  setLossyEnabled,
  addDroppedOrPicked,
  onPickFiles,
  onPickFolder,
  removeItem,
  clearAll,
  clearFinished,
  stopQueued,
} = useImageOptimQueue();

const dragging = ref(false);
let unlistenDragDrop: UnlistenFn | null = null;

function statusIcon(item: QueueItem): string {
  switch (item.status) {
    case "queued":
      return "mdi:clock-outline";
    case "running":
      return "mdi:loading";
    case "optimized":
      return "mdi:check-circle";
    case "alreadyOptimal":
      return "mdi:minus-circle-outline";
    case "error":
      return "mdi:alert-circle";
    default:
      return "mdi:help-circle-outline";
  }
}

function statusColor(item: QueueItem): string {
  switch (item.status) {
    case "optimized":
      return "var(--ant-color-success)";
    case "alreadyOptimal":
      return "var(--ant-color-text-secondary)";
    case "error":
      return "var(--ant-color-error)";
    case "running":
      return "var(--ant-color-primary)";
    default:
      return "var(--ant-color-text-tertiary)";
  }
}

async function reveal(path: string) {
  try {
    await revealItemInDir(path);
  } catch (error) {
    message.error(getErrorMessage(error, "无法在文件管理器中显示"));
  }
}

const columns = computed(() => [
  {
    title: "",
    key: "status",
    width: 40,
    customRender: ({ record }: { record: QueueItem }) =>
      h(Icon, {
        icon: statusIcon(record),
        width: 18,
        height: 18,
        class: record.status === "running" ? "is-spin" : undefined,
        style: { color: statusColor(record) },
      }),
  },
  {
    title: "文件",
    key: "name",
    ellipsis: true,
    customRender: ({ record }: { record: QueueItem }) =>
      h("div", [
        h(
          "button",
          {
            type: "button",
            class: "image-optim__name",
            onClick: () => void reveal(record.path),
          },
          record.name,
        ),
        record.error ? h("div", { class: "image-optim__error" }, record.error) : null,
      ]),
  },
  {
    title: "原大小",
    key: "before",
    width: 100,
    align: "right" as const,
    customRender: ({ record }: { record: QueueItem }) =>
      record.status === "queued" || record.status === "running"
        ? "—"
        : formatBytes(record.beforeBytes),
  },
  {
    title: "新大小",
    key: "after",
    width: 100,
    align: "right" as const,
    customRender: ({ record }: { record: QueueItem }) =>
      record.status === "optimized" || record.status === "alreadyOptimal"
        ? formatBytes(record.afterBytes)
        : "—",
  },
  {
    title: "节省",
    key: "saved",
    width: 80,
    align: "right" as const,
    customRender: ({ record }: { record: QueueItem }) => formatSaved(record),
  },
  {
    title: "",
    key: "actions",
    width: 64,
    align: "center" as const,
    customRender: ({ record }: { record: QueueItem }) =>
      h(
        AButton,
        {
          type: "text",
          size: "small",
          disabled: record.status === "running",
          onClick: () => removeItem(record.id),
        },
        () => h(Icon, { icon: "mdi:close", width: 16, height: 16 }),
      ),
  },
]);

const subtitle = computed(() =>
  settings.value.lossyEnabled
    ? "有损模式已开启 · PNG / JPEG / GIF / SVG"
    : "默认无损 · PNG / JPEG / GIF / SVG",
);

const lossyTooltip = computed(() => {
  if (!settings.value.lossyEnabled) {
    return "开启后可用更激进压缩，画质可能略降；质量可在偏好设置中调整";
  }
  const { jpegMaxQuality, pngMinQuality, gifQuality } = settings.value;
  return `已开启有损压缩\nJPEG 质量上限 ${jpegMaxQuality}\nPNG 质量下限 ${pngMinQuality}\nGIF 质量 ${gifQuality}\n可在偏好设置中调整`;
});

onMounted(async () => {
  try {
    unlistenDragDrop = await getCurrentWebviewWindow().onDragDropEvent((event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        dragging.value = true;
        return;
      }
      if (event.payload.type === "leave") {
        dragging.value = false;
        return;
      }
      if (event.payload.type === "drop") {
        dragging.value = false;
        void addDroppedOrPicked(event.payload.paths);
      }
    });
  } catch (error) {
    console.warn("register drag drop listener failed", error);
  }
});

onUnmounted(() => {
  unlistenDragDrop?.();
  unlistenDragDrop = null;
});
</script>

<template>
  <div class="image-optim" :class="{ 'is-dragging': dragging }">
    <div class="image-optim__header">
      <div class="image-optim__title-row">
        <h1 class="image-optim__title">图片优化</h1>
        <p class="image-optim__subtitle">{{ subtitle }}</p>
      </div>
      <div class="image-optim__toolbar">
        <span class="image-optim__lossy">
          有损压缩
          <a-tooltip :title="lossyTooltip" :overlay-style="{ whiteSpace: 'pre-line' }">
            <a-switch
              size="small"
              :checked="settings.lossyEnabled"
              :disabled="!settingsLoaded"
              @change="(v: boolean) => void setLossyEnabled(v)"
            />
          </a-tooltip>
        </span>
        <a-space wrap>
          <a-button v-if="items.length" @click="onPickFiles">添加文件</a-button>
          <a-button v-if="items.length" @click="onPickFolder">添加文件夹</a-button>
          <a-button v-if="totals.pending" @click="stopQueued">停止排队</a-button>
          <a-button :disabled="!items.length" @click="clearFinished">清除已完成</a-button>
          <a-button danger :disabled="!items.length" @click="clearAll">清空</a-button>
        </a-space>
      </div>
    </div>

    <div v-if="!items.length" class="image-optim__empty">
      <Icon icon="mdi:file-image-outline" width="48" height="48" />
      <p>拖入图片或文件夹到此处</p>
      <p class="image-optim__empty-hint">
        支持 png、jpg、jpeg、gif、svg · 仅当更小才原地覆盖
      </p>
      <a-space>
        <a-button type="primary" @click="onPickFiles">选择文件</a-button>
        <a-button @click="onPickFolder">选择文件夹</a-button>
      </a-space>
    </div>

    <div v-else class="image-optim__table-wrap">
      <a-table
        size="small"
        :pagination="false"
        :columns="columns"
        :data-source="items"
        :row-key="(r: QueueItem) => r.id"
        :scroll="{ y: '100%' }"
      />
    </div>

    <div class="image-optim__footer">
      <span>
        {{ totals.done }}/{{ totals.count }}
        <template v-if="running"> · 处理中…</template>
      </span>
      <span v-if="totals.done">
        {{ formatBytes(totals.before) }} → {{ formatBytes(totals.after) }}
        · 节省 {{ formatBytes(totals.saved) }}
      </span>
    </div>
  </div>
</template>

<style scoped lang="scss">
.image-optim {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  padding: 16px 20px 12px;
  gap: 12px;

  &.is-dragging {
    outline: 2px dashed var(--ant-color-primary);
    outline-offset: -8px;
  }

  &__header {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex-shrink: 0;
    padding-bottom: 8px;
  }

  &__title-row {
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  &__title {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    flex-shrink: 0;
  }

  &__subtitle {
    margin: 0;
    font-size: 12px;
    color: var(--ant-color-text-secondary);
  }

  &__toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  &__lossy {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  &__empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--ant-color-text-secondary);
    border: 1px dashed var(--ant-color-border);
    border-radius: 8px;
  }

  &__empty-hint {
    margin: 0 0 8px;
    font-size: 12px;
  }

  &__table-wrap {
    flex: 1;
    min-height: 0;
    overflow: hidden;

    :deep(.ant-table-wrapper),
    :deep(.ant-spin-nested-loading),
    :deep(.ant-spin-container),
    :deep(.ant-table),
    :deep(.ant-table-container) {
      height: 100%;
    }

    :deep(.ant-table-body) {
      overflow-y: auto !important;
    }
  }

  &__footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    font-size: 12px;
    color: var(--ant-color-text-secondary);
    padding-top: 4px;
    border-top: 1px solid var(--ant-color-border-secondary);
    flex-shrink: 0;
  }
}

:deep(.image-optim__name) {
  background: none;
  border: 0;
  padding: 0;
  color: var(--ant-color-text);
  cursor: pointer;
  text-align: left;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;

  &:hover {
    color: var(--ant-color-primary);
  }
}

:deep(.image-optim__error) {
  font-size: 11px;
  color: var(--ant-color-error);
}

:deep(.is-spin) {
  animation: image-optim-spin 0.8s linear infinite;
}

@keyframes image-optim-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
