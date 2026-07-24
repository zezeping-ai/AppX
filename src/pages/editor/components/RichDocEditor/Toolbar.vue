<script setup lang="ts">
import { Icon } from "@iconify/vue";
import type { Editor } from "@tiptap/vue-3";
import { computed } from "vue";

const props = defineProps<{
  editor: Editor;
}>();

type ToolbarSep = { kind: "sep" };
type ToolbarBtn = {
  kind: "btn";
  title: string;
  icon?: string;
  label?: string;
  active?: () => boolean;
  disabled?: () => boolean;
  run: () => void;
};
type ToolbarItem = ToolbarSep | ToolbarBtn;

function chain() {
  return props.editor.chain().focus();
}

const items = computed<ToolbarItem[]>(() => {
  const ed = props.editor;
  const inTable = () => ed.isActive("table");

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
    { kind: "sep" },
    {
      kind: "btn",
      title: "插入表格",
      icon: "mdi:table-plus",
      run: () => chain().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run(),
    },
    {
      kind: "btn",
      title: "删除表格",
      icon: "mdi:table-remove",
      disabled: () => !inTable(),
      run: () => chain().deleteTable().run(),
    },
    {
      kind: "btn",
      title: "切换表头行",
      icon: "mdi:table-headers-eye",
      disabled: () => !inTable(),
      active: () => ed.isActive("tableHeader"),
      run: () => chain().toggleHeaderRow().run(),
    },
    { kind: "sep" },
    {
      kind: "btn",
      title: "上方插入行",
      icon: "mdi:table-row-plus-before",
      disabled: () => !inTable(),
      run: () => chain().addRowBefore().run(),
    },
    {
      kind: "btn",
      title: "下方插入行",
      icon: "mdi:table-row-plus-after",
      disabled: () => !inTable(),
      run: () => chain().addRowAfter().run(),
    },
    {
      kind: "btn",
      title: "删除行",
      icon: "mdi:table-row-remove",
      disabled: () => !inTable(),
      run: () => chain().deleteRow().run(),
    },
    { kind: "sep" },
    {
      kind: "btn",
      title: "左侧插入列",
      icon: "mdi:table-column-plus-before",
      disabled: () => !inTable(),
      run: () => chain().addColumnBefore().run(),
    },
    {
      kind: "btn",
      title: "右侧插入列",
      icon: "mdi:table-column-plus-after",
      disabled: () => !inTable(),
      run: () => chain().addColumnAfter().run(),
    },
    {
      kind: "btn",
      title: "删除列",
      icon: "mdi:table-column-remove",
      disabled: () => !inTable(),
      run: () => chain().deleteColumn().run(),
    },
    { kind: "sep" },
    {
      kind: "btn",
      title: "合并单元格（先拖选多个单元格）",
      icon: "mdi:table-merge-cells",
      disabled: () => !ed.can().mergeCells(),
      run: () => chain().mergeCells().run(),
    },
    {
      kind: "btn",
      title: "拆分单元格",
      icon: "mdi:table-split-cell",
      disabled: () => !ed.can().splitCell(),
      run: () => chain().splitCell().run(),
    },
  ];
});
</script>

<template>
  <div class="rich-doc-toolbar">
    <template v-for="(item, index) in items" :key="index">
      <span v-if="item.kind === 'sep'" class="rich-doc-toolbar__sep" />
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

  button {
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
</style>
