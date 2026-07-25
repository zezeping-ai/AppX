<script setup lang="ts">
import { Icon } from "@iconify/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed } from "vue";
import {
  HIGHLIGHT_COLORS,
  TEXT_COLORS,
  applyHighlight,
  applyTextColor,
  applyUnderlineColor,
  currentHighlightColor,
  currentTextColor,
  currentUnderlineColor,
} from "./colorMenu";
import {
  createFontFamilyMenu,
  currentFontFamily,
  FONT_FAMILIES,
} from "./fontFamilyMenu";
import { createFontSizeMenu, currentFontSize } from "./fontSizeMenu";
import { createFormatItems } from "./items";
import { createLabelMenu } from "./labelMenu";
import { createTableMenu } from "./tableMenu";

const props = defineProps<{
  editor: Editor;
}>();

const labelMenuActive = computed(
  () => props.editor.isActive("label") || props.editor.isActive("labelValue"),
);
const tableMenuActive = computed(() => props.editor.isActive("table"));
const underlineActive = computed(() => props.editor.isActive("underline"));
const colorActive = computed(() => currentTextColor(props.editor) != null);
const highlightActive = computed(() => props.editor.isActive("highlight"));
const fontFamilyActive = computed(() => currentFontFamily(props.editor) != null);
const fontSizeActive = computed(() => currentFontSize(props.editor) != null);

const fontFamilyLabel = computed(() => {
  const family = currentFontFamily(props.editor);
  if (!family) return "字体";
  return FONT_FAMILIES.find((f) => f.value === family)?.label ?? "字体";
});

const fontSizeLabel = computed(() => {
  const size = currentFontSize(props.editor);
  return size ? size.replace("px", "") : "14";
});

const labelMenu = computed(() => createLabelMenu(props.editor));
const tableMenu = computed(() => createTableMenu(props.editor));
const fontFamilyMenu = computed(() => createFontFamilyMenu(props.editor));
const fontSizeMenu = computed(() => createFontSizeMenu(props.editor));
const items = computed(() => createFormatItems(props.editor));

const activeTextColor = computed(() => currentTextColor(props.editor));
const activeHighlight = computed(() => currentHighlightColor(props.editor));
const activeUnderlineColor = computed(() => currentUnderlineColor(props.editor));

function insertLabel() {
  props.editor.chain().focus().insertLabel().run();
}

function insertTable() {
  props.editor
    .chain()
    .focus()
    .insertTable({ rows: 3, cols: 3, withHeaderRow: true })
    .run();
}

function toggleUnderline() {
  props.editor.chain().focus().toggleUnderline().run();
}

function onLabelMenuClick(info: { key: string | number }) {
  labelMenu.value.onClick?.(info as never);
}

function onTableMenuClick(info: { key: string | number }) {
  tableMenu.value.onClick?.(info as never);
}

function onFontFamilyMenuClick(info: { key: string | number }) {
  fontFamilyMenu.value.onClick?.(info as never);
}

function onFontSizeMenuClick(info: { key: string | number }) {
  fontSizeMenu.value.onClick?.(info as never);
}

function onPickTextColor(value: string | null) {
  applyTextColor(props.editor, value);
}

function onPickHighlight(value: string | null) {
  applyHighlight(props.editor, value);
}

function onPickUnderlineColor(value: string | null) {
  applyUnderlineColor(props.editor, value);
}
</script>

<template>
  <div class="rich-doc-toolbar">
    <template v-for="(item, index) in items" :key="index">
      <span v-if="item.kind === 'sep'" class="rich-doc-toolbar__sep" />

      <!-- Word：U 主按钮切换下划线，箭头选下划线颜色 -->
      <a-dropdown-button
        v-else-if="item.kind === 'underlineMenu'"
        trigger="click"
        placement="bottomLeft"
        class="rich-doc-toolbar__drop"
        :class="{ 'is-active': underlineActive }"
        @click="toggleUnderline"
      >
        <span class="rich-doc-toolbar__drop-main" title="下划线">
          <Icon icon="mdi:format-underline" width="16" height="16" />
        </span>
        <template #icon>
          <Icon icon="mdi:chevron-down" width="14" height="14" />
        </template>
        <template #overlay>
          <div class="rich-doc-toolbar__palette" @mousedown.prevent>
            <div class="rich-doc-toolbar__palette-title">下划线颜色</div>
            <div class="rich-doc-toolbar__swatches">
              <button
                v-for="c in TEXT_COLORS"
                :key="c.key"
                type="button"
                class="rich-doc-toolbar__swatch"
                :class="{
                  'is-default': c.value == null,
                  'is-active':
                    c.value == null
                      ? underlineActive && activeUnderlineColor == null
                      : activeUnderlineColor === c.value,
                }"
                :title="c.label"
                :style="c.value ? { background: c.value } : undefined"
                @click="onPickUnderlineColor(c.value)"
              >
                <Icon
                  v-if="c.value == null"
                  icon="mdi:format-color-marker-cancel"
                  width="14"
                  height="14"
                />
              </button>
            </div>
          </div>
        </template>
      </a-dropdown-button>

      <!-- Word：字体颜色 A + 色条 -->
      <a-dropdown
        v-else-if="item.kind === 'colorMenu'"
        trigger="click"
        placement="bottomLeft"
      >
        <button
          type="button"
          class="rich-doc-toolbar__btn rich-doc-toolbar__color-btn"
          title="字体颜色"
          :class="{ 'is-active': colorActive }"
        >
          <Icon icon="mdi:format-color-text" width="16" height="16" />
          <span
            class="rich-doc-toolbar__color-bar"
            :style="{
              background: activeTextColor ?? 'var(--app-fg-subtle)',
            }"
          />
        </button>
        <template #overlay>
          <div class="rich-doc-toolbar__palette" @mousedown.prevent>
            <div class="rich-doc-toolbar__palette-title">字体颜色</div>
            <div class="rich-doc-toolbar__swatches">
              <button
                v-for="c in TEXT_COLORS"
                :key="c.key"
                type="button"
                class="rich-doc-toolbar__swatch"
                :class="{
                  'is-default': c.value == null,
                  'is-active':
                    c.value == null
                      ? activeTextColor == null
                      : activeTextColor === c.value,
                }"
                :title="c.label"
                :style="c.value ? { background: c.value } : undefined"
                @click="onPickTextColor(c.value)"
              >
                <Icon
                  v-if="c.value == null"
                  icon="mdi:format-color-marker-cancel"
                  width="14"
                  height="14"
                />
              </button>
            </div>
          </div>
        </template>
      </a-dropdown>

      <!-- Word：文本突出显示颜色（荧光笔） -->
      <a-dropdown
        v-else-if="item.kind === 'highlightMenu'"
        trigger="click"
        placement="bottomLeft"
      >
        <button
          type="button"
          class="rich-doc-toolbar__btn rich-doc-toolbar__color-btn"
          title="突出显示"
          :class="{ 'is-active': highlightActive }"
        >
          <Icon icon="mdi:format-color-highlight" width="16" height="16" />
          <span
            class="rich-doc-toolbar__color-bar"
            :style="{
              background: activeHighlight ?? 'var(--app-fg-subtle)',
            }"
          />
        </button>
        <template #overlay>
          <div class="rich-doc-toolbar__palette" @mousedown.prevent>
            <div class="rich-doc-toolbar__palette-title">突出显示</div>
            <div class="rich-doc-toolbar__swatches">
              <button
                v-for="c in HIGHLIGHT_COLORS"
                :key="c.key"
                type="button"
                class="rich-doc-toolbar__swatch"
                :class="{
                  'is-default': c.value == null,
                  'is-active':
                    c.value == null
                      ? activeHighlight == null
                      : activeHighlight === c.value,
                }"
                :title="c.label"
                :style="c.value ? { background: c.value } : undefined"
                @click="onPickHighlight(c.value)"
              >
                <Icon
                  v-if="c.value == null"
                  icon="mdi:format-color-marker-cancel"
                  width="14"
                  height="14"
                />
              </button>
            </div>
          </div>
        </template>
      </a-dropdown>

      <!-- Word：字体名称下拉 -->
      <a-dropdown
        v-else-if="item.kind === 'fontFamilyMenu'"
        trigger="click"
        placement="bottomLeft"
      >
        <button
          type="button"
          title="字体"
          class="rich-doc-toolbar__btn rich-doc-toolbar__font-btn"
          :class="{ 'is-active': fontFamilyActive }"
        >
          <Icon icon="mdi:format-font" width="16" height="16" />
          <span>{{ fontFamilyLabel }}</span>
          <Icon icon="mdi:chevron-down" width="12" height="12" />
        </button>
        <template #overlay>
          <a-menu
            :items="fontFamilyMenu.items"
            :selected-keys="fontFamilyMenu.selectedKeys"
            @click="onFontFamilyMenuClick"
          />
        </template>
      </a-dropdown>

      <!-- Word：字号 Aa + 数值 -->
      <a-dropdown
        v-else-if="item.kind === 'fontSizeMenu'"
        trigger="click"
        placement="bottomLeft"
      >
        <button
          type="button"
          title="字号"
          class="rich-doc-toolbar__btn rich-doc-toolbar__size-btn"
          :class="{ 'is-active': fontSizeActive }"
        >
          <Icon icon="mdi:format-size" width="16" height="16" />
          <span>{{ fontSizeLabel }}</span>
          <Icon icon="mdi:chevron-down" width="12" height="12" />
        </button>
        <template #overlay>
          <a-menu
            :items="fontSizeMenu.items"
            :selected-keys="fontSizeMenu.selectedKeys"
            @click="onFontSizeMenuClick"
          />
        </template>
      </a-dropdown>

      <!-- DropdownButton 只认 overlay，不转发 :menu -->
      <a-dropdown-button
        v-else-if="item.kind === 'labelMenu'"
        trigger="click"
        placement="bottomLeft"
        class="rich-doc-toolbar__drop"
        :class="{ 'is-active': labelMenuActive }"
        @click="insertLabel"
      >
        <span class="rich-doc-toolbar__drop-main" title="插入标签">
          <Icon icon="mdi:label-outline" width="16" height="16" />
        </span>
        <template #icon>
          <Icon icon="mdi:chevron-down" width="14" height="14" />
        </template>
        <template #overlay>
          <a-menu :items="labelMenu.items" @click="onLabelMenuClick" />
        </template>
      </a-dropdown-button>

      <a-dropdown-button
        v-else-if="item.kind === 'tableMenu'"
        trigger="click"
        placement="bottomLeft"
        class="rich-doc-toolbar__drop"
        :class="{ 'is-active': tableMenuActive }"
        @click="insertTable"
      >
        <span class="rich-doc-toolbar__drop-main" title="插入表格">
          <Icon icon="mdi:table" width="16" height="16" />
        </span>
        <template #icon>
          <Icon icon="mdi:chevron-down" width="14" height="14" />
        </template>
        <template #overlay>
          <a-menu :items="tableMenu.items" @click="onTableMenuClick" />
        </template>
      </a-dropdown-button>

      <button
        v-else-if="item.kind === 'btn'"
        type="button"
        class="rich-doc-toolbar__btn"
        :title="item.title"
        :class="{ 'is-active': item.active?.() }"
        :disabled="item.disabled?.()"
        @click="item.run()"
      >
        <Icon v-if="item.icon" :icon="item.icon" width="16" height="16" />
        <template v-else>{{ item.label }}</template>
      </button>
    </template>
  </div>
</template>

<style scoped lang="scss">
.rich-doc-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 2px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--app-border);
  background: var(--app-surface-muted);

  :deep(.rich-doc-toolbar__btn) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 28px;
    height: 28px;
    padding: 0 6px;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: var(--app-fg-subtle);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;

    &:hover:not(:disabled) {
      background: var(--app-hover-bg);
    }

    &:disabled {
      opacity: 0.35;
      cursor: not-allowed;
    }

    &.is-active {
      background: var(--app-active-bg);
      color: var(--app-active-fg);
    }
  }
}

.rich-doc-toolbar__sep {
  width: 1px;
  height: 16px;
  margin: 0 4px;
  background: var(--app-divider);
}

.rich-doc-toolbar__size-btn {
  gap: 2px;
  min-width: 52px;
}

.rich-doc-toolbar__font-btn {
  gap: 3px;
  min-width: 64px;
  max-width: 96px;

  span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.rich-doc-toolbar__color-btn {
  position: relative;
  padding-bottom: 4px;
}

.rich-doc-toolbar__color-bar {
  position: absolute;
  bottom: 4px;
  left: 6px;
  right: 6px;
  height: 2px;
  border-radius: 1px;
}

.rich-doc-toolbar__drop {
  display: inline-flex;
  align-items: stretch;
  height: 28px;
  border-radius: 6px;
  overflow: visible;

  :deep(.ant-btn) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 28px;
    min-width: 28px;
    padding: 0 6px;
    border: none;
    border-radius: 0 !important;
    background: transparent;
    color: var(--app-fg-subtle);
    box-shadow: none;
    cursor: pointer;

    &:hover,
    &:focus {
      background: var(--app-hover-bg);
      color: var(--app-fg);
    }
  }

  &:hover :deep(.ant-btn) {
    background: var(--app-hover-bg);
    color: var(--app-fg);
  }

  :deep(.ant-btn:first-child) {
    border-radius: 6px 0 0 6px !important;
  }

  :deep(.ant-btn:last-child),
  :deep(.ant-dropdown-trigger) {
    min-width: 18px;
    padding: 0 4px;
    border-radius: 0 6px 6px 0 !important;
  }

  &.is-active {
    background: var(--app-active-bg);

    :deep(.ant-btn) {
      background: transparent;
      color: var(--app-active-fg);
    }

    &:hover :deep(.ant-btn) {
      background: transparent;
      color: var(--app-active-fg);
    }
  }
}

.rich-doc-toolbar__drop-main {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
</style>

<style lang="scss">
.rich-doc-toolbar__palette {
  min-width: 180px;
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid var(--app-border);
  background: var(--app-surface);
  box-shadow: 0 6px 20px rgb(0 0 0 / 12%);
}

.rich-doc-toolbar__palette-title {
  margin: 0 0 6px;
  color: var(--app-fg-muted);
  font-size: 11px;
  font-weight: 600;
}

.rich-doc-toolbar__swatches {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.rich-doc-toolbar__swatch {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  padding: 0;
  border: 1px solid var(--app-border);
  border-radius: 4px;
  background: var(--app-surface-muted);
  color: var(--app-fg-subtle);
  cursor: pointer;

  &.is-default {
    background: var(--app-surface);
  }

  &.is-active {
    outline: 2px solid var(--app-active-fg);
    outline-offset: 1px;
  }

  &:hover {
    filter: brightness(0.96);
  }
}
</style>
