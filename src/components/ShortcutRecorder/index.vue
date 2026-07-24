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

function clear(event: MouseEvent) {
  event.stopPropagation();
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
  <div
    class="shortcut-recorder"
    :class="{ 'shortcut-recorder--recording': recording }"
    @click="startRecording"
  >
    <span class="shortcut-recorder__label">{{ displayText }}</span>
    <button
      v-if="value && !recording"
      type="button"
      class="shortcut-recorder__clear"
      aria-label="清除快捷键"
      @click="clear"
    >
      <Icon icon="mdi:close" width="14" height="14" />
    </button>
  </div>
</template>

<style scoped lang="scss">
.shortcut-recorder {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  box-sizing: border-box;
  width: fit-content;
  max-width: 100%;
  min-width: 7.5rem;
  height: 24px;
  padding: 0 4px 0 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  background: #fff;
  color: rgba(0, 0, 0, 0.88);
  font-size: 12px;
  cursor: pointer;
  transition:
    border-color 0.2s,
    box-shadow 0.2s;

  &:hover {
    border-color: #4096ff;

    .shortcut-recorder__clear {
      opacity: 0.65;
    }
  }

  &--recording {
    border-color: #1677ff;
    box-shadow: 0 0 0 2px rgba(22, 119, 255, 0.15);
    color: #1677ff;
  }
}

.shortcut-recorder__label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
  line-height: 22px;
}

.shortcut-recorder__clear {
  display: inline-flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  margin-left: 2px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: rgba(0, 0, 0, 0.45);
  opacity: 0.45;
  cursor: pointer;
  transition:
    opacity 0.15s,
    color 0.15s,
    background 0.15s;

  &:hover {
    opacity: 1 !important;
    color: rgba(0, 0, 0, 0.75);
    background: rgba(0, 0, 0, 0.06);
  }
}

[data-theme="dark"] .shortcut-recorder {
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

[data-theme="dark"] .shortcut-recorder__clear {
  color: rgba(255, 255, 255, 0.55);

  &:hover {
    color: rgba(255, 255, 255, 0.85);
    background: rgba(255, 255, 255, 0.08);
  }
}
</style>
