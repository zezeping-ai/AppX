/** 按应用覆盖的内容变体（数据层 canonical；content 阶段由调用方区分明文/密文） */

export type CodeSnippetAppVariant = {
  appId: string;
  appName: string;
  content: string;
};

export function emptyAppVariant(): CodeSnippetAppVariant {
  return { appId: "", appName: "", content: "" };
}

/** 提交前校验：目标 App 必填、内容非空、appId 不重复 */
export function validateAppVariants(variants: CodeSnippetAppVariant[]): string | null {
  for (const [i, v] of variants.entries()) {
    if (!v.appId.trim()) {
      return `第 ${i + 1} 条应用覆盖未选择目标应用`;
    }
    if (!v.content.trim()) {
      return `「${v.appName || v.appId}」的覆盖内容不能为空`;
    }
  }
  const ids = variants.map((v) => v.appId);
  if (new Set(ids).size !== ids.length) {
    return "同一应用只能绑定一条覆盖";
  }
  return null;
}
