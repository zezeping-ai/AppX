import { resolveExplorerFileIcon } from "@/pages/editor/components/FileExplorer/fileIcon";
import type { ExplorerFileIcon } from "@/pages/editor/components/FileExplorer/fileIcon";

export type PaletteFileIconInput = {
  path: string;
  name: string;
  language?: string;
  encrypted?: boolean;
  customEncrypted?: boolean;
  hidden?: boolean;
};

export function resolvePaletteFileIcon(file: PaletteFileIconInput): ExplorerFileIcon {
  return resolveExplorerFileIcon({
    key: file.path,
    title: file.name,
    path: file.path,
    kind: "file",
    language: file.language,
    encrypted: file.encrypted,
    customEncrypted: file.customEncrypted,
    hidden: file.hidden,
    isLeaf: true,
  });
}
