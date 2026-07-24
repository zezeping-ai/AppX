import { common, createLowlight } from "lowlight";
import * as prettier from "prettier/standalone";
import * as prettierPluginBabel from "prettier/plugins/babel";
import * as prettierPluginEstree from "prettier/plugins/estree";
import * as prettierPluginHtml from "prettier/plugins/html";
import * as prettierPluginMarkdown from "prettier/plugins/markdown";
import * as prettierPluginPostcss from "prettier/plugins/postcss";
import * as prettierPluginTypescript from "prettier/plugins/typescript";
import * as prettierPluginYaml from "prettier/plugins/yaml";

/** TipTap CodeBlockLowlight 共用实例（common 语言集） */
export const richDocLowlight = createLowlight(common);

export type CodeLanguageOption = {
  value: string;
  label: string;
};

/** 代码块可选语言（与 lowlight common / prettier 能力对齐） */
export const CODE_LANGUAGE_OPTIONS: CodeLanguageOption[] = [
  { value: "", label: "纯文本" },
  { value: "javascript", label: "JavaScript" },
  { value: "typescript", label: "TypeScript" },
  { value: "json", label: "JSON" },
  { value: "html", label: "HTML" },
  { value: "css", label: "CSS" },
  { value: "scss", label: "SCSS" },
  { value: "markdown", label: "Markdown" },
  { value: "yaml", label: "YAML" },
  { value: "xml", label: "XML" },
  { value: "sql", label: "SQL" },
  { value: "bash", label: "Bash" },
  { value: "python", label: "Python" },
  { value: "go", label: "Go" },
  { value: "rust", label: "Rust" },
  { value: "java", label: "Java" },
  { value: "c", label: "C" },
  { value: "cpp", label: "C++" },
];

type PrettierParser =
  | "babel"
  | "typescript"
  | "json"
  | "html"
  | "css"
  | "scss"
  | "markdown"
  | "yaml";

const LANGUAGE_TO_PARSER: Record<string, PrettierParser> = {
  javascript: "babel",
  js: "babel",
  jsx: "babel",
  typescript: "typescript",
  ts: "typescript",
  tsx: "typescript",
  json: "json",
  jsonc: "json",
  html: "html",
  vue: "html",
  css: "css",
  scss: "scss",
  less: "css",
  markdown: "markdown",
  md: "markdown",
  yaml: "yaml",
  yml: "yaml",
};

const PRETTIER_PLUGINS = [
  prettierPluginBabel,
  prettierPluginEstree,
  prettierPluginHtml,
  prettierPluginMarkdown,
  prettierPluginPostcss,
  prettierPluginTypescript,
  prettierPluginYaml,
];

export function canFormatCodeLanguage(language: string | null | undefined): boolean {
  if (!language) {
    return false;
  }
  return Boolean(LANGUAGE_TO_PARSER[language.toLowerCase()]);
}

/** 按语言格式化代码；不支持的语言返回错误 */
export async function formatCodeByLanguage(
  source: string,
  language: string | null | undefined,
): Promise<{ ok: true; code: string } | { ok: false; error: string }> {
  const key = (language ?? "").toLowerCase();
  const parser = LANGUAGE_TO_PARSER[key];
  if (!parser) {
    return { ok: false, error: "当前语言暂不支持一键格式化" };
  }

  try {
    const code = await prettier.format(source, {
      parser,
      plugins: PRETTIER_PLUGINS,
      printWidth: 100,
      tabWidth: 2,
      semi: true,
      singleQuote: false,
      trailingComma: "all",
    });
    return { ok: true, code: code.replace(/\n$/, "") };
  } catch (error) {
    return {
      ok: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}
