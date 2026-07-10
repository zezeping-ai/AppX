import type { FileInspect, OpenedEditorFile } from "@/modules/editor/types";

export function fileNameFromPath(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

export function createOpenedEditorFile(
  path: string,
  inspect: FileInspect,
  content: string,
): OpenedEditorFile {
  return {
    path,
    name: fileNameFromPath(path),
    content,
    language: inspect.language,
    encrypted: inspect.encrypted,
    customEncrypted: inspect.customEncrypted,
    dirty: false,
  };
}
