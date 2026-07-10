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
  children?: EditorTreeNode[];
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
