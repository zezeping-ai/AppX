<script setup lang="ts">
import { message } from "ant-design-vue";
import { useImageOptimSettings } from "@/modules/imageOptim";
import { getErrorMessage } from "@/shared/error";

const { settings, loaded, persist, patch } = useImageOptimSettings();

async function onToggle(key: "stripMetadata" | "lossyEnabled", checked: boolean) {
  try {
    await patch({ [key]: checked });
    message.success(
      key === "stripMetadata"
        ? checked
          ? "已开启剥离元数据"
          : "已关闭剥离元数据"
        : checked
          ? "已开启有损压缩"
          : "已关闭有损压缩",
    );
  } catch (error) {
    message.error(getErrorMessage(error, "保存失败"));
  }
}

async function onSliderPersist(label: string) {
  try {
    await persist({ ...settings.value });
    message.success(`已更新${label}`);
  } catch (error) {
    message.error(getErrorMessage(error, "保存失败"));
  }
}
</script>

<template>
  <a-spin :spinning="!loaded">
    <a-space direction="vertical" size="middle" class="image-optim-settings">
      <div class="image-optim-settings__row">
        <div>
          <div class="image-optim-settings__label">剥离元数据</div>
          <div class="image-optim-settings__hint">移除 EXIF、注释等可选元数据</div>
        </div>
        <a-switch
          :checked="settings.stripMetadata"
          @change="(v: boolean) => void onToggle('stripMetadata', v)"
        />
      </div>

      <div class="image-optim-settings__row">
        <div>
          <div class="image-optim-settings__label">有损压缩</div>
          <div class="image-optim-settings__hint">默认关闭；开启后可用更激进算法换更小体积</div>
        </div>
        <a-switch
          :checked="settings.lossyEnabled"
          @change="(v: boolean) => void onToggle('lossyEnabled', v)"
        />
      </div>

      <div class="image-optim-settings__block">
        <div class="image-optim-settings__label">压缩强度（{{ settings.effort }}）</div>
        <div class="image-optim-settings__hint">主要影响 PNG；0 最快，6 最慢通常最小</div>
        <a-slider
          v-model:value="settings.effort"
          :min="0"
          :max="6"
          :marks="{ 0: '快', 2: '默认', 6: '强' }"
          @afterChange="() => void onSliderPersist('压缩强度')"
        />
      </div>

      <div class="image-optim-settings__block" :class="{ 'is-disabled': !settings.lossyEnabled }">
        <div class="image-optim-settings__label">JPEG 质量上限（{{ settings.jpegMaxQuality }}）</div>
        <div class="image-optim-settings__hint">100 表示对该格式不做有损重编码</div>
        <a-slider
          v-model:value="settings.jpegMaxQuality"
          :min="40"
          :max="100"
          :disabled="!settings.lossyEnabled"
          @afterChange="() => void onSliderPersist('JPEG 质量')"
        />
      </div>

      <div class="image-optim-settings__block" :class="{ 'is-disabled': !settings.lossyEnabled }">
        <div class="image-optim-settings__label">PNG 质量下限（{{ settings.pngMinQuality }}）</div>
        <div class="image-optim-settings__hint">100 表示对该格式不做有损量化</div>
        <a-slider
          v-model:value="settings.pngMinQuality"
          :min="40"
          :max="100"
          :disabled="!settings.lossyEnabled"
          @afterChange="() => void onSliderPersist('PNG 质量')"
        />
      </div>

      <div class="image-optim-settings__block" :class="{ 'is-disabled': !settings.lossyEnabled }">
        <div class="image-optim-settings__label">GIF 质量（{{ settings.gifQuality }}）</div>
        <div class="image-optim-settings__hint">100 表示对该格式不做有损量化</div>
        <a-slider
          v-model:value="settings.gifQuality"
          :min="40"
          :max="100"
          :disabled="!settings.lossyEnabled"
          @afterChange="() => void onSliderPersist('GIF 质量')"
        />
      </div>
    </a-space>
  </a-spin>
</template>

<style scoped lang="scss">
.image-optim-settings {
  width: 100%;
  max-width: 560px;

  &__row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  &__block {
    &.is-disabled {
      opacity: 0.55;
    }
  }

  &__label {
    font-size: 13px;
    font-weight: 500;
    color: var(--ant-color-text);
  }

  &__hint {
    margin-top: 2px;
    font-size: 12px;
    color: var(--ant-color-text-secondary);
  }
}
</style>
