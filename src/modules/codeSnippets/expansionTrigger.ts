import { ref } from "vue";
import { formatShortcutLabel } from "@/shared/shortcut";

export const DEFAULT_INLINE_EXPANSION_TRIGGER = "F12";

/** 运行时缓存的缩写展开触发键 */
export const inlineExpansionTrigger = ref(DEFAULT_INLINE_EXPANSION_TRIGGER);

export function setInlineExpansionTrigger(shortcut: string) {
  inlineExpansionTrigger.value = shortcut.trim() || DEFAULT_INLINE_EXPANSION_TRIGGER;
}

export function inlineExpansionTriggerLabel(): string {
  return formatShortcutLabel(inlineExpansionTrigger.value);
}
