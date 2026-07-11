import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { resolveAppDatabasePath } from "@/shared/tauri";

export async function getDatabasePath(): Promise<string> {
  return resolveAppDatabasePath();
}

/** 在系统文件管理器中定位 SQLite 数据库文件 */
export async function revealDatabaseInFolder(): Promise<void> {
  await revealItemInDir(await getDatabasePath());
}
