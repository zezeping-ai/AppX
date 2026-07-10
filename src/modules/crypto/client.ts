import { invoke } from "@tauri-apps/api/core";

export async function encryptText(plaintext: string): Promise<string> {
  return invoke<string>("crypto_encrypt_text", { plaintext });
}

export async function decryptText(ciphertextB64: string): Promise<string> {
  return invoke<string>("crypto_decrypt_text", { ciphertextB64 });
}
