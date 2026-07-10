/** 将 `Mod+Shift+H` 规范为 Tauri global-shortcut 格式 */
export function normalizeGlobalShortcut(raw: string): string | null {
  const trimmed = raw.trim();
  if (!trimmed) return null;

  return trimmed
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean)
    .map((part) => {
      const upper = part.toUpperCase();
      if (upper === "MOD" || upper === "CTRL" || upper === "CONTROL") return "CommandOrControl";
      if (upper === "CMD" || upper === "COMMAND") return "Command";
      if (upper === "ALT" || upper === "OPTION") return "Alt";
      if (upper === "SHIFT") return "Shift";
      if (/^F\d{1,2}$/i.test(part)) return part.toUpperCase();
      if (part.length === 1) return `Key${part.toUpperCase()}`;
      if (/^KEY[A-Z0-9]$/i.test(part)) return `Key${part.slice(-1).toUpperCase()}`;
      return part;
    })
    .join("+");
}
