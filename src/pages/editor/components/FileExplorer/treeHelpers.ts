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

export function parentDirectory(
  path: string,
  workspaceRoot?: string | null,
): string | null {
  const index = Math.max(path.lastIndexOf("/"), path.lastIndexOf("\\"));
  if (index > 0) {
    return path.slice(0, index);
  }
  return workspaceRoot ?? null;
}

export function isSameOrDescendantPath(ancestor: string, path: string): boolean {
  if (ancestor === path) {
    return true;
  }
  const sep = pathSep(ancestor);
  return path.startsWith(ancestor + sep);
}

/**
 * 是否可将 source 拖入 targetDirectory（已在同目录则视为无效，避免无意义高亮）
 */
export function canDropIntoDirectory(
  sourcePath: string,
  targetDirectory: string,
): boolean {
  if (!sourcePath || !targetDirectory || sourcePath === targetDirectory) {
    return false;
  }
  if (isSameOrDescendantPath(sourcePath, targetDirectory)) {
    return false;
  }
  const sourceParent = parentDirectory(sourcePath);
  if (sourceParent === targetDirectory) {
    return false;
  }
  return true;
}

/** 路径变更后重映射（文件或整棵子树） */
export function remapPathAfterMove(
  oldPath: string,
  newPath: string,
  path: string,
): string | null {
  if (path === oldPath) {
    return newPath;
  }
  if (isSameOrDescendantPath(oldPath, path)) {
    return newPath + path.slice(oldPath.length);
  }
  return null;
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
