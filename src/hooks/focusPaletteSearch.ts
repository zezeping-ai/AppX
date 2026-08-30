import { nextTick } from "vue";

export type PaletteSearchInputRef = {
  focus?: () => void;
  input?: HTMLInputElement;
};

/** 窗口 show/focus 可能晚于首帧，补一次确保搜索框获得焦点 */
export function focusPaletteSearch(
  inputRef: PaletteSearchInputRef | null | undefined,
  enabled = true,
) {
  if (!enabled) {
    return;
  }
  const run = () => {
    inputRef?.focus?.();
    inputRef?.input?.focus?.();
  };
  void nextTick(() => {
    requestAnimationFrame(() => {
      run();
      setTimeout(run, 50);
    });
  });
}
