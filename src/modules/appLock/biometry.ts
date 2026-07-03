export type BiometryStatusView =
  | { available: true; typeLabel: string }
  | { available: false; reason: string };

const AUTH_OPTIONS = {
  allowDeviceCredential: true,
  fallbackTitle: "使用密码",
} as const;

function mapBiometryType(type: number): string {
  if (type === 2) return "Touch ID";
  if (type === 3) return "Face ID";
  if (type === 4) return "Iris";
  if (type === 1) return "Windows Hello";
  return "Biometry";
}

/** 用户主动取消验证（非认证失败） */
export function isBiometryUserDismissed(error: unknown): boolean {
  const text = String(error);
  return text.includes("userCancel") || text.includes("systemCancel");
}

export async function getBiometryStatusView(): Promise<BiometryStatusView> {
  try {
    const mod = await import("@choochmeque/tauri-plugin-biometry-api");
    const status = await mod.checkStatus();
    if (!status.isAvailable) {
      return { available: false, reason: status.error ? String(status.error) : "不可用" };
    }
    return { available: true, typeLabel: mapBiometryType(status.biometryType) };
  } catch (error) {
    return { available: false, reason: String(error) };
  }
}

export async function authenticateBiometry(reason: string) {
  const mod = await import("@choochmeque/tauri-plugin-biometry-api");
  await mod.authenticate(reason, AUTH_OPTIONS);
}

