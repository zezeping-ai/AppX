import { invoke } from "@tauri-apps/api/core";

export type ClipboardSoundKind = "copy" | "paste";

/** 经 Rust 播放（afplay 等），不依赖 WebView Audio */
export async function playClipboardSound(
  kind: ClipboardSoundKind,
  options?: { enabled?: boolean; path?: string; force?: boolean },
): Promise<void> {
  if (options?.enabled === false) return;
  try {
    await invoke("clipboard_assistant_play_sound", {
      kind,
      path: options?.path?.trim() || null,
      force: options?.force ?? options?.enabled === true,
    });
  } catch {
    // 忽略播放失败，避免打断主流程
  }
}

export async function pickClipboardSoundFile(): Promise<string | null> {
  return invoke<string | null>("clipboard_assistant_pick_sound_file");
}

export function soundFileLabel(path: string | undefined): string {
  const trimmed = path?.trim();
  if (!trimmed) return "默认";
  const parts = trimmed.split(/[/\\]/);
  return parts[parts.length - 1] || trimmed;
}
