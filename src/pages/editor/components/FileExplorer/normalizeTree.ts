import type { EditorTreeNode } from "@/modules/editor/types";

export interface ExplorerTreeItem {
  key: string;
  title: string;
  path: string;
  kind: EditorTreeNode["kind"];
  encrypted?: boolean;
  customEncrypted?: boolean;
  language?: string;
  hidden?: boolean;
  ignored?: boolean;
  isLeaf: boolean;
  children?: ExplorerTreeItem[];
}

export function toExplorerTree(nodes: EditorTreeNode[]): ExplorerTreeItem[] {
  return nodes.map((node) => ({
    key: node.path,
    title: node.name,
    path: node.path,
    kind: node.kind,
    encrypted: node.encrypted,
    customEncrypted: node.customEncrypted,
    language: node.language,
    hidden: Boolean(node.hidden),
    ignored: Boolean(node.ignored),
    isLeaf: node.kind === "file",
    // 目录：null/undefined = 未加载；[] = 已加载空目录
    children:
      node.kind === "directory"
        ? node.children == null
          ? undefined
          : toExplorerTree(node.children)
        : undefined,
  }));
}
