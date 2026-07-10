import { invoke } from "@tauri-apps/api/core";
import type { CodeSnippetPaletteItem } from "@/modules/codeSnippets/types";

export async function listPaletteItems(): Promise<CodeSnippetPaletteItem[]> {
  return invoke<CodeSnippetPaletteItem[]>("code_snippets_list_palette_items");
}

export async function insertPaletteItem(id: number): Promise<void> {
  await invoke("code_snippets_insert_palette_item", { id });
}

export async function copyPaletteItem(id: number): Promise<void> {
  await invoke("code_snippets_copy_palette_item", { id });
}

export async function hidePalette(): Promise<void> {
  await invoke("code_snippets_hide_palette");
}
