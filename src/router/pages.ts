import type { AppRouteRecord } from "@/router/types";

export const EDITOR_ROUTE_PATH = "/editor";
export const CODE_SNIPPETS_ROUTE_PATH = "/code-snippets";
export const CLIPBOARD_ASSISTANT_ROUTE_PATH = "/clipboard-assistant";

/** 主窗口 Layout 子路由；页面组件在 src/pages/<name>/，在此注册 */
export const pageRoutes: AppRouteRecord[] = [
  {
    path: "editor",
    name: "editor",
    component: () => import("@/pages/editor/index.vue"),
    meta: { feature: "editor" },
  },
  {
    path: "code-snippets",
    name: "code-snippets",
    component: () => import("@/pages/code-snippets/index.vue"),
    meta: { feature: "code-snippets" },
  },
  {
    path: "clipboard-assistant",
    name: "clipboard-assistant",
    component: () => import("@/pages/clipboard-assistant/index.vue"),
    meta: { feature: "clipboard-assistant" },
  },
];
