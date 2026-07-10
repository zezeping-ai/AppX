<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { message } from "ant-design-vue";
import { Icon } from "@iconify/vue";
import {
  getCodeSnippetPermissions,
  getCodeSnippetSettings,
  openAccessibilitySettings,
  saveCodeSnippetSettings,
  type CodeSnippetPermissionsView,
} from "@/modules/codeSnippets";
import { formatAbbreviationTrigger } from "@/shared/abbreviation";
import { formatShortcutLabel } from "@/shared/shortcut";

const loading = ref(false);
const saving = ref(false);
const permissions = ref<CodeSnippetPermissionsView | null>(null);
const inlineExpansionEnabled = ref(true);

const platformLabel = computed(() => {
  switch (permissions.value?.platform) {
    case "macos":
      return "macOS";
    case "windows":
      return "Windows";
    case "linux":
      return "Linux";
    default:
      return "当前系统";
  }
});

const accessibilityGranted = computed(() => permissions.value?.accessibilityGranted === true);

const listenerReady = computed(
  () =>
    permissions.value?.listenerActive === true &&
    inlineExpansionEnabled.value &&
    (permissions.value?.registeredAbbreviationCount ?? 0) > 0,
);

const paletteShortcutLabel = computed(() =>
  permissions.value?.paletteShortcut
    ? formatShortcutLabel(permissions.value.paletteShortcut)
    : "—",
);

function formatRegisteredTrigger(abbrev: string) {
  return formatAbbreviationTrigger(abbrev);
}

async function refresh() {
  loading.value = true;
  try {
    const [perm, settings] = await Promise.all([
      getCodeSnippetPermissions(),
      getCodeSnippetSettings(),
    ]);
    permissions.value = perm;
    inlineExpansionEnabled.value = settings.inlineExpansionEnabled;
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function onToggleInlineExpansion(checked: boolean) {
  const previous = inlineExpansionEnabled.value;
  inlineExpansionEnabled.value = checked;
  saving.value = true;
  try {
    await saveCodeSnippetSettings({ inlineExpansionEnabled: checked });
    await refresh();
    message.success(checked ? "已启用 :缩写; 展开" : "已停用 :缩写; 展开");
  } catch (error) {
    inlineExpansionEnabled.value = previous;
    message.error(String(error));
  } finally {
    saving.value = false;
  }
}

async function openAccessibilitySettingsPage() {
  try {
    await openAccessibilitySettings();
  } catch (error) {
    message.error(String(error));
  }
}

watch(inlineExpansionEnabled, (value) => {
  if (permissions.value) {
    permissions.value = { ...permissions.value, inlineExpansionEnabled: value };
  }
});

onMounted(() => {
  void refresh();
});
</script>

<template>
  <a-space direction="vertical" size="middle" class="code-snippets-settings">
    <a-card title="展开方式" :bordered="false" :loading="loading">
      <a-space direction="vertical" size="middle" class="w-full">
        <a-checkbox
          :checked="inlineExpansionEnabled"
          :disabled="saving || loading"
          @update:checked="onToggleInlineExpansion"
        >
          启用 <code>:缩写;</code> 全局展开
        </a-checkbox>
        <a-typography-text type="secondary" class="block text-[12px] leading-snug">
          在其他应用中输入 <code>:缩写;</code> 自动展开；关闭后仅保留全局快捷键与命令面板。
        </a-typography-text>

        <a-alert type="info" show-icon>
          <template #message>命令面板</template>
          <template #description>
            按 <strong>{{ paletteShortcutLabel }}</strong> 打开命令面板，搜索并选择代码段插入到当前光标位置。
          </template>
        </a-alert>
      </a-space>
    </a-card>

    <a-card title="权限与系统配置" :bordered="false" :loading="loading">
      <a-space direction="vertical" size="middle" class="w-full">
        <a-alert type="info" show-icon>
          <template #message>当前系统：{{ platformLabel }}</template>
          <template #description>
            <code>:缩写;</code> 跨应用展开目前仅在 macOS 上可用；命令面板与全局快捷键全平台可用。
          </template>
        </a-alert>

        <template v-if="permissions?.platform === 'macos'">
          <div class="permission-block">
            <div class="permission-block__head">
              <Icon icon="mdi:human-greeting-proximity" class="permission-block__icon" aria-hidden="true" />
              <div>
                <a-typography-text strong>辅助功能（Accessibility）</a-typography-text>
                <a-typography-text type="secondary" class="block text-[12px] leading-snug mt-0.5">
                  监听全局按键（<code>:缩写;</code>）并向其他应用插入文本时需要此权限。
                </a-typography-text>
              </div>
            </div>

            <a-space wrap>
              <a-tag :color="accessibilityGranted ? 'success' : 'warning'">
                {{ accessibilityGranted ? "已授权" : "未授权" }}
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

            <a-typography-text
              v-if="(permissions?.registeredAbbreviations?.length ?? 0) > 0"
              type="secondary"
              class="block text-[12px] leading-snug"
            >
              已注册缩写：{{ permissions?.registeredAbbreviations?.map(formatRegisteredTrigger).join("、") }}
            </a-typography-text>

            <a-alert
              v-if="inlineExpansionEnabled && accessibilityGranted && !permissions?.listenerActive"
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

            <a-typography-title :level="5" class="permission-block__steps-title">
              配置步骤
            </a-typography-title>
            <ol class="permission-block__steps">
              <li>打开「系统设置」→「隐私与安全性」→「辅助功能」</li>
              <li>点击锁图标解锁（如需要）</li>
              <li>在列表中找到并启用 <strong>AppX</strong></li>
              <li>授权后点击「刷新状态」确认显示「已授权」</li>
            </ol>
          </div>
        </template>

        <template v-else-if="permissions?.platform === 'windows'">
          <div class="permission-block">
            <a-typography-title :level="5" class="permission-block__steps-title">
              Windows 使用说明
            </a-typography-title>
            <ul class="permission-block__steps permission-block__steps--unordered">
              <li><strong>命令面板</strong>：{{ paletteShortcutLabel }}</li>
              <li><strong>全局快捷键</strong>：各代码段单独配置，无需管理员权限。</li>
              <li><strong>:缩写; 展开</strong>：当前版本暂不支持，请使用命令面板或快捷键。</li>
            </ul>
          </div>
        </template>

        <template v-else-if="permissions?.platform === 'linux'">
          <div class="permission-block">
            <a-typography-title :level="5" class="permission-block__steps-title">
              Linux 使用说明
            </a-typography-title>
            <ul class="permission-block__steps permission-block__steps--unordered">
              <li><strong>命令面板</strong>：{{ paletteShortcutLabel }}</li>
              <li><strong>全局快捷键</strong>：依赖桌面环境，Wayland 下可能受限。</li>
              <li><strong>:缩写; 展开</strong>：当前版本暂不支持，请使用命令面板或快捷键。</li>
            </ul>
          </div>
        </template>
      </a-space>
    </a-card>
  </a-space>
</template>

<style scoped lang="scss">
.code-snippets-settings {
  width: 100%;
  max-width: 760px;
}

.permission-block {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 8px;
  background: rgb(0 0 0 / 3%);
}

.permission-block__head {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.permission-block__icon {
  flex-shrink: 0;
  margin-top: 2px;
  font-size: 22px;
  color: rgb(0 0 0 / 55%);
}

.permission-block__steps-title {
  margin: 4px 0 0 !important;
  font-size: 14px !important;
}

.permission-block__steps {
  margin: 0;
  padding-left: 1.25rem;
  font-size: 13px;
  line-height: 1.65;
  color: rgb(0 0 0 / 75%);

  li + li {
    margin-top: 6px;
  }

  &--unordered {
    list-style: disc;
  }
}
</style>
