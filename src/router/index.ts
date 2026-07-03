import { createRouter, createWebHashHistory } from "vue-router";
import { editorRoutes } from "@/features/editor/routes";
import { preferencesRoutes } from "@/features/preferences/routes";
import { appLockRoutes, UNLOCK_ROUTE_PATH } from "@/features/appLock/routes";
import { getAppLockSettings } from "@/modules/appLock";

export const ROUTE_CHANGE_EVENT = "appx:route-change";

/** 聚合各功能模块路由；新增功能时在 features/<name>/routes.ts 定义并在此注册 */
export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      component: () => import("@/components/Layout/index.vue"),
      children: [
        { path: "", redirect: "/editor" },
        ...editorRoutes,
      ],
    },
    ...appLockRoutes,
    ...preferencesRoutes,
  ],
});

router.beforeEach(async (to) => {
  if (to.path === UNLOCK_ROUTE_PATH) {
    return true;
  }
  if (!to.path.startsWith("/editor")) {
    return true;
  }

  try {
    const settings = await getAppLockSettings();
    if (settings.enabled && settings.lockOnStartup && settings.sessionLocked) {
      return {
        path: UNLOCK_ROUTE_PATH,
        query: { redirect: to.fullPath },
      };
    }
  } catch {
    // 获取不到设置时，不阻断进入；避免在异常情况下把用户锁死。
  }

  return true;
});

router.afterEach(() => {
  window.dispatchEvent(new Event(ROUTE_CHANGE_EVENT));
});
