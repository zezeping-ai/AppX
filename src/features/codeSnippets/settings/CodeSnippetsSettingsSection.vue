<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { message } from "ant-design-vue";
import ShortcutRecorder from "@/components/ShortcutRecorder/index.vue";
import {
  getCodeSnippetPermissions,
  getCodeSnippetSettings,
  openAccessibilitySettings,
  saveCodeSnippetSettings,
  setInlineExpansionTrigger,
  type CodeSnippetPermissionsView,
  type CodeSnippetSettingsView,
} from "@/modules/codeSnippets";
import { setGlobalShortcutsPaused } from "@/modules/globalShortcut";
import { formatShortcutLabel, normalizeGlobalShortcut } from "@/shared/shortcut";

const loading = ref(false);
const saving = ref(false);
const permissions = ref<CodeSnippetPermissionsView | null>(null);

const enabled = ref(true);
const inlineExpansionEnabled = ref(true);
const inlineExpansionTrigger = ref("F12");
const shortcutsEnabled = ref(true);
const paletteEnabled = ref(true);

let lastPersistedTrigger = "F12";

const featuresActive = computed(() => enabled.value);

const inlineExpansionActive = computed(
  () => featuresActive.value && inlineExpansionEnabled.value,
);

const shortcutsActive = computed(() => featuresActive.value && shortcutsEnabled.value);

const paletteActive = computed(() => featuresActive.value && paletteEnabled.value);

const accessibilityGranted = computed(() => permissions.value?.accessibilityGranted === true);

const listenerReady = computed(
  () =>
    inlineExpansionActive.value &&
    permissions.value?.listenerActive === true &&
    (permissions.value?.registeredAbbreviationCount ?? 0) > 0,
);

const inlineExpansionTriggerLabel = computed(() =>
  formatShortcutLabel(inlineExpansionTrigger.value),
);

const paletteShortcutLabel = computed(() =>
  permissions.value?.paletteShortcut
    ? formatShortcutLabel(permissions.value.paletteShortcut)
    : "—",
);

function applySettings(settings: CodeSnippetSettingsView) {
  enabled.value = settings.enabled;
  inlineExpansionEnabled.value = settings.inlineExpansionEnabled;
  inlineExpansionTrigger.value = settings.inlineExpansionTrigger;
  shortcutsEnabled.value = settings.shortcutsEnabled;
  paletteEnabled.value = settings.paletteEnabled;
  lastPersistedTrigger = settings.inlineExpansionTrigger;
  setInlineExpansionTrigger(settings.inlineExpansionTrigger);
}

function currentSettingsInput() {
  return {
    enabled: enabled.value,
    inlineExpansionEnabled: inlineExpansionEnabled.value,
    inlineExpansionTrigger: inlineExpansionTrigger.value,
    shortcutsEnabled: shortcutsEnabled.value,
    paletteEnabled: paletteEnabled.value,
  };
}

async function refresh() {
  loading.value = true;
  try {
    const [perm, settings] = await Promise.all([
      getCodeSnippetPermissions(),
      getCodeSnippetSettings(),
    ]);
    permissions.value = perm;
    applySettings(settings);
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function persistSettings(successMessage: string) {
  const previous = currentSettingsInput();
  saving.value = true;
  try {
    const saved = await saveCodeSnippetSettings(currentSettingsInput());
    applySettings(saved);
    await refresh();
    message.success(successMessage);
  } catch (error) {
    applySettings(previous);
    message.error(String(error));
  } finally {
    saving.value = false;
  }
}

async function onToggleEnabled(checked: boolean) {
  enabled.value = checked;
  await persistSettings(checked ? "已启用代码段全局功能" : "已停用代码段全局功能");
}

async function onToggleInlineExpansion(checked: boolean) {
  inlineExpansionEnabled.value = checked;
  await persistSettings(
    checked
      ? `已启用 :缩写 + ${inlineExpansionTriggerLabel.value} 展开`
      : `已停用 :缩写 + ${inlineExpansionTriggerLabel.value} 展开`,
  );
}

async function onToggleShortcuts(checked: boolean) {
  shortcutsEnabled.value = checked;
  await persistSettings(checked ? "已启用全局快捷键" : "已停用全局快捷键");
}

async function onTogglePalette(checked: boolean) {
  paletteEnabled.value = checked;
  await persistSettings(checked ? "已启用快捷键命令面板" : "已停用快捷键命令面板");
}

async function onInlineExpansionTriggerChange(shortcut: string) {
  const normalized = normalizeGlobalShortcut(shortcut);
  if (!normalized || normalized === lastPersistedTrigger) return;

  inlineExpansionTrigger.value = normalized;
  await persistSettings(`展开触发键已设为 ${formatShortcutLabel(normalized)}`);
}

async function openAccessibilitySettingsPage() {
  try {
    await openAccessibilitySettings();
  } catch (error) {
    message.error(String(error));
  }
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <a-space direction="vertical" size="middle" class="code-snippets-settings">
    <a-card title="全局功能" :bordered="false" :loading="loading">
      <a-space direction="vertical" size="middle" class="w-full">
        <a-checkbox
          :checked="enabled"
          :disabled="saving || loading"
          @update:checked="onToggleEnabled"
        >
          启用代码段全局功能
        </a-checkbox>
        <a-typography-text type="secondary" class="block text-[12px] leading-snug">
          关闭后，缩写展开、全局快捷键与快捷键命令面板将全部停用；子项开关保留配置，重新开启总开关后按各自状态恢复。
        </a-typography-text>

        <div class="feature-switches app-surface-muted" :class="{ 'feature-switches--disabled': !featuresActive }">
          <a-checkbox
            :checked="inlineExpansionEnabled"
            :disabled="!featuresActive || saving || loading"
            @update:checked="onToggleInlineExpansion"
          >
            启用 <code>:缩写 + {{ inlineExpansionTriggerLabel }}</code> 全局展开
          </a-checkbox>
          <a-typography-text type="secondary" class="feature-switches__hint">
            在其他应用中输入 <code>:缩写</code> 后按
            <strong>{{ inlineExpansionTriggerLabel }}</strong> 自动展开（仅 macOS）。
          </a-typography-text>

          <div
            v-if="permissions?.platform === 'macos'"
            class="feature-switches__trigger"
            :class="{ 'feature-switches__trigger--disabled': !featuresActive || saving || loading }"
          >
            <span class="feature-switches__trigger-label">展开触发键</span>
            <ShortcutRecorder
              v-model:value="inlineExpansionTrigger"
              :on-recording-change="setGlobalShortcutsPaused"
              @update:value="onInlineExpansionTriggerChange"
            />
          </div>

          <a-checkbox
            :checked="shortcutsEnabled"
            :disabled="!featuresActive || saving || loading"
            @update:checked="onToggleShortcuts"
          >
            启用全局快捷键
          </a-checkbox>
          <a-typography-text type="secondary" class="feature-switches__hint">
            各代码段单独配置的快捷键，可在任意应用中插入内容。
          </a-typography-text>

          <a-checkbox
            :checked="paletteEnabled"
            :disabled="!featuresActive || saving || loading"
            @update:checked="onTogglePalette"
          >
            启用快捷键命令面板
          </a-checkbox>
          <a-typography-text type="secondary" class="feature-switches__hint">
            按 <strong>{{ paletteShortcutLabel }}</strong> 打开快捷键命令面板，搜索并选择代码段插入到当前光标位置。
          </a-typography-text>
        </div>

        <a-space wrap>
          <a-tag :color="inlineExpansionActive ? 'success' : 'default'">
            缩写展开：{{ inlineExpansionActive ? "运行中" : "已停用" }}
          </a-tag>
          <a-tag :color="shortcutsActive ? 'success' : 'default'">
            全局快捷键：{{ shortcutsActive ? "运行中" : "已停用" }}
          </a-tag>
          <a-tag :color="paletteActive ? 'success' : 'default'">
            快捷键命令面板：{{ paletteActive ? "运行中" : "已停用" }}
          </a-tag>
        </a-space>
      </a-space>
    </a-card>

    <a-card
      v-if="permissions?.platform === 'macos'"
      title="权限与系统配置"
      :bordered="false"
      :loading="loading"
    >
      <a-space direction="vertical" size="middle" class="w-full">
        <a-space wrap>
          <a-tag :color="accessibilityGranted ? 'success' : 'warning'">
            辅助功能：{{ accessibilityGranted ? "已授权" : "未授权" }}
          </a-tag>
          <a-tag :color="permissions?.listenerActive ? 'success' : 'error'">
            全局监听：{{ permissions?.listenerActive ? "运行中" : "未启动" }}
          </a-tag>
          <a-tag :color="listenerReady ? 'success' : 'default'">
            已同步缩写：{{ permissions?.registeredAbbreviationCount ?? 0 }}
          </a-tag>
          <a-button size="small" @click="refresh">刷新状态</a-button>
          <a-button size="small" type="primary" @click="openAccessibilitySettingsPage">
            打开系统设置
          </a-button>
        </a-space>

        <a-alert
          v-if="inlineExpansionActive && accessibilityGranted && !permissions?.listenerActive"
          type="error"
          show-icon
          message="全局缩写监听未启动"
          description="请完全退出并重启 AppX；若仍失败，请移除后重新添加辅助功能权限。"
        />

        <a-alert
          v-else-if="permissions?.listenerActive && (permissions?.registeredAbbreviationCount ?? 0) === 0"
          type="warning"
          show-icon
          message="运行时未加载任何缩写"
          description="请打开「代码段」页面或保存一次代码段，将数据同步到运行时。"
        />

        <ol class="permission-steps">
          <li>打开「系统设置」→「隐私与安全性」→「辅助功能」</li>
          <li>点击锁图标解锁（如需要）</li>
          <li>在列表中找到并启用 <strong>AppX</strong></li>
          <li>授权后点击「刷新状态」确认显示「已授权」</li>
        </ol>
      </a-space>
    </a-card>
  </a-space>
</template>

<style scoped lang="scss">
.code-snippets-settings {
  width: 100%;
  max-width: 760px;
}

.feature-switches {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 14px;
  border-radius: 8px;

  &--disabled {
    opacity: 0.55;
  }
}

.feature-switches__hint {
  display: block;
  margin: 0 0 8px 24px;
  font-size: 12px;
  line-height: 1.5;
}

.feature-switches__trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 0 8px 24px;
  max-width: 320px;

  &--disabled {
    pointer-events: none;
  }
}

.feature-switches__trigger-label {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--app-fg-subtle);
}

.permission-steps {
  margin: 0;
  padding-left: 1.25rem;
  font-size: 13px;
  line-height: 1.65;
  color: var(--app-fg-subtle);

  li + li {
    margin-top: 6px;
  }
}
</style>
