import { CodeSnippetRecord } from "@/models";
import {
  formatShortcutLabel,
  normalizeGlobalShortcut,
} from "@/shared/shortcut";
import { inlineExpansionTrigger } from "@/modules/codeSnippets/expansionTrigger";

/** 查找占用同一全局快捷键的代码段（规范化后比较） */
export async function findSnippetByShortcut(
  shortcut: string,
  excludeId?: number,
): Promise<CodeSnippetRecord | null> {
  const normalized = normalizeGlobalShortcut(shortcut);
  if (!normalized) return null;

  const { data } = await CodeSnippetRecord.query({
    where: {},
    pagination: { page: 1, pageSize: 2000 },
  });

  return (
    data.find((row) => {
      if (excludeId != null && row.id === excludeId) return false;
      const existing = normalizeGlobalShortcut(String(row.shortcut ?? ""));
      return existing === normalized;
    }) ?? null
  );
}

/** 校验快捷键未被其它代码段 / 缩写触发键占用 */
export async function assertSnippetShortcutAvailable(
  shortcut: string | null | undefined,
  excludeId?: number,
): Promise<string | null> {
  const normalized = normalizeGlobalShortcut(String(shortcut ?? ""));
  if (!normalized) return null;

  const trigger = normalizeGlobalShortcut(inlineExpansionTrigger.value);
  if (trigger && trigger === normalized) {
    return `快捷键「${formatShortcutLabel(normalized)}」已用作缩写展开触发键`;
  }

  const conflict = await findSnippetByShortcut(normalized, excludeId);
  if (conflict) {
    return `快捷键「${formatShortcutLabel(normalized)}」已被「${conflict.name}」占用`;
  }

  return null;
}
