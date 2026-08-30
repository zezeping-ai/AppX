import { clamp } from "lodash-es";
import { nextTick, toValue, type MaybeRefOrGetter, type Ref } from "vue";

export type PaletteListNavOptions = {
  activeIndex: Ref<number>;
  length: () => number;
  onEnter: () => void;
  /** 横向布局时左右键导航；默认仅上下键 */
  horizontal?: MaybeRefOrGetter<boolean>;
  /** DOM 列表：按 data-index / data-key 滚入视口 */
  listRef?: Ref<HTMLElement | null>;
  /** 虚拟列表等：自定义滚到激活项（与 listRef 二选一或同时，优先本回调） */
  scrollToActive?: () => void;
  itemAttr?: string;
  keyAt?: (index: number) => string | null;
};

/** 循环移动激活下标并 clamp 到合法范围 */
export function moveActiveIndex(activeIndex: Ref<number>, length: number, delta: number) {
  if (length <= 0) {
    return;
  }
  const next = (activeIndex.value + delta + length) % length;
  activeIndex.value = clamp(next, 0, length - 1);
}

/** ↑↓ / ←→ / Enter 列表导航（Esc 由 usePaletteEscape 负责） */
export function usePaletteListNav(options: PaletteListNavOptions) {
  const itemAttr = options.itemAttr ?? "data-index";

  function scrollActiveIntoView() {
    if (options.scrollToActive) {
      options.scrollToActive();
      return;
    }
    const list = options.listRef?.value;
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
    list.querySelector<HTMLElement>(selector)?.scrollIntoView({
      block: "nearest",
      inline: "nearest",
    });
  }

  function moveActive(delta: number) {
    moveActiveIndex(options.activeIndex, options.length(), delta);
    void nextTick(scrollActiveIntoView);
  }

  function onPanelKeydown(event: KeyboardEvent): boolean {
    const len = options.length();
    const horizontal = toValue(options.horizontal ?? false);

    if (horizontal) {
      if (event.key === "ArrowRight" && len) {
        event.preventDefault();
        moveActive(1);
        return true;
      }
      if (event.key === "ArrowLeft" && len) {
        event.preventDefault();
        moveActive(-1);
        return true;
      }
    }

    if (event.key === "ArrowDown" && len) {
      event.preventDefault();
      moveActive(1);
      return true;
    }
    if (event.key === "ArrowUp" && len) {
      event.preventDefault();
      moveActive(-1);
      return true;
    }
    if (event.key === "Enter" && len) {
      event.preventDefault();
      options.onEnter();
      return true;
    }
    return false;
  }

  return {
    moveActive,
    scrollActiveIntoView,
    onPanelKeydown,
  };
}
