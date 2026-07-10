import { invoke, type InvokeArgs } from "@tauri-apps/api/core";

export type AppInvokeErrorCode = "config" | "io" | "tauri" | "error" | "unknown";

export class AppInvokeError extends Error {
  code: AppInvokeErrorCode;

  constructor(payload: { code?: string; message: string }) {
    super(payload.message);
    this.name = "AppInvokeError";
    this.code = (payload.code as AppInvokeErrorCode | undefined) ?? "unknown";
  }
}

export function getErrorMessage(error: unknown, fallback = "未知错误"): string {
  if (error instanceof AppInvokeError) return error.message || fallback;
  if (error instanceof Error) return error.message || fallback;
  if (typeof error === "string") return error;
  return fallback;
}

export async function invokeApp<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (error) {
    if (error instanceof Error) {
      throw new AppInvokeError({ message: error.message });
    }
    throw new AppInvokeError({ message: String(error) });
  }
}
