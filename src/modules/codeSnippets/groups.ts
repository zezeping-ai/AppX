export const CODE_SNIPPET_GROUPS = [
  { value: "general", label: "通用", icon: "mdi:star-outline" },
  { value: "code", label: "代码", icon: "mdi:code-tags" },
  { value: "password", label: "密码", icon: "mdi:key-outline" },
  { value: "email", label: "邮件", icon: "mdi:email-outline" },
  { value: "address", label: "地址", icon: "mdi:map-marker-outline" },
  { value: "markdown", label: "文档", icon: "mdi:markdown-outline" },
  { value: "sql", label: "SQL", icon: "mdi:database-outline" },
  { value: "shell", label: "命令行", icon: "mdi:console" },
  { value: "git", label: "Git", icon: "mdi:git" },
  { value: "api", label: "API", icon: "mdi:api" },
  { value: "comment", label: "注释", icon: "mdi:comment-text-outline" },
  { value: "template", label: "模板", icon: "mdi:file-document-outline" },
] as const;

export type CodeSnippetGroup = (typeof CODE_SNIPPET_GROUPS)[number]["value"];

export const DEFAULT_CODE_SNIPPET_GROUP: CodeSnippetGroup = "general";

export function labelOfSnippetGroup(group: string): string {
  return CODE_SNIPPET_GROUPS.find((item) => item.value === group)?.label ?? group;
}

export function isPasswordSnippetGroup(group: string): boolean {
  return group === "password";
}
