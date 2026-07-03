import { invoke } from "@tauri-apps/api/core";
import type { AppLockSettingsView, SaveAppLockSettingsInput } from "./types";

export async function getAppLockSettings(): Promise<AppLockSettingsView> {
  return invoke<AppLockSettingsView>("app_lock_get_settings");
}

export async function saveAppLockSettings(
  input: SaveAppLockSettingsInput,
): Promise<AppLockSettingsView> {
  return invoke<AppLockSettingsView>("app_lock_save_settings", { input });
}

export async function lockAppLockSession(): Promise<AppLockSettingsView> {
  return invoke<AppLockSettingsView>("app_lock_lock_session");
}

export async function unlockAppLockSession(): Promise<AppLockSettingsView> {
  return invoke<AppLockSettingsView>("app_lock_unlock_session");
}

