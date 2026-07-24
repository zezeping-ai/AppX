import { invoke } from "@tauri-apps/api/core";
import type {
  EditorTreeNode,
  FileInspect,
  UnlockEncryptedFileResult,
} from "@/modules/editor/types";

export async function pickFolder(): Promise<string | null> {
  return invoke<string | null>("editor_pick_folder");
}

export async function pickFile(): Promise<string | null> {
  return invoke<string | null>("editor_pick_file");
}

export async function listDirectory(
  path: string,
  workspaceRoot?: string | null,
): Promise<EditorTreeNode[]> {
  return invoke<EditorTreeNode[]>("editor_list_directory", {
    path,
    workspaceRoot: workspaceRoot ?? null,
  });
}

export async function inspectFile(path: string): Promise<FileInspect> {
  return invoke<FileInspect>("editor_inspect_file", { path });
}

export async function readFile(path: string): Promise<string> {
  return invoke<string>("editor_read_file", { path });
}

export async function writeFile(path: string, content: string): Promise<void> {
  return invoke("editor_write_file", { path, content });
}

export async function unlockEncryptedFile(
  path: string,
  passphrase: string,
): Promise<UnlockEncryptedFileResult> {
  return invoke<UnlockEncryptedFileResult>("editor_unlock_encrypted_file", {
    path,
    passphrase,
  });
}

export async function createFile(
  directory: string,
  options?: {
    fileName?: string;
    encrypted?: boolean;
    content?: string;
  },
): Promise<string> {
  return invoke<string>("editor_create_file", {
    directory,
    fileName: options?.fileName,
    encrypted: options?.encrypted,
    content: options?.content,
  });
}

export async function createDirectory(directory: string, folderName: string): Promise<string> {
  return invoke<string>("editor_create_directory", { directory, folderName });
}

export async function deletePath(path: string): Promise<void> {
  return invoke("editor_delete_path", { path });
}

export async function renamePath(path: string, newName: string): Promise<string> {
  return invoke<string>("editor_rename_path", { path, newName });
}

export async function convertToEncryptedFile(path: string): Promise<string> {
  return invoke<string>("editor_convert_to_encrypted", { path });
}

export async function convertToCustomEncryptedFile(
  path: string,
  passphrase: string,
): Promise<string> {
  return invoke<string>("editor_convert_to_custom_encrypted", { path, passphrase });
}

export async function convertCustomToDefaultEncryptedFile(path: string): Promise<string> {
  return invoke<string>("editor_convert_custom_to_default_encrypted", { path });
}

export async function convertToPlainFile(path: string): Promise<string> {
  return invoke<string>("editor_convert_to_plain", { path });
}
