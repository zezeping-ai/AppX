export const ENCRYPTED_FILE_PATTERN = "*.{lang}.x";

export type EditorNodeKind = "directory" | "file";

export interface EditorTreeNode {
  name: string;
  path: string;
  kind: EditorNodeKind;
  encrypted?: boolean;
  language?: string;
  children?: EditorTreeNode[];
}

export interface OpenedEditorFile {
  path: string;
  name: string;
  content: string;
  language: string;
  encrypted: boolean;
  dirty: boolean;
}

export interface FileInspect {
  path: string;
  encrypted: boolean;
  language: string;
  editable: boolean;
}
