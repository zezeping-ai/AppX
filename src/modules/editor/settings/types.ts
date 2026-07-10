export interface EditorEncryptionSettings {
  passphrase: string;
  usesGlobalPassphrase: boolean;
}

export interface EditorSettingsView {
  encryption: EditorEncryptionSettings;
}
