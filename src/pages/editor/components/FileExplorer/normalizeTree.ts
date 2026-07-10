import type { EditorTreeNode } from "@/modules/editor/types";

export interface ExplorerTreeItem {
  key: string;
  title: string;
  path: string;
  kind: EditorTreeNode["kind"];
  encrypted?: boolean;
  customEncrypted?: boolean;
  language?: string;
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
    isLeaf: node.kind === "file",
    children: node.children ? toExplorerTree(node.children) : undefined,
  }));
}
