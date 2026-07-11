import { invoke } from "@tauri-apps/api/core";
import type { ApplyAction, PaletteGeometry } from "@/modules/clipboardAssistant/types";

/** 浮层 IPC（与 Rust palette 模块对应） */
export async function applyItem(
  id: number,
  action: ApplyAction,
  plainText?: boolean,
): Promise<void> {
  await invoke("clipboard_assistant_apply_item", {
    input: { id, action, plainText },
  });
}

export async function hidePalette(): Promise<void> {
  await invoke("clipboard_assistant_hide_palette");
}

export async function savePaletteGeometry(geometry: PaletteGeometry): Promise<void> {
  await invoke("clipboard_assistant_save_palette_geometry", { geometry });
}
