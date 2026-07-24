const MODIFIER_ONLY_KEYS = new Set(["Control", "Shift", "Alt", "Meta", "OS", "Fn"]);

/** 将 KeyboardEvent 转为 Tauri global-shortcut 字符串 */
export function captureShortcut(event: KeyboardEvent): string | null {
  // 忽略 enigo 等注入事件，避免粘贴模拟的 Cmd/Ctrl+V 被录入
  if (!event.isTrusted) return null;
  if (MODIFIER_ONLY_KEYS.has(event.key)) return null;

  const fnKey = resolveFunctionKey(event);
  if (fnKey) {
    return joinShortcut(event, fnKey);
  }

  const key = resolveShortcutKey(event);
  if (!key) return null;
  return joinShortcut(event, key);
}

function joinShortcut(event: KeyboardEvent, key: string): string {
  const parts: string[] = [];
  if (event.metaKey) {
    parts.push("Command");
  } else if (event.ctrlKey) {
    parts.push("CommandOrControl");
  }
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  parts.push(key);
  return parts.join("+");
}

/**
 * macOS WebView 上 F1 等常把 `key` 误报成普通字母（如 "a"），`code` 也可能不可靠；
 * `keyCode`（112–135 → F1–F24）相对稳定，优先用于功能键识别。
 */
function resolveFunctionKey(event: KeyboardEvent): string | null {
  const { key, code, keyCode } = event;

  if (/^F([1-9]|1[0-9]|2[0-4])$/i.test(code)) return code.toUpperCase();
  if (/^F([1-9]|1[0-9]|2[0-4])$/i.test(key)) return key.toUpperCase();

  if (keyCode >= 112 && keyCode <= 135) {
    return `F${keyCode - 111}`;
  }

  return null;
}

function resolveShortcutKey(event: KeyboardEvent): string | null {
  const { key, code } = event;

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
