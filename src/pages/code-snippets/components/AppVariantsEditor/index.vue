<script setup lang="ts">
import { Icon } from "@/components/Icon";
import MonacoEditor from "@/components/MonacoEditor/index.vue";
import type { CodeSnippetAppVariant } from "@/models/codeSnippets/variants";
import { emptyAppVariant } from "@/models/codeSnippets/variants";
import { usePickTargetApp } from "@/pages/code-snippets/composables/usePickTargetApp";

const variants = defineModel<CodeSnippetAppVariant[]>({ required: true });

const props = defineProps<{
  language: string;
}>();

const { pickingIndex, startPick, stopPicking } = usePickTargetApp({
  isAppTaken: (appId, index) =>
    variants.value.some((v, i) => i !== index && v.appId === appId),
});

function addVariant() {
  variants.value = [...variants.value, emptyAppVariant()];
}

function removeVariant(index: number) {
  if (pickingIndex.value === index) stopPicking();
  variants.value = variants.value.filter((_, i) => i !== index);
}

function onPick(index: number) {
  void startPick(index, (app) => {
    const row = variants.value[index];
    if (!row) return;
    row.appId = app.appId;
    row.appName = app.appName;
  });
}

defineExpose({ stopPicking });
</script>

<template>
  <div class="app-variants">
    <div class="app-variants__head">
      <div>
        <div class="app-variants__title">按应用覆盖</div>
        <div class="app-variants__hint">可选。触发时若前台为绑定应用，优先使用对应内容</div>
      </div>
      <a-button size="small" @click="addVariant">
        <template #icon><Icon icon="mdi:plus" /></template>
        添加
      </a-button>
    </div>

    <div v-if="variants.length === 0" class="app-variants__empty">暂无覆盖，使用上方默认内容</div>

    <div
      v-for="(item, index) in variants"
      :key="index"
      class="app-variants__item"
    >
      <div class="app-variants__item-bar">
        <div class="app-variants__target">
          <span class="app-variants__app-name">
            {{ item.appName || item.appId || "未选择应用" }}
          </span>
          <a-button
            size="small"
            type="primary"
            ghost
            :loading="pickingIndex === index"
            @click="onPick(index)"
          >
            <template #icon><Icon icon="mdi:crosshairs-gps" /></template>
            {{ pickingIndex === index ? "选择中…" : "选择目标" }}
          </a-button>
          <a-button
            v-if="pickingIndex === index"
            size="small"
            @click="stopPicking"
          >
            取消
          </a-button>
        </div>
        <a-button type="text" danger size="small" @click="removeVariant(index)">
          <template #icon><Icon icon="mdi:delete-outline" /></template>
        </a-button>
      </div>
      <MonacoEditor
        v-model="item.content"
        :language="props.language"
        auto-height
        :min-height="120"
      />
    </div>
  </div>
</template>

<style scoped lang="scss">
.app-variants {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.app-variants__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.app-variants__title {
  font-size: 14px;
  font-weight: 500;
}

.app-variants__hint {
  margin-top: 2px;
  font-size: 12px;
  color: #6b7280;
}

[data-theme="dark"] .app-variants__hint {
  color: rgba(255, 255, 255, 0.45);
}

.app-variants__empty {
  font-size: 12px;
  color: #9ca3af;
  padding: 8px 0;
}

.app-variants__item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 12px;
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 8px;
}

[data-theme="dark"] .app-variants__item {
  border-color: rgba(255, 255, 255, 0.08);
}

.app-variants__item-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.app-variants__target {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.app-variants__app-name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 220px;
}
</style>
