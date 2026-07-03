import type { AppRouteRecord } from "@/router/types";

export const UNLOCK_ROUTE_PATH = "/unlock";

export const appLockRoutes: AppRouteRecord[] = [
  {
    path: UNLOCK_ROUTE_PATH,
    name: "unlock",
    component: () => import("./pages/UnlockPage.vue"),
    meta: { feature: "appLock", standalone: true },
  },
];

