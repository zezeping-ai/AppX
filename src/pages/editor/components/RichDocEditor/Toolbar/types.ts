export type ToolbarSep = { kind: "sep" };

export type ToolbarMenuKind =
  | "labelMenu"
  | "tableMenu"
  | "colorMenu"
  | "highlightMenu"
  | "underlineMenu"
  | "fontFamilyMenu"
  | "fontSizeMenu";

export type ToolbarMenu = { kind: ToolbarMenuKind };

export type ToolbarBtn = {
  kind: "btn";
  title: string;
  icon?: string;
  label?: string;
  active?: () => boolean;
  disabled?: () => boolean;
  run: () => void;
};

export type ToolbarItem = ToolbarSep | ToolbarBtn | ToolbarMenu;
