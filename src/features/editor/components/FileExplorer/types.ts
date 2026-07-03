export type InlineEditMode = "create-file" | "create-folder" | "rename";

export interface InlineEditState {
  mode: InlineEditMode;
  /** 新建时的父目录；重命名时为节点所在目录 */
  parentPath: string;
  /** 重命名目标路径；新建时与 parentPath 相同 */
  targetPath: string;
  value: string;
}

export type ContextMenuAction = {
  key: string;
  label: string;
  danger?: boolean;
  divider?: boolean;
};
