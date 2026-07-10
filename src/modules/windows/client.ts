import { invoke } from "@tauri-apps/api/core";

/** 显示偏好设置独立窗口（Rust: app/windows） */
export async function showPreferencesWindow(): Promise<void> {
  return invoke("window_show_preferences");
}
