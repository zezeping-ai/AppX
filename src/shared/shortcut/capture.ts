const MODIFIER_ONLY_KEYS = new Set(["Control", "Shift", "Alt", "Meta", "OS"]);

/** 将 KeyboardEvent 转为 Tauri global-shortcut 字符串 */
export function captureShortcut(event: KeyboardEvent): string | null {
  if (MODIFIER_ONLY_KEYS.has(event.key)) return null;

  const parts: string[] = [];
  if (event.metaKey) {
    parts.push("Command");
  } else if (event.ctrlKey) {
    parts.push("CommandOrControl");
  }
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");

  const key = resolveShortcutKey(event);
  if (!key) return null;

  parts.push(key);
  return parts.join("+");
}

function resolveShortcutKey(event: KeyboardEvent): string | null {
  const { key, code } = event;

  if (/^F\d{1,2}$/i.test(key)) return key.toUpperCase();
  if (key === " ") return "Space";
  if (key === "Enter") return "Enter";
  if (key === "Tab") return "Tab";
  if (key === "Backspace") return "Backspace";
  if (key === "Delete") return "Delete";
  if (key === "Escape") return "Escape";
  if (key.startsWith("Arrow")) return key.slice(5);

  if (key.length === 1 && /[a-zA-Z0-9]/.test(key)) return `Key${key.toUpperCase()}`;
  if (/^Key[A-Z0-9]$/.test(code)) return code;
  if (/^Digit[0-9]$/.test(code)) return `Key${code.slice(-1)}`;

  return null;
}

/** Tauri 快捷键转展示文案 */
export function formatShortcutLabel(shortcut: string): string {
  if (!shortcut.trim()) return "";

  return shortcut
    .split("+")
    .map((part) => {
      if (part === "CommandOrControl") return "Mod";
      if (part === "Command") return "Cmd";
      if (part.startsWith("Key") && part.length === 4) return part.slice(3);
      if (part.startsWith("Digit") && part.length === 6) return part.slice(5);
      return part;
    })
    .join(" + ");
}
