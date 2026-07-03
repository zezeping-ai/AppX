export function getErrorMessage(error: unknown, fallback = "未知错误"): string {
  if (error instanceof Error) return error.message || fallback;
  if (typeof error === "string") return error || fallback;
  return fallback;
}
