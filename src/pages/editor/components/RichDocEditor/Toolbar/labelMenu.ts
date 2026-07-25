import type { Editor } from "@tiptap/vue-3";
import type { MenuProps } from "ant-design-vue";
import { menuIcon } from "./menuIcon";

/** Label / LabelValue 下拉 */
export function createLabelMenu(editor: Editor): MenuProps {
  const chain = () => editor.chain().focus();

  return {
    items: [
      {
        key: "label",
        label: "标签",
        icon: menuIcon("mdi:label-outline"),
      },
      {
        key: "labelValue",
        label: "标签值",
        icon: menuIcon("mdi:tag-text-outline"),
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
  };
}
