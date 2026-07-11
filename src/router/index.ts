import { createRouter, createWebHashHistory } from "vue-router";
import { ROUTE_CHANGE_EVENT } from "@/router/events";
import { pageRoutes } from "@/router/pages";
import { standaloneRoutes, UNLOCK_ROUTE_PATH } from "@/router/standalone";
import { isAppSessionLocked } from "@/modules/appLock";

export { ROUTE_CHANGE_EVENT } from "@/router/events";
export { pageRoutes, EDITOR_ROUTE_PATH, CODE_SNIPPETS_ROUTE_PATH } from "@/router/pages";
export {
  standaloneRoutes,
  PREFERENCES_ROUTE_PATH,
  UNLOCK_ROUTE_PATH,
} from "@/router/standalone";

/** 聚合路由：Layout 子路由见 router/pages.ts，独立窗口见 router/standalone.ts */
export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      component: () => import("@/components/Layout/index.vue"),
      children: [
        { path: "", redirect: "/editor" },
        ...pageRoutes,
      ],
    },
    ...standaloneRoutes,
  ],
});

router.beforeEach(async (to) => {
  if (to.path === UNLOCK_ROUTE_PATH) {
    return true;
  }

  try {
    if (await isAppSessionLocked()) {
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
