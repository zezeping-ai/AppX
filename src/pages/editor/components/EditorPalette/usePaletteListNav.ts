import { clamp } from "lodash-es";
import { nextTick, type Ref } from "vue";

export type PaletteListNavOptions = {
  listRef: Ref<HTMLElement | null>;
  /** 当前激活下标 */
  activeIndex: Ref<number>;
  /** 列表长度 */
  length: () => number;
  onEnter: () => void;
  /** 滚动定位属性；默认 data-index */
  itemAttr?: string;
  /** 用 key 定位时，根据 index 取 data-key */
  keyAt?: (index: number) => string | null;
};

/** ↑↓ / Enter 列表导航（Esc 由 usePaletteEscape 负责） */
export function usePaletteListNav(options: PaletteListNavOptions) {
  const itemAttr = options.itemAttr ?? "data-index";

  function moveActive(delta: number) {
    const len = options.length();
    if (!len) {
      return;
    }
    const next = (options.activeIndex.value + delta + len) % len;
    options.activeIndex.value = clamp(next, 0, len - 1);
    void nextTick(() => scrollActiveIntoView());
  }

  function scrollActiveIntoView() {
    const list = options.listRef.value;
    if (!list) {
      return;
    }
    let selector: string;
    if (options.keyAt) {
      const key = options.keyAt(options.activeIndex.value);
      if (!key) {
        return;
      }
      selector = `[data-key="${CSS.escape(key)}"]`;
    } else {
      selector = `[${itemAttr}="${options.activeIndex.value}"]`;
    }
    list.querySelector<HTMLElement>(selector)?.scrollIntoView({ block: "nearest" });
  }

  function onPanelKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveActive(1);
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      moveActive(-1);
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      options.onEnter();
    }
  }

  return {
    moveActive,
    scrollActiveIntoView,
    onPanelKeydown,
  };
}
