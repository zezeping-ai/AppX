import type { AppRouteRecord } from "@/router/types";

export const PREFERENCES_ROUTE_PATH = "/preferences";

/** 偏好设置（独立窗口）路由 */
export const preferencesRoutes: AppRouteRecord[] = [
  {
    path: PREFERENCES_ROUTE_PATH,
    name: "preferences",
    component: () => import("./pages/PreferencesPage.vue"),
    meta: { feature: "preferences", standalone: true },
  },
];
