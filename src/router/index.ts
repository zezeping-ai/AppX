import { createRouter, createWebHashHistory } from "vue-router";
import { editorRoutes } from "@/features/editor/routes";
import { preferencesRoutes } from "@/features/preferences/routes";

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
    ...preferencesRoutes,
  ],
});

router.afterEach(() => {
  window.dispatchEvent(new Event(ROUTE_CHANGE_EVENT));
});
