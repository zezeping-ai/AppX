import { invoke } from "@tauri-apps/api/core";
import type { SecuritySettingsView } from "@/modules/security/types";

export async function getSecuritySettings(): Promise<SecuritySettingsView> {
  return invoke<SecuritySettingsView>("security_get_settings");
}
