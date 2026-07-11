import { formatShortcutLabel } from "@/shared/shortcut";
import { DEFAULT_INLINE_EXPANSION_TRIGGER } from "@/modules/codeSnippets/expansionTrigger";

/** 与 Rust 端一致的缩写字符集；`:` 为前缀触发符不可用于缩写 */
export const ABBREVIATION_PATTERN = /^[a-z0-9_\-!#%^&*()+.=,?/<>;]+$/;

export const ABBREVIATION_MAX_LEN = 32;

export { DEFAULT_INLINE_EXPANSION_TRIGGER };

const ABBREVIATION_INPUT_FILTER = /[^a-z0-9_\-!#%^&*()+.=,?/<>;]/g;

export function normalizeAbbreviationInput(value: string): string {
  return value
    .toLowerCase()
    .replace(ABBREVIATION_INPUT_FILTER, "")
    .slice(0, ABBREVIATION_MAX_LEN);
}

export function validateAbbreviation(value: string): string | null {
  const normalized = value.trim().toLowerCase();
  if (!normalized) return "缩写不能为空";
  if (normalized.includes(":")) {
    return "缩写不能包含 :（其为前缀触发符）";
  }
  if (!ABBREVIATION_PATTERN.test(normalized)) {
    return "缩写含不支持的字符";
  }
  if (normalized.length > ABBREVIATION_MAX_LEN) {
    return `缩写最多 ${ABBREVIATION_MAX_LEN} 个字符`;
  }
  return null;
}

export function formatAbbreviationTrigger(
  abbreviation: string,
  triggerKey = DEFAULT_INLINE_EXPANSION_TRIGGER,
): string {
  const abbr = abbreviation.trim().toLowerCase();
  const triggerLabel = formatShortcutLabel(triggerKey) || triggerKey;
  return abbr ? `:${abbr} + ${triggerLabel}` : `:缩写 + ${triggerLabel}`;
}
