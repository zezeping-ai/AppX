<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { message } from "ant-design-vue";
import ShortcutRecorder from "@/components/ShortcutRecorder/index.vue";
import {
  getSettings,
  getStatus,
  saveSettings,
  normalizePaletteLayout,
  pickClipboardSoundFile,
  playClipboardSound,
  soundFileLabel,
  type ClipboardAssistantSettings,
  type ClipboardAssistantStatus,
  type ClipboardSoundKind,
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
const copySoundEnabled = ref(true);
const pasteSoundEnabled = ref(true);
const copySoundPath = ref("");
const pasteSoundPath = ref("");

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
  copySoundEnabled.value = settings.copySoundEnabled ?? true;
  pasteSoundEnabled.value = settings.pasteSoundEnabled ?? true;
  copySoundPath.value = settings.copySoundPath ?? "";
  pasteSoundPath.value = settings.pasteSoundPath ?? "";
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
    copySoundEnabled: copySoundEnabled.value,
    pasteSoundEnabled: pasteSoundEnabled.value,
    copySoundPath: copySoundPath.value,
    pasteSoundPath: pasteSoundPath.value,
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

async function onToggleCopySound(checked: boolean) {
  copySoundEnabled.value = checked;
  await persistSettings(checked ? "已开启复制音效" : "已关闭复制音效");
}

async function onTogglePasteSound(checked: boolean) {
  pasteSoundEnabled.value = checked;
  await persistSettings(checked ? "已开启粘贴音效" : "已关闭粘贴音效");
}

async function onPreviewSound(kind: ClipboardSoundKind) {
  await playClipboardSound(kind, {
    force: true,
    path: kind === "copy" ? copySoundPath.value : pasteSoundPath.value,
  });
}

async function onPickSound(kind: ClipboardSoundKind) {
  if (saving.value || loading.value || !featuresActive.value) return;
  try {
    const picked = await pickClipboardSoundFile();
    if (!picked) return;
    if (kind === "copy") copySoundPath.value = picked;
    else pasteSoundPath.value = picked;
    await persistSettings(kind === "copy" ? "复制音效已更新" : "粘贴音效已更新");
    await onPreviewSound(kind);
  } catch (error) {
    message.error(getErrorMessage(error, "选择音效失败"));
  }
}

async function onResetSound(kind: ClipboardSoundKind) {
  if (kind === "copy") {
    if (!copySoundPath.value) return;
    copySoundPath.value = "";
  } else {
    if (!pasteSoundPath.value) return;
    pasteSoundPath.value = "";
  }
  await persistSettings(kind === "copy" ? "已恢复默认复制音效" : "已恢复默认粘贴音效");
  await onPreviewSound(kind);
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <a-spin :spinning="loading">
    <a-space direction="vertical" size="small" class="clipboard-settings">
      <a-card title="全局功能" size="small" :bordered="false">
        <a-space direction="vertical" size="small" class="w-full">
          <a-checkbox
            :checked="enabled"
            :disabled="saving || loading"
            @update:checked="onToggleEnabled"
          >
            启用剪切助手
          </a-checkbox>

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
              浮层快捷键（{{ paletteShortcutLabel }}）
            </a-checkbox>
          </div>
        </a-space>
      </a-card>

      <a-card title="历史与浮层" size="small" :bordered="false">
        <div class="clipboard-settings__rows">
          <div class="clipboard-settings__row">
            <span class="clipboard-settings__label">历史条数上限</span>
            <div class="clipboard-settings__value">
              <a-input-number
                :value="maxHistoryItems"
                size="small"
                :min="50"
                :max="5000"
                :disabled="!featuresActive || saving || loading"
                class="clipboard-settings__number"
                @update:value="onMaxHistoryItemsChange"
              />
            </div>
          </div>
          <div class="clipboard-settings__row">
            <span class="clipboard-settings__label">呼出快捷键</span>
            <div class="clipboard-settings__value">
              <ShortcutRecorder
                v-model:value="paletteShortcut"
                :on-recording-change="setGlobalShortcutsPaused"
                @update:value="onPaletteShortcutChange"
              />
            </div>
          </div>
          <div class="clipboard-settings__row">
            <span class="clipboard-settings__label">浮层布局</span>
            <div class="clipboard-settings__value">
              <a-radio-group
                :value="paletteLayout"
                size="small"
                :disabled="!paletteActive || saving || loading"
                @change="onPaletteLayoutChange"
              >
                <a-radio-button value="topPanel">上</a-radio-button>
                <a-radio-button value="bottomPanel">下</a-radio-button>
                <a-radio-button value="leftPanel">左</a-radio-button>
                <a-radio-button value="rightPanel">右</a-radio-button>
              </a-radio-group>
            </div>
          </div>
          <div class="clipboard-settings__row">
            <span class="clipboard-settings__label">浮层最大条目</span>
            <div class="clipboard-settings__value">
              <a-input-number
                :value="paletteMaxItems"
                size="small"
                :min="20"
                :max="200"
                :disabled="!paletteActive || saving || loading"
                class="clipboard-settings__number"
                @update:value="onPaletteMaxItemsChange"
              />
            </div>
          </div>
        </div>
      </a-card>

      <a-card title="行为" size="small" :bordered="false">
        <div class="clipboard-settings__rows">
          <div class="clipboard-settings__row">
            <span class="clipboard-settings__label">粘贴后关闭浮层</span>
            <div class="clipboard-settings__value">
              <a-switch
                size="small"
                :checked="autoHideOnPaste"
                :disabled="!featuresActive || saving || loading"
                @update:checked="onToggleAutoHideOnPaste"
              />
            </div>
          </div>
          <div class="clipboard-settings__row">
            <span class="clipboard-settings__label">锁定时清空未固定</span>
            <div class="clipboard-settings__value">
              <a-switch
                size="small"
                :checked="clearOnLock"
                :disabled="!featuresActive || saving || loading"
                @update:checked="onToggleClearOnLock"
              />
            </div>
          </div>
        </div>
      </a-card>

      <a-card title="音效" size="small" :bordered="false">
        <div class="clipboard-settings__rows">
          <div class="clipboard-settings__row">
            <span class="clipboard-settings__label">
              复制音效
              <span
                v-if="copySoundPath"
                class="clipboard-settings__hint"
                :title="copySoundPath"
              >
                · {{ soundFileLabel(copySoundPath) }}
              </span>
            </span>
            <div class="clipboard-settings__value">
              <a-space size="small">
                <a-button
                  type="link"
                  size="small"
                  class="clipboard-settings__link"
                  :disabled="!featuresActive || saving || loading"
                  @click="onPreviewSound('copy')"
                >
                  试听
                </a-button>
                <a-button
                  type="link"
                  size="small"
                  class="clipboard-settings__link"
                  :disabled="!featuresActive || saving || loading"
                  @click="onPickSound('copy')"
                >
                  更换
                </a-button>
                <a-button
                  v-if="copySoundPath"
                  type="link"
                  size="small"
                  class="clipboard-settings__link"
                  :disabled="!featuresActive || saving || loading"
                  @click="onResetSound('copy')"
                >
                  默认
                </a-button>
                <a-switch
                  size="small"
                  :checked="copySoundEnabled"
                  :disabled="!featuresActive || saving || loading"
                  @update:checked="onToggleCopySound"
                />
              </a-space>
            </div>
          </div>

          <div class="clipboard-settings__row">
            <span class="clipboard-settings__label">
              粘贴音效
              <span
                v-if="pasteSoundPath"
                class="clipboard-settings__hint"
                :title="pasteSoundPath"
              >
                · {{ soundFileLabel(pasteSoundPath) }}
              </span>
            </span>
            <div class="clipboard-settings__value">
              <a-space size="small">
                <a-button
                  type="link"
                  size="small"
                  class="clipboard-settings__link"
                  :disabled="!featuresActive || saving || loading"
                  @click="onPreviewSound('paste')"
                >
                  试听
                </a-button>
                <a-button
                  type="link"
                  size="small"
                  class="clipboard-settings__link"
                  :disabled="!featuresActive || saving || loading"
                  @click="onPickSound('paste')"
                >
                  更换
                </a-button>
                <a-button
                  v-if="pasteSoundPath"
                  type="link"
                  size="small"
                  class="clipboard-settings__link"
                  :disabled="!featuresActive || saving || loading"
                  @click="onResetSound('paste')"
                >
                  默认
                </a-button>
                <a-switch
                  size="small"
                  :checked="pasteSoundEnabled"
                  :disabled="!featuresActive || saving || loading"
                  @update:checked="onTogglePasteSound"
                />
              </a-space>
            </div>
          </div>
        </div>
      </a-card>

      <a-card v-if="status" title="状态" size="small" :bordered="false">
        <a-space size="small" wrap>
          <a-tag :color="status.paletteActive || status.monitoringActive ? 'success' : 'default'">
            {{ status.paletteActive || status.monitoringActive ? "运行中" : "已停用" }}
          </a-tag>
          <a-tag :color="status.monitoringActive ? 'success' : 'default'">
            监听 {{ status.monitoringActive ? "运行中" : "暂停" }}
          </a-tag>
          <a-tag>{{ formatShortcutLabel(status.paletteShortcut) }}</a-tag>
          <a-tag>{{ status.totalCount }} 条（固定 {{ status.pinnedCount }}）</a-tag>
        </a-space>
      </a-card>
    </a-space>
  </a-spin>
</template>

<style scoped lang="scss">
.clipboard-settings {
  width: 100%;
  max-width: 640px;
}

.clipboard-settings__switches {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 6px;

  &--disabled {
    opacity: 0.55;
  }
}

.clipboard-settings__rows {
  display: flex;
  flex-direction: column;
}

.clipboard-settings__row {
  display: grid;
  grid-template-columns: 8.5rem minmax(0, 1fr);
  align-items: center;
  column-gap: 12px;
  min-height: 36px;
  padding: 2px 0;

  & + & {
    border-top: 1px solid var(--app-border, rgba(0, 0, 0, 0.06));
  }
}

.clipboard-settings__label {
  font-size: 13px;
  line-height: 1.3;
  color: var(--app-fg);
}

.clipboard-settings__value {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  min-width: 0;
}

.clipboard-settings__number {
  width: 112px;
}

.clipboard-settings__hint {
  margin-left: 4px;
  color: var(--app-fg-subtle);
  font-weight: 400;
}

.clipboard-settings__link {
  padding-inline: 0 !important;
  height: auto !important;
}
</style>
