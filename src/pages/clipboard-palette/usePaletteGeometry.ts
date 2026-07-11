import { useDebounceFn } from "@vueuse/core";
import { onMounted, onUnmounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { savePaletteGeometry } from "@/modules/clipboardAssistant";

/** 拖动/缩放浮层时 debounce 持久化位置（需偏好开启 rememberWindowPosition）。 */
export function usePaletteGeometry(rememberPosition: () => boolean) {
  const debouncedSave = useDebounceFn(async () => {
    if (!rememberPosition()) return;
    try {
      const win = getCurrentWindow();
      const [pos, size, scale] = await Promise.all([
        win.outerPosition(),
        win.outerSize(),
        win.scaleFactor(),
      ]);
      await savePaletteGeometry({
        x: pos.x / scale,
        y: pos.y / scale,
        width: size.width / scale,
        height: size.height / scale,
      });
    } catch {
      // 非 Tauri 环境或窗口不可用时忽略
    }
  }, 400);

  let unlistenMove: (() => void) | undefined;
  let unlistenResize: (() => void) | undefined;

  onMounted(async () => {
    try {
      const win = getCurrentWindow();
      unlistenMove = await win.onMoved(() => void debouncedSave());
      unlistenResize = await win.onResized(() => void debouncedSave());
    } catch {
      // dev 浏览器预览无窗口 API
    }
  });

  onUnmounted(() => {
    unlistenMove?.();
    unlistenResize?.();
  });
}
