export type AppNavItem = {
  key: string;
  label: string;
  to: string;
  icon: string;
  /** 是否已实现；未实现菜单不展示 */
  enabled?: boolean;
};
