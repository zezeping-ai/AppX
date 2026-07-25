import { Icon } from "@iconify/vue";
import { h } from "vue";

/** 工具栏下拉菜单统一图标渲染 */
export function menuIcon(name: string) {
  return () => h(Icon, { icon: name, width: 15, height: 15 });
}
