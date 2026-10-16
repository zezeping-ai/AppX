import { Icon } from "@/components/Icon";
import { h } from "vue";

/** 工具栏下拉菜单统一图标渲染 */
export function menuIcon(name: string) {
  return () => h(Icon, { icon: name, size: 15 });
}
