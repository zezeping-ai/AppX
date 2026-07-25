import { isAxdocFileName } from "@/modules/editor/axdoc";
import { isEncryptedFileName } from "@/modules/editor/encryption";

function fileNameOf(path: string): string {
  return path.split(/[/\\]/).pop() ?? path;
}

/** AppX 自有格式（.axdoc / 加密）：系统默认应用就是本程序 */
export function isAppxOwnedFile(pathOrName: string): boolean {
  const name = fileNameOf(pathOrName);
  return isEncryptedFileName(name) || isAxdocFileName(name);
}
