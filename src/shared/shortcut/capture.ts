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

  // 优先用 code：macOS 上 F 键等可能把 key 报成普通字符（如 F1 → "a"）
  if (/^F\d{1,2}$/i.test(code)) return code.toUpperCase();
  if (/^F\d{1,2}$/i.test(key)) return key.toUpperCase();

  if (code === "Space" || key === " ") return "Space";
  if (code === "Enter" || code === "NumpadEnter" || key === "Enter") return "Enter";
  if (code === "Tab" || key === "Tab") return "Tab";
  if (code === "Backspace" || key === "Backspace") return "Backspace";
  if (code === "Delete" || key === "Delete") return "Delete";
  if (code === "Escape" || key === "Escape") return "Escape";
  if (code.startsWith("Arrow")) return code.slice(5);
  if (key.startsWith("Arrow")) return key.slice(5);

  if (/^Digit[0-9]$/.test(code)) return `Key${code.slice(-1)}`;
  if (/^Numpad[0-9]$/.test(code)) return `Key${code.slice(-1)}`;
  if (/^Key[A-Z0-9]$/.test(code)) return code;

  if (key.length === 1 && /[a-zA-Z0-9]/.test(key)) return `Key${key.toUpperCase()}`;

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
