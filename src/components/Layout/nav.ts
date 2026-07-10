import type { AppNavItem } from "./types";

/** 主窗口功能导航；新增功能时在此注册 */
export const APP_NAV_ITEMS: AppNavItem[] = [
  {
    key: "editor",
    label: "Editor",
    to: "/editor",
    icon: "mdi:application-edit-outline",
    enabled: true,
  },
  {
    key: "code-snippets",
    label: "代码段",
    to: "/code-snippets",
    icon: "mdi:lightning-bolt-outline",
    enabled: true,
  },
];
