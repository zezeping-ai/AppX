import { invoke } from "@tauri-apps/api/core";

export async function showPreferencesWindow(): Promise<void> {
  return invoke("window_show_preferences");
}
