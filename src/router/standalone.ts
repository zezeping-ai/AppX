import type { AppRouteRecord } from "@/router/types";

export const PREFERENCES_ROUTE_PATH = "/preferences";
export const UNLOCK_ROUTE_PATH = "/unlock";
export const SNIPPET_PALETTE_ROUTE_PATH = "/snippet-palette";

/** 独立窗口 / 全屏流程页（不走主 Layout） */
export const standaloneRoutes: AppRouteRecord[] = [
  {
    path: PREFERENCES_ROUTE_PATH,
    name: "preferences",
    component: () => import("@/pages/preferences/index.vue"),
    meta: { feature: "preferences", standalone: true },
  },
  {
    path: UNLOCK_ROUTE_PATH,
    name: "unlock",
    component: () => import("@/pages/unlock/index.vue"),
    meta: { feature: "appLock", standalone: true },
  },
  {
    path: SNIPPET_PALETTE_ROUTE_PATH,
    name: "snippet-palette",
    component: () => import("@/pages/snippet-palette/index.vue"),
    meta: { feature: "snippet-palette", standalone: true },
  },
];
