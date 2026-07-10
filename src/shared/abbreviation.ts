/** 与 Rust 端一致的缩写字符集；`:` `;` 为触发符不可用于缩写 */
export const ABBREVIATION_PATTERN = /^[a-z0-9_\-!#%^&*()+.=,?/<>]+$/;

export const ABBREVIATION_MAX_LEN = 32;

const ABBREVIATION_INPUT_FILTER = /[^a-z0-9_\-!#%^&*()+.=,?/<>]/g;

export function normalizeAbbreviationInput(value: string): string {
  return value
    .toLowerCase()
    .replace(ABBREVIATION_INPUT_FILTER, "")
    .slice(0, ABBREVIATION_MAX_LEN);
}

export function validateAbbreviation(value: string): string | null {
  const normalized = value.trim().toLowerCase();
  if (!normalized) return "缩写不能为空";
  if (normalized.includes(":") || normalized.includes(";")) {
    return "缩写不能包含 : 或 ;（二者为触发符）";
  }
  if (!ABBREVIATION_PATTERN.test(normalized)) {
    return "缩写含不支持的字符";
  }
  if (normalized.length > ABBREVIATION_MAX_LEN) {
    return `缩写最多 ${ABBREVIATION_MAX_LEN} 个字符`;
  }
  return null;
}

export function formatAbbreviationTrigger(abbreviation: string): string {
  const abbr = abbreviation.trim().toLowerCase();
  return abbr ? `:${abbr};` : ":缩写;";
}
