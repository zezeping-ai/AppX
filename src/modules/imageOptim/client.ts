import { invoke } from "@tauri-apps/api/core";
import type {
  ImageOptimSettings,
  OptimizeFileInput,
  OptimizeResult,
} from "@/modules/imageOptim/types";

export async function pickFiles(): Promise<string[]> {
  return invoke<string[]>("image_optim_pick_files");
}

export async function pickFolder(): Promise<string | null> {
  return invoke<string | null>("image_optim_pick_folder");
}

export async function expandPaths(paths: string[]): Promise<string[]> {
  return invoke<string[]>("image_optim_expand_paths", { paths });
}

export async function optimizeFile(input: OptimizeFileInput): Promise<OptimizeResult> {
  return invoke<OptimizeResult>("image_optim_optimize_file", { input });
}

export async function getSettings(): Promise<ImageOptimSettings> {
  return invoke<ImageOptimSettings>("image_optim_get_settings");
}

export async function saveSettings(
  settings: ImageOptimSettings,
): Promise<ImageOptimSettings> {
  return invoke<ImageOptimSettings>("image_optim_save_settings", { settings });
}
