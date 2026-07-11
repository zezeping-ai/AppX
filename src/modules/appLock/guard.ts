import { getAppLockSettings } from "./client";

/** 应用锁已启用且当前会话处于锁定状态 */
export async function isAppSessionLocked(): Promise<boolean> {
  const settings = await getAppLockSettings();
  return settings.enabled && settings.sessionLocked;
}
