import type { AppRouteRecord } from "@/router/types";

export const EDITOR_ROUTE_PATH = "/editor";

/** Editor 工作区路由（作为 Layout 子路由） */
export const editorRoutes: AppRouteRecord[] = [
  {
    path: "editor",
    name: "editor",
    component: () => import("./pages/WorkspacePage.vue"),
    meta: { feature: "editor" },
  },
];
