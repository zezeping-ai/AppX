export const ENCRYPTED_FILE_PATTERN = "*.{lang}.x";
export const CUSTOM_ENCRYPTED_FILE_PATTERN = "*.{lang}.x0";
export const DECRYPT_PASSPHRASE_REQUIRED = "DECRYPT_PASSPHRASE_REQUIRED";

export type EditorNodeKind = "directory" | "file";

export interface EditorTreeNode {
  name: string;
  path: string;
  kind: EditorNodeKind;
  encrypted?: boolean;
  customEncrypted?: boolean;
  language?: string;
  /** 点文件 / 点目录 */
  hidden?: boolean;
  /** 被 .gitignore 忽略 */
  ignored?: boolean;
  /** 目录未加载时为 null/undefined；已加载空目录为 [] */
  children?: EditorTreeNode[] | null;
}

/** 工作区扁平文件（Cmd+P 快速打开） */
export interface EditorFileEntry {
  name: string;
  path: string;
  /** 相对工作区根，统一 `/` */
  relativePath: string;
  encrypted: boolean;
  customEncrypted: boolean;
  language: string;
  hidden: boolean;
}

/** 工作区内容搜索（Cmd+Shift+F） */
export interface WorkspaceSearchMatch {
  line: number;
  column: number;
  preview: string;
  matchStart: number;
  matchEnd: number;
}

export interface WorkspaceSearchFile {
  path: string;
  relativePath: string;
  name: string;
  language: string;
  matches: WorkspaceSearchMatch[];
}

export interface WorkspaceSearchResult {
  files: WorkspaceSearchFile[];
  matchCount: number;
  filesScanned: number;
  truncated: boolean;
  skippedLocked: number;
}

/** 全局搜索选中一条匹配 */
export interface SearchInFilesSelectPayload {
  path: string;
  line: number;
  column: number;
  language: string;
}

export interface OpenedEditorFile {
  path: string;
  name: string;
  content: string;
  language: string;
  encrypted: boolean;
  customEncrypted: boolean;
  dirty: boolean;
}

export interface FileInspect {
  path: string;
  encrypted: boolean;
  customEncrypted: boolean;
  language: string;
  editable: boolean;
}

export interface UnlockEncryptedFileResult {
  path: string;
  content: string;
}

export function isDecryptPassphraseRequired(error: unknown): boolean {
  return String(error).startsWith(DECRYPT_PASSPHRASE_REQUIRED);
}
