import { invoke } from "@tauri-apps/api/core";
import type {
  ClipboardAssistantSettings,
  ClipboardAssistantStatus,
  GetContentResult,
  ListItemsQuery,
  ListItemsResult,
  MutateOp,
  SaveClipboardAssistantSettingsInput,
} from "@/modules/clipboardAssistant/types";

export async function listItems(query: ListItemsQuery): Promise<ListItemsResult> {
  return invoke<ListItemsResult>("clipboard_assistant_list_items", { query });
}

export async function mutateItems(op: MutateOp, ids?: number[]): Promise<void> {
  await invoke("clipboard_assistant_mutate_items", { input: { op, ids } });
}

export async function getContent(id: number): Promise<GetContentResult> {
  return invoke<GetContentResult>("clipboard_assistant_get_content", { id });
}

export async function getSettings(): Promise<ClipboardAssistantSettings> {
  return invoke<ClipboardAssistantSettings>("clipboard_assistant_get_settings");
}

export async function saveSettings(
  input: SaveClipboardAssistantSettingsInput,
): Promise<void> {
  await invoke("clipboard_assistant_save_settings", { input });
}

export async function getStatus(): Promise<ClipboardAssistantStatus> {
  return invoke<ClipboardAssistantStatus>("clipboard_assistant_get_status");
}

export async function syncClipboardAssistantRuntime(): Promise<void> {
  await invoke("clipboard_assistant_sync_runtime");
}
