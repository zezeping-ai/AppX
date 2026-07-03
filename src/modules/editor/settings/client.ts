import { invoke } from "@tauri-apps/api/core";
import type { EditorSettingsView } from "@/modules/editor/settings/types";

export async function getEditorSettings(): Promise<EditorSettingsView> {
  return invoke<EditorSettingsView>("editor_get_settings");
}

export async function saveEditorEncryptionPassphrase(
  passphrase: string,
): Promise<EditorSettingsView> {
  return invoke<EditorSettingsView>("editor_save_encryption_passphrase", { passphrase });
}
