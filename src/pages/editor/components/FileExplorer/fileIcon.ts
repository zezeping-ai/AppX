import type { ExplorerTreeItem } from "@/pages/editor/components/FileExplorer/normalizeTree";

export interface ExplorerFileIcon {
  icon: string;
  /** 未灰显时的图标色；灰显时由样式覆盖 */
  color?: string;
}

const BY_FILE_NAME: Record<string, ExplorerFileIcon> = {
  "package.json": { icon: "mdi:npm", color: "#cb3837" },
  "package-lock.json": { icon: "mdi:npm", color: "#cb3837" },
  "pnpm-lock.yaml": { icon: "mdi:npm", color: "#f69220" },
  "yarn.lock": { icon: "mdi:npm", color: "#2c8ebb" },
  "cargo.toml": { icon: "mdi:language-rust", color: "#dea584" },
  "cargo.lock": { icon: "mdi:language-rust", color: "#dea584" },
  "tsconfig.json": { icon: "mdi:language-typescript", color: "#3178c6" },
  "jsconfig.json": { icon: "mdi:language-javascript", color: "#f7df1e" },
  "readme.md": { icon: "mdi:language-markdown", color: "#083fa1" },
  dockerfile: { icon: "mdi:docker", color: "#2496ed" },
  "docker-compose.yml": { icon: "mdi:docker", color: "#2496ed" },
  "docker-compose.yaml": { icon: "mdi:docker", color: "#2496ed" },
  makefile: { icon: "mdi:file-cog-outline", color: "#6b7280" },
  ".gitignore": { icon: "mdi:git", color: "#f05032" },
  ".env": { icon: "mdi:cog-outline", color: "#eab308" },
  ".editorconfig": { icon: "mdi:cog-outline", color: "#6b7280" },
};

const BY_EXTENSION: Record<string, ExplorerFileIcon> = {
  ts: { icon: "mdi:language-typescript", color: "#3178c6" },
  mts: { icon: "mdi:language-typescript", color: "#3178c6" },
  cts: { icon: "mdi:language-typescript", color: "#3178c6" },
  tsx: { icon: "mdi:react", color: "#61dafb" },
  js: { icon: "mdi:language-javascript", color: "#f7df1e" },
  mjs: { icon: "mdi:language-javascript", color: "#f7df1e" },
  cjs: { icon: "mdi:language-javascript", color: "#f7df1e" },
  jsx: { icon: "mdi:react", color: "#61dafb" },
  vue: { icon: "mdi:vuejs", color: "#41b883" },
  json: { icon: "mdi:code-json", color: "#cbcb41" },
  jsonc: { icon: "mdi:code-json", color: "#cbcb41" },
  md: { icon: "mdi:language-markdown", color: "#083fa1" },
  mdx: { icon: "mdi:language-markdown", color: "#083fa1" },
  html: { icon: "mdi:language-html5", color: "#e34f26" },
  htm: { icon: "mdi:language-html5", color: "#e34f26" },
  css: { icon: "mdi:language-css3", color: "#1572b6" },
  scss: { icon: "mdi:sass", color: "#c69" },
  sass: { icon: "mdi:sass", color: "#c69" },
  less: { icon: "mdi:language-css3", color: "#1d365d" },
  rs: { icon: "mdi:language-rust", color: "#dea584" },
  py: { icon: "mdi:language-python", color: "#3776ab" },
  go: { icon: "mdi:language-go", color: "#00add8" },
  java: { icon: "mdi:language-java", color: "#ed8b00" },
  kt: { icon: "mdi:language-kotlin", color: "#7f52ff" },
  swift: { icon: "mdi:language-swift", color: "#f05138" },
  c: { icon: "mdi:language-c", color: "#a8b9cc" },
  h: { icon: "mdi:language-c", color: "#a8b9cc" },
  cpp: { icon: "mdi:language-cpp", color: "#00599c" },
  cc: { icon: "mdi:language-cpp", color: "#00599c" },
  hpp: { icon: "mdi:language-cpp", color: "#00599c" },
  cs: { icon: "mdi:language-csharp", color: "#239120" },
  php: { icon: "mdi:language-php", color: "#777bb4" },
  rb: { icon: "mdi:language-ruby", color: "#cc342d" },
  sh: { icon: "mdi:bash", color: "#4eaa25" },
  bash: { icon: "mdi:bash", color: "#4eaa25" },
  zsh: { icon: "mdi:bash", color: "#4eaa25" },
  yaml: { icon: "mdi:file-code-outline", color: "#cb171e" },
  yml: { icon: "mdi:file-code-outline", color: "#cb171e" },
  toml: { icon: "mdi:file-code-outline", color: "#9c4221" },
  xml: { icon: "mdi:file-code-outline", color: "#e37933" },
  svg: { icon: "mdi:svg", color: "#ffb13b" },
  png: { icon: "mdi:file-image-outline", color: "#a855f7" },
  jpg: { icon: "mdi:file-image-outline", color: "#a855f7" },
  jpeg: { icon: "mdi:file-image-outline", color: "#a855f7" },
  gif: { icon: "mdi:file-image-outline", color: "#a855f7" },
  webp: { icon: "mdi:file-image-outline", color: "#a855f7" },
  ico: { icon: "mdi:file-image-outline", color: "#a855f7" },
  pdf: { icon: "mdi:file-pdf-box", color: "#f40f02" },
  zip: { icon: "mdi:folder-zip-outline", color: "#eab308" },
  gz: { icon: "mdi:folder-zip-outline", color: "#eab308" },
  tar: { icon: "mdi:folder-zip-outline", color: "#eab308" },
  sql: { icon: "mdi:database-outline", color: "#336791" },
  sqlite: { icon: "mdi:database-outline", color: "#336791" },
  db: { icon: "mdi:database-outline", color: "#336791" },
  axdoc: { icon: "mdi:file-document-edit-outline", color: "#2563eb" },
  txt: { icon: "mdi:file-document-outline", color: "#6b7280" },
  log: { icon: "mdi:text-box-outline", color: "#9ca3af" },
  lock: { icon: "mdi:lock-outline", color: "#6b7280" },
};

const BY_LANGUAGE: Record<string, ExplorerFileIcon> = {
  typescript: { icon: "mdi:language-typescript", color: "#3178c6" },
  javascript: { icon: "mdi:language-javascript", color: "#f7df1e" },
  json: { icon: "mdi:code-json", color: "#cbcb41" },
  markdown: { icon: "mdi:language-markdown", color: "#083fa1" },
  html: { icon: "mdi:language-html5", color: "#e34f26" },
  css: { icon: "mdi:language-css3", color: "#1572b6" },
  scss: { icon: "mdi:sass", color: "#c69" },
  rust: { icon: "mdi:language-rust", color: "#dea584" },
  python: { icon: "mdi:language-python", color: "#3776ab" },
  go: { icon: "mdi:language-go", color: "#00add8" },
  java: { icon: "mdi:language-java", color: "#ed8b00" },
  appxdoc: { icon: "mdi:file-document-edit-outline", color: "#2563eb" },
  plaintext: { icon: "mdi:file-document-outline", color: "#6b7280" },
  dockerfile: { icon: "mdi:docker", color: "#2496ed" },
  shell: { icon: "mdi:bash", color: "#4eaa25" },
  yaml: { icon: "mdi:file-code-outline", color: "#cb171e" },
  xml: { icon: "mdi:file-code-outline", color: "#e37933" },
  sql: { icon: "mdi:database-outline", color: "#336791" },
};

/** 去掉 `.x` / `.x0` 加密后缀，得到原始文件名 */
function plainFileName(fileName: string): string {
  return fileName.replace(/\.(x0?)$/i, "");
}

function extensionOf(fileName: string): string | null {
  const base = plainFileName(fileName);
  const dot = base.lastIndexOf(".");
  if (dot <= 0 || dot === base.length - 1) {
    return null;
  }
  return base.slice(dot + 1).toLowerCase();
}

/** 按原始文件名 / 扩展名 / 语言解析图标（`.txt.x` → txt） */
export function resolveExplorerFileIcon(node: ExplorerTreeItem): ExplorerFileIcon {
  if (node.kind === "directory") {
    return { icon: "mdi:folder-outline", color: "#94a3b8" };
  }

  const plainName = plainFileName(node.title).toLowerCase();
  const byName = BY_FILE_NAME[plainName];
  if (byName) {
    return byName;
  }

  const ext = extensionOf(node.title);
  if (ext && BY_EXTENSION[ext]) {
    return BY_EXTENSION[ext];
  }

  if (node.language && BY_LANGUAGE[node.language]) {
    return BY_LANGUAGE[node.language];
  }

  return { icon: "mdi:file-document-outline", color: "#6b7280" };
}

export function resolveExplorerFolderIcon(expanded: boolean): ExplorerFileIcon {
  return {
    icon: expanded ? "mdi:folder-open-outline" : "mdi:folder-outline",
  };
}
