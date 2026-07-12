<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { message } from "ant-design-vue";
import ShortcutRecorder from "@/components/ShortcutRecorder/index.vue";
import {
  getSettings,
  getStatus,
  saveSettings,
  normalizePaletteLayout,
  type ClipboardAssistantSettings,
  type ClipboardAssistantStatus,
  type PaletteLayout,
} from "@/modules/clipboardAssistant";
import { setGlobalShortcutsPaused } from "@/modules/globalShortcut";
import { formatShortcutLabel, normalizeGlobalShortcut } from "@/shared/shortcut";
import { getErrorMessage } from "@/shared/error";

const loading = ref(false);
const saving = ref(false);
const status = ref<ClipboardAssistantStatus | null>(null);
/** 后端完整配置快照，保存时合并以免覆盖 UI 未暴露字段 */
const loadedSettings = ref<ClipboardAssistantSettings | null>(null);

const enabled = ref(true);
const monitoringEnabled = ref(true);
const paletteEnabled = ref(true);
const paletteShortcut = ref("CommandOrControl+Shift+V");
const maxHistoryItems = ref(500);
const paletteLayout = ref<PaletteLayout>("bottomPanel");
const paletteMaxItems = ref(80);
const autoHideOnPaste = ref(true);
const clearOnLock = ref(false);

let lastPersistedShortcut = paletteShortcut.value;

const featuresActive = computed(() => enabled.value);
const paletteActive = computed(() => featuresActive.value && paletteEnabled.value);
const paletteShortcutLabel = computed(() => formatShortcutLabel(paletteShortcut.value));

function applySettings(settings: ClipboardAssistantSettings) {
  loadedSettings.value = settings;
  enabled.value = settings.enabled;
  monitoringEnabled.value = settings.monitoringEnabled;
  paletteEnabled.value = settings.paletteEnabled;
  paletteShortcut.value = settings.paletteShortcut;
  maxHistoryItems.value = settings.maxHistoryItems;
  paletteLayout.value = normalizePaletteLayout(settings.paletteLayout);
  paletteMaxItems.value = settings.paletteMaxItems;
  autoHideOnPaste.value = settings.autoHideOnPaste;
  clearOnLock.value = settings.clearOnLock;
  lastPersistedShortcut = settings.paletteShortcut;
}

function currentInput(): ClipboardAssistantSettings {
  const base = loadedSettings.value;
  return {
    ...(base ?? {}),
    enabled: enabled.value,
    monitoringEnabled: monitoringEnabled.value,
    paletteEnabled: paletteEnabled.value,
    paletteShortcut: normalizeGlobalShortcut(paletteShortcut.value) ?? paletteShortcut.value,
    maxHistoryItems: maxHistoryItems.value,
    paletteLayout: paletteLayout.value,
    paletteMaxItems: paletteMaxItems.value,
    autoHideOnPaste: autoHideOnPaste.value,
    clearOnLock: clearOnLock.value,
  } as ClipboardAssistantSettings;
}

async function refresh() {
  loading.value = true;
  try {
    const [settings, stat] = await Promise.all([getSettings(), getStatus()]);
    applySettings(settings);
    status.value = stat;
  } catch (error) {
    message.error(getErrorMessage(error, "加载设置失败"));
  } finally {
    loading.value = false;
  }
}

async function persistSettings(successMessage: string) {
  const snapshot = loadedSettings.value;
  saving.value = true;
  try {
    await saveSettings(currentInput());
    message.success(successMessage);
    await refresh();
  } catch (error) {
    if (snapshot) applySettings(snapshot);
    message.error(getErrorMessage(error, "保存失败"));
  } finally {
    saving.value = false;
  }
}

async function onToggleEnabled(checked: boolean) {
  enabled.value = checked;
  await persistSettings(checked ? "已启用剪切助手" : "已停用剪切助手");
}

async function onToggleMonitoring(checked: boolean) {
  monitoringEnabled.value = checked;
  await persistSettings(checked ? "已启用剪贴板监听" : "已停用剪贴板监听");
}

async function onTogglePalette(checked: boolean) {
  paletteEnabled.value = checked;
  await persistSettings(checked ? "已启用剪切助手浮层" : "已停用剪切助手浮层");
}

async function onPaletteShortcutChange(shortcut: string) {
  if (saving.value || loading.value) return;
  const normalized = normalizeGlobalShortcut(shortcut);
  if (!normalized || normalized === lastPersistedShortcut) return;
  paletteShortcut.value = normalized;
  await persistSettings(`呼出快捷键已设为 ${formatShortcutLabel(normalized)}`);
}

async function onMaxHistoryItemsChange(value: number | string | null) {
  if (saving.value || loading.value || !featuresActive.value) return;
  const next = typeof value === "number" ? value : maxHistoryItems.value;
  if (next === loadedSettings.value?.maxHistoryItems) return;
  maxHistoryItems.value = next;
  await persistSettings("历史条数上限已更新");
}

async function onPaletteLayoutChange(event: { target: { value?: unknown } }) {
  if (saving.value || loading.value || !paletteActive.value) return;
  const next = event.target.value;
  if (
    next !== "topPanel" &&
    next !== "bottomPanel" &&
    next !== "leftPanel" &&
    next !== "rightPanel"
  ) {
    return;
  }
  if (next === paletteLayout.value) return;
  paletteLayout.value = next;
  await persistSettings("浮层布局已更新");
}

async function onPaletteMaxItemsChange(value: number | string | null) {
  if (saving.value || loading.value || !paletteActive.value) return;
  const next = typeof value === "number" ? value : paletteMaxItems.value;
  if (next === loadedSettings.value?.paletteMaxItems) return;
  paletteMaxItems.value = next;
  await persistSettings("浮层最大条目已更新");
}

async function onToggleAutoHideOnPaste(checked: boolean) {
  autoHideOnPaste.value = checked;
  await persistSettings(checked ? "已开启：粘贴后自动关闭浮层" : "已关闭：粘贴后自动关闭浮层");
}

async function onToggleClearOnLock(checked: boolean) {
  clearOnLock.value = checked;
  await persistSettings(checked ? "已开启：锁定时清空未固定" : "已关闭：锁定时清空未固定");
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <a-spin :spinning="loading">
    <div class="clipboard-settings">
      <a-card title="全局功能" size="small" class="clipboard-settings__card">
        <a-space direction="vertical" size="middle" class="w-full">
          <a-checkbox
            :checked="enabled"
            :disabled="saving || loading"
            @update:checked="onToggleEnabled"
          >
            启用剪切助手
          </a-checkbox>
          <a-typography-text type="secondary" class="clipboard-settings__desc">
            关闭后，剪贴板监听与浮层快捷键将全部停用；须先解锁应用后才会生效。子项开关保留配置，重新开启总开关后按各自状态恢复。
          </a-typography-text>

          <div
            class="clipboard-settings__switches app-surface-muted"
            :class="{ 'clipboard-settings__switches--disabled': !featuresActive }"
          >
            <a-checkbox
              :checked="monitoringEnabled"
              :disabled="!featuresActive || saving || loading"
              @update:checked="onToggleMonitoring"
            >
              监听剪贴板并记录历史
            </a-checkbox>
            <a-checkbox
              :checked="paletteEnabled"
              :disabled="!featuresActive || saving || loading"
              @update:checked="onTogglePalette"
            >
              启用浮层快捷键（默认 {{ paletteShortcutLabel }}）
            </a-checkbox>
          </div>
        </a-space>
      </a-card>

      <a-card title="历史与浮层" size="small" class="clipboard-settings__card">
        <a-form layout="vertical">
          <a-form-item label="历史条数上限">
            <a-input-number
              :value="maxHistoryItems"
              :min="50"
              :max="5000"
              :disabled="!featuresActive || saving || loading"
              style="width: 100%"
              @update:value="onMaxHistoryItemsChange"
            />
          </a-form-item>
          <a-form-item label="呼出快捷键">
            <ShortcutRecorder
              v-model:value="paletteShortcut"
              :on-recording-change="setGlobalShortcutsPaused"
              @update:value="onPaletteShortcutChange"
            />
          </a-form-item>
          <a-form-item label="浮层布局">
            <a-radio-group
              :value="paletteLayout"
              :disabled="!paletteActive || saving || loading"
              @change="onPaletteLayoutChange"
            >
              <a-radio value="topPanel">顶部</a-radio>
              <a-radio value="bottomPanel">底部</a-radio>
              <a-radio value="leftPanel">左侧</a-radio>
              <a-radio value="rightPanel">右侧</a-radio>
            </a-radio-group>
          </a-form-item>
          <a-form-item label="浮层最大条目">
            <a-input-number
              :value="paletteMaxItems"
              :min="20"
              :max="200"
              :disabled="!paletteActive || saving || loading"
              style="width: 100%"
              @update:value="onPaletteMaxItemsChange"
            />
          </a-form-item>
        </a-form>
      </a-card>

      <a-card title="行为与隐私" size="small" class="clipboard-settings__card">
        <a-form layout="vertical">
          <a-form-item label="粘贴后自动关闭浮层">
            <a-switch
              :checked="autoHideOnPaste"
              :disabled="!featuresActive || saving || loading"
              @update:checked="onToggleAutoHideOnPaste"
            />
          </a-form-item>
          <a-form-item label="锁定时清空未固定">
            <a-switch
              :checked="clearOnLock"
              :disabled="!featuresActive || saving || loading"
              @update:checked="onToggleClearOnLock"
            />
          </a-form-item>
        </a-form>
      </a-card>

      <a-card v-if="status" title="状态" size="small" class="clipboard-settings__card">
        <a-descriptions :column="1" size="small">
          <a-descriptions-item label="功能">
            {{ status.paletteActive || status.monitoringActive ? "运行中" : "已停用" }}
          </a-descriptions-item>
          <a-descriptions-item label="监听">
            {{ status.monitoringActive ? "运行中" : "已暂停" }}
          </a-descriptions-item>
          <a-descriptions-item label="浮层快捷键">
            {{ formatShortcutLabel(status.paletteShortcut) }}
          </a-descriptions-item>
          <a-descriptions-item label="条目">
            {{ status.totalCount }}（固定 {{ status.pinnedCount }}）
          </a-descriptions-item>
        </a-descriptions>
      </a-card>
    </div>
  </a-spin>
</template>

<style scoped lang="scss">
.clipboard-settings {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  max-width: 760px;
}

.clipboard-settings__card {
  background: var(--app-surface);
}

.clipboard-settings__desc {
  display: block;
  font-size: 12px;
  line-height: 1.45;
}

.clipboard-settings__switches {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;

  &--disabled {
    opacity: 0.55;
  }
}
</style>
