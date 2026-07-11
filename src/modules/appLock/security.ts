import { invoke } from "@tauri-apps/api/core";

export interface SecuritySettingsView {
  defaultEncryptionPassphrase: string;
  defaultEncryptionPassphraseEditable: boolean;
}

/** 默认加密口令（应用锁域下的安全子能力，与 Rust security 模块对应） */
export async function getSecuritySettings(): Promise<SecuritySettingsView> {
  return invoke<SecuritySettingsView>("security_get_settings");
}
