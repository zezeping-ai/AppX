export interface AppLockSettingsView {
  enabled: boolean;
  lockOnStartup: boolean;
  lockOnWindowShow: boolean;
  sessionLocked: boolean;
}

export interface SaveAppLockSettingsInput {
  enabled: boolean;
  lockOnStartup: boolean;
  lockOnWindowShow: boolean;
}

