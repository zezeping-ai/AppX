import { createRouter, createWebHashHistory } from "vue-router";
import Home from "@/pages/home/index.vue";

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "home",
      component: Home,
    },
  ],
});
