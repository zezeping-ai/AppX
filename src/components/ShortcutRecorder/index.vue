<script setup lang="tsx">
import { Icon } from "@iconify/vue";
import { useEventListener } from "@vueuse/core";
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { captureShortcut, formatShortcutLabel } from "@/shared/shortcut";

const value = defineModel<string>("value", { default: "" });

const props = defineProps<{
  /** 录制期间回调，可用于暂停全局快捷键监听等 */
  onRecordingChange?: (recording: boolean) => void | Promise<void>;
}>();

const recording = ref(false);

watch(recording, (active) => {
  void props.onRecordingChange?.(active);
});

onBeforeUnmount(() => {
  void props.onRecordingChange?.(false);
});

const displayText = computed(() => {
  if (recording.value) return "请按下快捷键…";
  if (value.value) return formatShortcutLabel(value.value);
  return "点击录制快捷键";
});

function startRecording() {
  recording.value = true;
}

function clear() {
  value.value = "";
  recording.value = false;
}

useEventListener(
  window,
  "keydown",
  (event) => {
    if (!recording.value) return;

    // 忽略长按连发，避免一次录制产生多次判定
    if (event.repeat) return;

    if (event.key === "Escape") {
      event.preventDefault();
      recording.value = false;
      return;
    }

    const captured = captureShortcut(event);
    if (!captured) return;

    event.preventDefault();
    event.stopPropagation();
    value.value = captured;
    recording.value = false;
  },
  { capture: true },
);
</script>

<template>
  <div class="shortcut-recorder">
    <button
      type="button"
      class="shortcut-recorder__trigger"
      :class="{ 'shortcut-recorder__trigger--recording': recording }"
      @click="startRecording"
    >
      <span class="shortcut-recorder__label">{{ displayText }}</span>
    </button>
    <a-button
      v-if="value"
      type="text"
      size="small"
      class="shortcut-recorder__clear"
      aria-label="清除快捷键"
      @click="clear"
    >
      <Icon icon="mdi:close" width="14" height="14" />
    </a-button>
  </div>
</template>

<style scoped lang="scss">
.shortcut-recorder {
  display: flex;
  align-items: center;
  gap: 4px;
}

.shortcut-recorder__trigger {
  flex: 1;
  min-width: 0;
  height: 24px;
  padding: 0 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  background: #fff;
  color: rgba(0, 0, 0, 0.88);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;

  &:hover {
    border-color: #4096ff;
  }

  &--recording {
    border-color: #1677ff;
    box-shadow: 0 0 0 2px rgba(22, 119, 255, 0.15);
    color: #1677ff;
  }
}

.shortcut-recorder__label {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shortcut-recorder__clear {
  flex-shrink: 0;
  padding: 0 4px;
}

[data-theme="dark"] .shortcut-recorder__trigger {
  border-color: #424242;
  background: #141414;
  color: rgba(255, 255, 255, 0.85);

  &:hover {
    border-color: #1668dc;
  }

  &--recording {
    border-color: #1668dc;
    box-shadow: 0 0 0 2px rgba(22, 104, 220, 0.2);
    color: #69b1ff;
  }
}
</style>
