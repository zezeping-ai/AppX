import type { FileInspect, OpenedEditorFile } from "@/modules/editor/types";

export const ENCRYPTED_FILE_NAME_RE = /\.[^./]+\.x0?$/i;

export function isEncryptedFileName(fileName: string): boolean {
  return ENCRYPTED_FILE_NAME_RE.test(fileName);
}

export function encryptionFileIcon(
  file: Pick<OpenedEditorFile, "encrypted" | "customEncrypted">,
): string {
  if (file.customEncrypted) return "mdi:file-key-outline";
  if (file.encrypted) return "mdi:file-lock-outline";
  return "mdi:file-document-outline";
}

export function fileStatusMark(file: Pick<OpenedEditorFile, "encrypted" | "customEncrypted">): string {
  if (file.customEncrypted) return " [独立加密]";
  if (file.encrypted) return " [加密]";
  return "";
}

export function saveSuccessLabel(file: Pick<OpenedEditorFile, "encrypted" | "customEncrypted">): string {
  if (file.customEncrypted) return "已独立加密保存";
  if (file.encrypted) return "已加密保存";
  return "已保存";
}

export function unlockPromptFor(inspect: Pick<FileInspect, "customEncrypted">) {
  if (inspect.customEncrypted) {
    return {
      title: "输入独立口令",
      content: "该文件使用独立口令加密 (.x0)",
    };
  }
  return {
    title: "解密失败，请输入口令",
    content: "默认口令无法解密。验证成功后将转为独立口令文件 (.x0)",
  };
}

export function convertUnlockPromptFor(
  inspect: Pick<FileInspect, "customEncrypted">,
  action: "plain" | "default",
) {
  if (action === "plain") {
    return {
      title: "输入独立口令",
      content: inspect.customEncrypted
        ? "解密该 .x0 文件需要独立口令"
        : "解密失败，请输入口令",
    };
  }
  return {
    title: "输入独立口令",
    content: "转换前需要验证该 .x0 文件的独立口令",
  };
}
