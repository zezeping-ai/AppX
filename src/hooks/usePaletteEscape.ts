import { useEventListener } from "@vueuse/core";
import { toValue, type MaybeRefOrGetter } from "vue";

/** 面板打开时在 capture 阶段监听 Esc，避免 Monaco 等抢焦点后关不掉 */
export function usePaletteEscape(
  open: MaybeRefOrGetter<boolean>,
  onEscape: () => void,
) {
  useEventListener(
    window,
    "keydown",
    (event: KeyboardEvent) => {
      if (!toValue(open)) {
        return;
      }
      if (event.key !== "Escape") {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      onEscape();
    },
    { capture: true },
  );
}
