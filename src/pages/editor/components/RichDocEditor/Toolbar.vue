<script setup lang="ts">
import { Icon } from "@iconify/vue";
import type { Editor } from "@tiptap/vue-3";
import type { MenuProps } from "ant-design-vue";
import { computed, h } from "vue";

const props = defineProps<{
  editor: Editor;
}>();

type ToolbarSep = { kind: "sep" };
type ToolbarMenu = { kind: "labelMenu" } | { kind: "tableMenu" };
type ToolbarBtn = {
  kind: "btn";
  title: string;
  icon?: string;
  label?: string;
  active?: () => boolean;
  disabled?: () => boolean;
  run: () => void;
};
type ToolbarItem = ToolbarSep | ToolbarBtn | ToolbarMenu;

function chain() {
  return props.editor.chain().focus();
}

function menuIcon(name: string) {
  return () => h(Icon, { icon: name, width: 15, height: 15 });
}

const inTable = computed(() => props.editor.isActive("table"));

const labelMenuActive = computed(
  () => props.editor.isActive("label") || props.editor.isActive("labelValue"),
);

const tableMenuActive = computed(() => inTable.value);

const labelMenu = computed<MenuProps>(() => ({
  items: [
    {
      key: "label",
      label: "标签",
      icon: menuIcon("mdi:label-outline"),
    },
    {
      key: "labelValue",
      label: "标签值",
      icon: menuIcon("mdi:form-textbox"),
    },
  ],
  onClick: ({ key }) => {
    if (key === "label") {
      chain().insertLabel().run();
      return;
    }
    if (key === "labelValue") {
      chain().insertLabelValue().run();
    }
  },
}));

const tableMenu = computed<MenuProps>(() => {
  const inside = inTable.value;
  const ed = props.editor;

  return {
    items: [
      {
        key: "insert",
        label: "插入表格",
        icon: menuIcon("mdi:table-plus"),
      },
      {
        key: "delete",
        label: "删除表格",
        icon: menuIcon("mdi:table-remove"),
        disabled: !inside,
      },
      {
        key: "header",
        label: "切换表头行",
        icon: menuIcon("mdi:table-headers-eye"),
        disabled: !inside,
      },
      { type: "divider" },
      {
        key: "row-before",
        label: "上方插入行",
        icon: menuIcon("mdi:table-row-plus-before"),
        disabled: !inside,
      },
      {
        key: "row-after",
        label: "下方插入行",
        icon: menuIcon("mdi:table-row-plus-after"),
        disabled: !inside,
      },
      {
        key: "row-remove",
        label: "删除行",
        icon: menuIcon("mdi:table-row-remove"),
        disabled: !inside,
      },
      { type: "divider" },
      {
        key: "col-before",
        label: "左侧插入列",
        icon: menuIcon("mdi:table-column-plus-before"),
        disabled: !inside,
      },
      {
        key: "col-after",
        label: "右侧插入列",
        icon: menuIcon("mdi:table-column-plus-after"),
        disabled: !inside,
      },
      {
        key: "col-remove",
        label: "删除列",
        icon: menuIcon("mdi:table-column-remove"),
        disabled: !inside,
      },
      { type: "divider" },
      {
        key: "merge",
        label: "合并单元格",
        icon: menuIcon("mdi:table-merge-cells"),
        disabled: !ed.can().mergeCells(),
      },
      {
        key: "split",
        label: "拆分单元格",
        icon: menuIcon("mdi:table-split-cell"),
        disabled: !ed.can().splitCell(),
      },
    ],
    onClick: ({ key }) => {
      const run: Record<string, () => void> = {
        insert: () =>
          chain().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
        delete: () => chain().deleteTable().run(),
        header: () => chain().toggleHeaderRow().run(),
        "row-before": () => chain().addRowBefore().run(),
        "row-after": () => chain().addRowAfter().run(),
        "row-remove": () => chain().deleteRow().run(),
        "col-before": () => chain().addColumnBefore().run(),
        "col-after": () => chain().addColumnAfter().run(),
        "col-remove": () => chain().deleteColumn().run(),
        merge: () => chain().mergeCells().run(),
        split: () => chain().splitCell().run(),
      };
      run[String(key)]?.();
    },
  };
});

const items = computed<ToolbarItem[]>(() => {
  const ed = props.editor;

  return [
    {
      kind: "btn",
      title: "加粗",
      icon: "mdi:format-bold",
      active: () => ed.isActive("bold"),
      run: () => chain().toggleBold().run(),
    },
    {
      kind: "btn",
      title: "斜体",
      icon: "mdi:format-italic",
      active: () => ed.isActive("italic"),
      run: () => chain().toggleItalic().run(),
    },
    {
      kind: "btn",
      title: "删除线",
      icon: "mdi:format-strikethrough",
      active: () => ed.isActive("strike"),
      run: () => chain().toggleStrike().run(),
    },
    { kind: "sep" },
    {
      kind: "btn",
      title: "标题 1",
      label: "H1",
      active: () => ed.isActive("heading", { level: 1 }),
      run: () => chain().toggleHeading({ level: 1 }).run(),
    },
    {
      kind: "btn",
      title: "标题 2",
      label: "H2",
      active: () => ed.isActive("heading", { level: 2 }),
      run: () => chain().toggleHeading({ level: 2 }).run(),
    },
    {
      kind: "btn",
      title: "标题 3",
      label: "H3",
      active: () => ed.isActive("heading", { level: 3 }),
      run: () => chain().toggleHeading({ level: 3 }).run(),
    },
    { kind: "sep" },
    {
      kind: "btn",
      title: "无序列表",
      icon: "mdi:format-list-bulleted",
      active: () => ed.isActive("bulletList"),
      run: () => chain().toggleBulletList().run(),
    },
    {
      kind: "btn",
      title: "有序列表",
      icon: "mdi:format-list-numbered",
      active: () => ed.isActive("orderedList"),
      run: () => chain().toggleOrderedList().run(),
    },
    {
      kind: "btn",
      title: "引用",
      icon: "mdi:format-quote-close",
      active: () => ed.isActive("blockquote"),
      run: () => chain().toggleBlockquote().run(),
    },
    {
      kind: "btn",
      title: "代码块",
      icon: "mdi:code-tags",
      active: () => ed.isActive("codeBlock"),
      run: () => chain().toggleCodeBlock().run(),
    },
    { kind: "labelMenu" },
    { kind: "sep" },
    { kind: "tableMenu" },
  ];
});

function insertLabel() {
  chain().insertLabel().run();
}

function insertTable() {
  chain().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
}
</script>

<template>
  <div class="rich-doc-toolbar">
    <template v-for="(item, index) in items" :key="index">
      <span v-if="item.kind === 'sep'" class="rich-doc-toolbar__sep" />

      <a-dropdown-button
        v-else-if="item.kind === 'labelMenu'"
        trigger="click"
        placement="bottomLeft"
        :menu="labelMenu"
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
      </a-dropdown-button>

      <a-dropdown-button
        v-else-if="item.kind === 'tableMenu'"
        trigger="click"
        placement="bottomLeft"
        :menu="tableMenu"
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
      </a-dropdown-button>

      <button
        v-else
        type="button"
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

  > button {
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

.rich-doc-toolbar__drop {
  display: inline-flex;
  align-items: stretch;
  height: 28px;
  border-radius: 6px;
  overflow: hidden;

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

  :deep(.ant-btn:last-child),
  :deep(.ant-dropdown-trigger) {
    min-width: 18px;
    padding: 0 4px;
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
