import type { RouteRecordRaw } from "vue-router";

/** 路由 meta：按功能模块标记，便于守卫与布局扩展 */
export type AppRouteMeta = {
  /** 所属功能模块标识 */
  feature?: string;
  /** 独立窗口路由（如偏好设置），不参与主窗口布局 */
  standalone?: boolean;
};

export type AppRouteRecord = RouteRecordRaw & {
  meta?: AppRouteMeta;
};
