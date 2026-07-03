import type { ComputedRef, InjectionKey } from "vue";

/** 根 a-config-provider 的 props 子集，供 createRoot 式 Modal/Drawer 继承主题与尺寸 */
export type AntdProgrammaticRootConfig = ComputedRef<Record<string, unknown>>;

export const antdProgrammaticRootConfigKey: InjectionKey<AntdProgrammaticRootConfig> = Symbol(
  "antdProgrammaticRootConfig",
);
