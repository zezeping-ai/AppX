import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { onMounted, onUnmounted, ref, type Ref } from "vue";
import { getSettings, saveSettings } from "@/modules/imageOptim/client";
import {
  DEFAULT_IMAGE_OPTIM_SETTINGS,
  type ImageOptimSettings,
} from "@/modules/imageOptim/types";

const SETTINGS_CHANGED_EVENT = "appx:image-optim-settings-changed";

/** 同窗口内共享快照；跨窗口靠 Tauri 事件同步 */
const sharedSettings = ref<ImageOptimSettings>({ ...DEFAULT_IMAGE_OPTIM_SETTINGS });
const loaded = ref(false);

export function useImageOptimSettings(): {
  settings: Ref<ImageOptimSettings>;
  loaded: Ref<boolean>;
  refresh: () => Promise<void>;
  persist: (next: ImageOptimSettings) => Promise<ImageOptimSettings>;
  patch: (partial: Partial<ImageOptimSettings>) => Promise<ImageOptimSettings>;
} {
  let unlisten: UnlistenFn | null = null;

  async function refresh() {
    sharedSettings.value = await getSettings();
    loaded.value = true;
  }

  async function persist(next: ImageOptimSettings) {
    const saved = await saveSettings(next);
    sharedSettings.value = saved;
    loaded.value = true;
    return saved;
  }

  async function patch(partial: Partial<ImageOptimSettings>) {
    return persist({ ...sharedSettings.value, ...partial });
  }

  onMounted(() => {
    void refresh().catch(() => {
      /* 由页面/偏好自行提示 */
    });
    void listen<ImageOptimSettings>(SETTINGS_CHANGED_EVENT, (event) => {
      if (event.payload) {
        sharedSettings.value = event.payload;
        loaded.value = true;
      }
    }).then((fn) => {
      unlisten = fn;
    });
  });

  onUnmounted(() => {
    unlisten?.();
    unlisten = null;
  });

  return {
    settings: sharedSettings,
    loaded,
    refresh,
    persist,
    patch,
  };
}
