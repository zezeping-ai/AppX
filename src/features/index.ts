import type { AppRouteRecord } from "@/router/types";
import { appLockRoutes } from "./appLock/routes";
import { editorRoutes } from "./editor/routes";
import { preferencesRoutes } from "./preferences/routes";

export { appLockRoutes, UNLOCK_ROUTE_PATH } from "./appLock";
export { editorRoutes, EDITOR_ROUTE_PATH } from "./editor";
export { preferencesRoutes, PREFERENCES_ROUTE_PATH } from "./preferences";
export { useThemePreferences } from "./appearance";
export type { ThemeMode } from "./appearance";

/** 主窗口 Layout 子路由 */
export const layoutChildRoutes: AppRouteRecord[] = [...editorRoutes];

/** 独立窗口 / 顶层路由 */
export const standaloneRoutes: AppRouteRecord[] = [...appLockRoutes, ...preferencesRoutes];
