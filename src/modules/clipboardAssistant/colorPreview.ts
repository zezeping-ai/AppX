import type { ClipboardItemSummary } from "@/modules/clipboardAssistant/types";

/** 解析剪贴板颜色条目可用的 CSS 色值（从 preview 读取，不用 accentColor） */
export function resolveClipboardColor(item: ClipboardItemSummary): string | undefined {
  if (item.contentType !== "color") return undefined;
  return normalizeCssColor(item.preview);
}

/** 归一化 #rgb / #rrggbb / #rrggbbaa */
export function normalizeCssColor(raw: string): string | undefined {
  const value = raw.trim();
  if (!value.startsWith("#")) return undefined;
  const hex = value.slice(1);
  if (hex.length === 3 && isHex(hex)) {
    return `#${hex
      .split("")
      .map((c) => c + c)
      .join("")
      .toLowerCase()}`;
  }
  if ((hex.length === 6 || hex.length === 8) && isHex(hex)) {
    return `#${hex.toLowerCase()}`;
  }
  return undefined;
}

/** 根据背景色亮度选择可读的前景色 */
export function contrastTextOnFill(fill: string): string {
  const rgb = hexToRgb(fill);
  if (!rgb) return "var(--app-fg)";
  const luminance = (0.299 * rgb.r + 0.587 * rgb.g + 0.114 * rgb.b) / 255;
  return luminance > 0.62 ? "#111827" : "#f9fafb";
}

function isHex(value: string): boolean {
  return /^[0-9a-fA-F]+$/.test(value);
}

function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
  const normalized = normalizeCssColor(hex);
  if (!normalized) return null;
  const body = normalized.slice(1);
  const rgbPart = body.length === 8 ? body.slice(0, 6) : body;
  const r = Number.parseInt(rgbPart.slice(0, 2), 16);
  const g = Number.parseInt(rgbPart.slice(2, 4), 16);
  const b = Number.parseInt(rgbPart.slice(4, 6), 16);
  if ([r, g, b].some((n) => Number.isNaN(n))) return null;
  return { r, g, b };
}
