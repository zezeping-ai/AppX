import { onBeforeUnmount, ref } from "vue";
import { message } from "ant-design-vue";
import { getFrontmostApp } from "@/modules/codeSnippets";
import { setGlobalShortcutsPaused } from "@/modules/globalShortcut";
import { getErrorMessage } from "@/shared/error";

const PICK_TIMEOUT_MS = 20_000;
const PICK_INTERVAL_MS = 400;

export type PickedTargetApp = {
  appId: string;
  appName: string;
};

/**
 * 「选择目标应用」：暂停全局快捷键，轮询前台非本进程 App。
 * `isAppTaken` 用于排除其它覆盖已占用的 appId。
 */
export function usePickTargetApp(options: {
  isAppTaken?: (appId: string, pickingIndex: number) => boolean;
}) {
  const pickingIndex = ref<number | null>(null);
  let timer: ReturnType<typeof setInterval> | null = null;
  let deadline = 0;

  function stopPicking() {
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
    pickingIndex.value = null;
    void setGlobalShortcutsPaused(false);
  }

  async function pollOnce(
    index: number,
    onPicked: (app: PickedTargetApp) => void,
  ) {
    try {
      const front = await getFrontmostApp();
      if (front.isOwn || !front.appId) return;

      const appId = front.appId;
      if (options.isAppTaken?.(appId, index)) {
        message.warning(`「${front.appName || appId}」已在其它覆盖中使用`);
        stopPicking();
        return;
      }

      onPicked({
        appId,
        appName: front.appName?.trim() || appId,
      });
      message.success(`已绑定「${front.appName?.trim() || appId}」`);
      stopPicking();
    } catch (error) {
      message.error(getErrorMessage(error, "读取前台应用失败"));
      stopPicking();
    }
  }

  async function startPick(index: number, onPicked: (app: PickedTargetApp) => void) {
    stopPicking();
    pickingIndex.value = index;
    await setGlobalShortcutsPaused(true);
    message.info("请切换到目标软件，检测到后自动填入（约 20 秒）");
    deadline = Date.now() + PICK_TIMEOUT_MS;
    timer = setInterval(() => {
      if (Date.now() > deadline) {
        message.warning("选择超时，请重试");
        stopPicking();
        return;
      }
      void pollOnce(index, onPicked);
    }, PICK_INTERVAL_MS);
  }

  onBeforeUnmount(stopPicking);

  return { pickingIndex, startPick, stopPicking };
}
