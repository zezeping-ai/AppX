import { invoke } from "@tauri-apps/api/core";

/** 快捷键录制期间暂停全局快捷键，结束后恢复。 */
export async function setGlobalShortcutsPaused(paused: boolean): Promise<void> {
  await invoke("global_shortcuts_set_paused", { paused });
}
