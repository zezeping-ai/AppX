import { invoke } from "@tauri-apps/api/core";
import type {
  CodeSnippetPermissionsView,
  CodeSnippetSettingsView,
  CodeSnippetSyncItem,
} from "@/modules/codeSnippets/types";

export async function syncSnippetRegistry(snippets: CodeSnippetSyncItem[]): Promise<void> {
  await invoke("code_snippets_sync", { snippets });
}

export async function setExpansionPaused(paused: boolean): Promise<void> {
  await invoke("code_snippets_set_expansion_paused", { paused });
}

export async function getCodeSnippetPermissions(): Promise<CodeSnippetPermissionsView> {
  return invoke<CodeSnippetPermissionsView>("code_snippets_get_permissions");
}

export async function openAccessibilitySettings(): Promise<void> {
  await invoke("code_snippets_open_accessibility_settings");
}

export async function getCodeSnippetSettings(): Promise<CodeSnippetSettingsView> {
  return invoke<CodeSnippetSettingsView>("code_snippets_get_settings");
}

export async function saveCodeSnippetSettings(input: {
  enabled: boolean;
  inlineExpansionEnabled: boolean;
  inlineExpansionTrigger: string;
  shortcutsEnabled: boolean;
  paletteEnabled: boolean;
}): Promise<CodeSnippetSettingsView> {
  return invoke<CodeSnippetSettingsView>("code_snippets_save_settings", { input });
}
