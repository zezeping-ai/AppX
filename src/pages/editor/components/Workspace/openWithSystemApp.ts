import { Modal, message } from "ant-design-vue";
import { openPath } from "@tauri-apps/plugin-opener";
import { isAppxOwnedFile } from "@/modules/editor/ownedFile";

function fileNameOf(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

/** 用系统默认应用打开本地文件 */
export async function openWithSystemApp(path: string): Promise<boolean> {
  if (isAppxOwnedFile(path)) {
    message.info("该文件请在 AppX 编辑器中打开");
    return false;
  }
  try {
    await openPath(path);
    return true;
  } catch (error) {
    message.error(`无法用系统应用打开：${String(error)}`);
    return false;
  }
}

/** 不可编辑文件：确认后交系统应用打开 */
export async function confirmOpenWithSystemApp(path: string): Promise<boolean> {
  if (isAppxOwnedFile(path)) {
    message.info("该文件请在 AppX 编辑器中打开");
    return false;
  }
  const fileName = fileNameOf(path);
  const confirmed = await new Promise<boolean>((resolve) => {
    Modal.confirm({
      title: "无法在编辑器中打开",
      content: `「${fileName}」不是可编辑的文本文件，是否使用系统应用打开？`,
      okText: "用系统应用打开",
      cancelText: "取消",
      onOk: () => resolve(true),
      onCancel: () => resolve(false),
    });
  });
  if (!confirmed) {
    return false;
  }
  return openWithSystemApp(path);
}
