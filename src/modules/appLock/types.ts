export interface AppLockSettingsView {
  enabled: boolean;
  lockOnStartup: boolean;
  lockOnWindowShow: boolean;
  sessionLocked: boolean;
  /** 是否已配置解锁 PIN（后端校验所需） */
  hasUnlockPin: boolean;
}

export interface SaveAppLockSettingsInput {
  enabled: boolean;
  lockOnStartup: boolean;
  lockOnWindowShow: boolean;
  /** 设置或更换解锁 PIN；省略则不改动 */
  unlockPin?: string;
}

export interface UnlockSessionInput {
  pin: string;
}
