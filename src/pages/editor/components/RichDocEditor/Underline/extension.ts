import Underline from "@tiptap/extension-underline";

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    underlineColor: {
      /** 设置下划线颜色（同时启用下划线）；null 恢复自动色 */
      setUnderlineColor: (color: string | null) => ReturnType;
    };
  }
}

/**
 * 带颜色的下划线（Word：下划线颜色）。
 * color 写入 text-decoration-color；null 表示跟随文字色。
 */
export const RichDocUnderline = Underline.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      color: {
        default: null as string | null,
        parseHTML: (element: HTMLElement) =>
          element.style.textDecorationColor ||
          element.getAttribute("data-underline-color") ||
          null,
        renderHTML: (attributes: { color?: string | null }) => {
          if (!attributes.color) return {};
          return {
            "data-underline-color": attributes.color,
            style: `text-decoration-color: ${attributes.color}`,
          };
        },
      },
    };
  },

  addCommands() {
    return {
      ...this.parent?.(),
      setUnderlineColor:
        (color) =>
        ({ chain, editor }) => {
          if (color == null) {
            if (!editor.isActive("underline")) return true;
            return chain()
              .extendMarkRange("underline")
              .updateAttributes("underline", { color: null })
              .run();
          }
          return chain()
            .extendMarkRange("underline")
            .setMark("underline", { color })
            .run();
        },
    };
  },
});
