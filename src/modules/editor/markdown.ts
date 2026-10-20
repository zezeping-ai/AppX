/** Markdown（.md / .markdown，含 .x / .x0）约定 */

export const MARKDOWN_LANGUAGE = "markdown";
export const DEFAULT_MARKDOWN_FILE_NAME = "untitled.md.x";

const MARKDOWN_NAME_SUFFIXES = [
  ".md",
  ".markdown",
  ".md.x",
  ".markdown.x",
  ".md.x0",
  ".markdown.x0",
] as const;

export function isMarkdownLanguage(language: string | undefined | null): boolean {
  return language === MARKDOWN_LANGUAGE;
}

/** 按文件名判断是否为 markdown（含加密后缀） */
export function isMarkdownFileName(fileName: string): boolean {
  const lower = fileName.trim().toLowerCase();
  return MARKDOWN_NAME_SUFFIXES.some((suffix) => lower.endsWith(suffix));
}
