import { bootstrapAfterUnlock as bootstrapCodeSnippetsAfterUnlock } from "@/modules/codeSnippets/bootstrap";
import { bootstrapClipboardAssistantAfterUnlock } from "@/modules/clipboardAssistant/bootstrap";

/** 解锁后启动各功能域运行时 */
export async function bootstrapAfterUnlock(): Promise<void> {
  await bootstrapCodeSnippetsAfterUnlock();
  await bootstrapClipboardAssistantAfterUnlock();
}
