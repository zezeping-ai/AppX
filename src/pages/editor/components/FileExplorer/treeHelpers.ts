import type { ExplorerTreeItem } from "@/pages/editor/components/FileExplorer/normalizeTree";

/** 目录尚未拉取子节点 */
export function isDirectoryUnloaded(node: ExplorerTreeItem): boolean {
  return node.kind === "directory" && node.children === undefined;
}

export function findNode(
  nodes: ExplorerTreeItem[],
  path: string,
): ExplorerTreeItem | null {
  for (const node of nodes) {
    if (node.path === path) {
      return node;
    }
    if (node.children) {
      const matched = findNode(node.children, path);
      if (matched) {
        return matched;
      }
    }
  }
  return null;
}

/** 替换某目录的 children（不可变更新） */
export function setDirectoryChildren(
  nodes: ExplorerTreeItem[],
  directoryPath: string,
  children: ExplorerTreeItem[],
): ExplorerTreeItem[] {
  return nodes.map((node) => {
    if (node.path === directoryPath) {
      return { ...node, children };
    }
    if (node.children) {
      return {
        ...node,
        children: setDirectoryChildren(node.children, directoryPath, children),
      };
    }
    return node;
  });
}

export function pathSep(samplePath: string): "/" | "\\" {
  return samplePath.includes("\\") ? "\\" : "/";
}

export function joinPath(parent: string, name: string): string {
  const sep = pathSep(parent);
  if (parent.endsWith("/") || parent.endsWith("\\")) {
    return `${parent}${name}`;
  }
  return `${parent}${sep}${name}`;
}

/** `root` 到 `filePath` 之间的祖先目录（不含 root，由近到远） */
export function ancestorDirectories(root: string, filePath: string): string[] {
  const sep = pathSep(root);
  const normalizedRoot = root.endsWith(sep) ? root.slice(0, -1) : root;
  if (filePath !== normalizedRoot && !filePath.startsWith(normalizedRoot + sep)) {
    return [];
  }
  const relative = filePath.slice(normalizedRoot.length).replace(/^[\\/]/, "");
  const parts = relative.split(/[/\\]/).filter(Boolean);
  if (parts.length <= 1) {
    return [];
  }
  parts.pop();
  const dirs: string[] = [];
  let current = normalizedRoot;
  for (const part of parts) {
    current = joinPath(current, part);
    dirs.push(current);
  }
  return dirs;
}

/**
 * 展开键是否仍应保留：
 * - 节点仍在树中；或
 * - 某祖先目录尚未加载（懒加载未完成，不能当删除）
 */
export function shouldKeepExpandedKey(
  key: string,
  nodes: ExplorerTreeItem[],
  workspaceRoot: string | null | undefined,
): boolean {
  if (!workspaceRoot) {
    return false;
  }
  const sep = pathSep(workspaceRoot);
  const root = workspaceRoot.endsWith(sep)
    ? workspaceRoot.slice(0, -1)
    : workspaceRoot;
  if (key !== root && !key.startsWith(root + sep)) {
    return false;
  }
  if (findNode(nodes, key)) {
    return true;
  }
  return hasUnloadedAncestorOnPath(nodes, key);
}

function hasUnloadedAncestorOnPath(
  nodes: ExplorerTreeItem[],
  targetPath: string,
): boolean {
  for (const node of nodes) {
    if (node.kind !== "directory") {
      continue;
    }
    const sep = pathSep(node.path);
    const isPrefix =
      targetPath === node.path || targetPath.startsWith(node.path + sep);
    if (!isPrefix) {
      continue;
    }
    if (node.children === undefined) {
      return true;
    }
    if (hasUnloadedAncestorOnPath(node.children, targetPath)) {
      return true;
    }
  }
  return false;
}
