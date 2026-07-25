import type { Editor } from "@tiptap/vue-3";
import type { MenuProps } from "ant-design-vue";
import { menuIcon } from "./menuIcon";

/** 表格操作下拉 */
export function createTableMenu(editor: Editor): MenuProps {
  const chain = () => editor.chain().focus();
  const inside = editor.isActive("table");

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
        disabled: !editor.can().mergeCells(),
      },
      {
        key: "split",
        label: "拆分单元格",
        icon: menuIcon("mdi:table-split-cell"),
        disabled: !editor.can().splitCell(),
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
}
