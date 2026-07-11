import { isAppSessionLocked } from "@/modules/appLock";
import { getCodeSnippetSettings } from "@/modules/codeSnippets/client";
import { setInlineExpansionTrigger } from "@/modules/codeSnippets/expansionTrigger";
import { syncAllSnippetsToRuntime } from "@/modules/codeSnippets/syncRuntime";

/** 解锁后启动任务：同步代码段到 Rust 运行时 */
export async function bootstrapAfterUnlock(): Promise<void> {
  if (await isAppSessionLocked()) return;
  try {
    const settings = await getCodeSnippetSettings();
    setInlineExpansionTrigger(settings.inlineExpansionTrigger);
  } catch {
    // 非 Tauri 或读取失败时保留默认值
  }
  await syncAllSnippetsToRuntime();
}
