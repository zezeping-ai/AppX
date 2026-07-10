import Database from "@tauri-apps/plugin-sql";
import { getErrorMessage, invokeApp } from "@/shared/tauri/invoke";

export const APP_DATABASE_PATH = import.meta.env.DEV
  ? "sqlite:debug/appx.db"
  : "sqlite:release/appx.db";

export function resolveAppDatabasePath(): Promise<string> {
  return invokeApp<string>("database_resolve_path");
}

let appDatabasePromise: Promise<Database> | null = null;

export function loadAppDatabase(): Promise<Database> {
  appDatabasePromise ??= Database.load(APP_DATABASE_PATH).catch(async (error) => {
    const message = getErrorMessage(error, "");
    if (import.meta.env.DEV && message.includes("previously applied but has been modified")) {
      try {
        await invokeApp("database_reset_dev");
      } finally {
        appDatabasePromise = null;
      }
      return await Database.load(APP_DATABASE_PATH);
    }
    throw error;
  });
  return appDatabasePromise;
}

export type { QueryResult } from "@tauri-apps/plugin-sql";
