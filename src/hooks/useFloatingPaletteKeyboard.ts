import { computed, type MaybeRefOrGetter, type Ref } from "vue";
import { focusPaletteSearch, type PaletteSearchInputRef } from "@/hooks/focusPaletteSearch";
import { usePaletteEscape } from "@/hooks/usePaletteEscape";
import { usePaletteListNav } from "@/hooks/usePaletteListNav";

export type FloatingPaletteShortcutActions<T> = {
  /** 搜索为空时启用 1–9 选中（走 onApply） */
  numberKeys?: boolean;
  onPin?: (item: T) => void;
  onDelete?: (item: T) => void;
};

export type FloatingPaletteKeyboardOptions<T> = {
  activeIndex: Ref<number>;
  keyword: Ref<string>;
  list: () => T[];
  onApply: (item: T) => void;
  onEscape: () => void;
  onToggleActionMode: () => void;
  horizontal?: MaybeRefOrGetter<boolean>;
  shortcutActions?: FloatingPaletteShortcutActions<T>;
  listRef?: Ref<HTMLElement | null>;
  scrollToActive?: () => void;
};

/** 独立浮层面板：在列表导航之上叠加 Tab / Esc / 空搜单键快捷 */
export function useFloatingPaletteKeyboard<T>(options: FloatingPaletteKeyboardOptions<T>) {
  const shortcutKeysReady = computed(() => !options.keyword.value.trim());

  usePaletteEscape(() => true, options.onEscape);

  const { onPanelKeydown } = usePaletteListNav({
    activeIndex: options.activeIndex,
    length: () => options.list().length,
    horizontal: options.horizontal,
    listRef: options.listRef,
    scrollToActive: options.scrollToActive,
    onEnter: () => {
      const item = options.list()[options.activeIndex.value];
      if (item) {
        options.onApply(item);
      }
    },
  });

  function onSearchKeydown(event: KeyboardEvent) {
    if (event.key === "Tab") {
      event.preventDefault();
      options.onToggleActionMode();
      return;
    }

    const list = options.list();
    const shortcuts = options.shortcutActions;

    if (
      shortcutKeysReady.value &&
      shortcuts?.numberKeys &&
      /^[1-9]$/.test(event.key) &&
      list.length
    ) {
      const index = Number(event.key) - 1;
      const item = list[index];
      if (item) {
        event.preventDefault();
        options.onApply(item);
      }
      return;
    }

    if (onPanelKeydown(event)) {
      return;
    }

    if (!shortcutKeysReady.value) {
      return;
    }

    const active = list[options.activeIndex.value];
    if (!active) {
      return;
    }

    if (event.key.toLowerCase() === "p" && shortcuts?.onPin) {
      event.preventDefault();
      shortcuts.onPin(active);
      return;
    }

    if (event.key === "Delete" && shortcuts?.onDelete) {
      event.preventDefault();
      shortcuts.onDelete(active);
    }
  }

  return {
    onSearchKeydown,
    shortcutKeysReady,
    focusSearch: (inputRef: PaletteSearchInputRef | null | undefined, enabled = true) =>
      focusPaletteSearch(inputRef, enabled),
  };
}
