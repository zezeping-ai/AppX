import { isAppSessionLocked } from "@/modules/appLock";
import { syncClipboardAssistantRuntime } from "@/modules/clipboardAssistant/client";

/** 解锁后同步剪切助手运行时（监听、快捷键） */
export async function bootstrapClipboardAssistantAfterUnlock(): Promise<void> {
  if (await isAppSessionLocked()) return;
  try {
    await syncClipboardAssistantRuntime();
  } catch {
    // 非 Tauri 或读取失败时忽略
  }
}
