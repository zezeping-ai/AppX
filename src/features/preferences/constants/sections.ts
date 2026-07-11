import type { Component } from "vue";
import { ApplicationSettingsSection } from "@/features/app";
import { AppLockSettingsSection } from "@/features/appLock";
import { CodeSnippetsSettingsSection } from "@/features/codeSnippets";
import { ClipboardAssistantSettingsSection } from "@/features/clipboardAssistant";
import { EditorSettingsSection } from "@/features/editor";

export type PreferenceSectionKey = "app" | "security" | "editor" | "code-snippets" | "clipboard-assistant";

export type PreferenceSection = {
  key: PreferenceSectionKey;
  label: string;
  icon: string;
  component: Component;
};

export const APP_PREFERENCE_SECTIONS: PreferenceSection[] = [
  {
    key: "app",
    label: "应用",
    icon: "mdi:cog-outline",
    component: ApplicationSettingsSection,
  },
  {
    key: "security",
    label: "安全",
    icon: "mdi:shield-lock-outline",
    component: AppLockSettingsSection,
  },
];

export const FEATURE_PREFERENCE_SECTIONS: PreferenceSection[] = [
  {
    key: "editor",
    label: "编辑器",
    icon: "mdi:application-edit-outline",
    component: EditorSettingsSection,
  },
  {
    key: "code-snippets",
    label: "代码段",
    icon: "mdi:code-braces",
    component: CodeSnippetsSettingsSection,
  },
  {
    key: "clipboard-assistant",
    label: "剪切助手",
    icon: "mdi:clipboard-text-clock-outline",
    component: ClipboardAssistantSettingsSection,
  },
];

export const PREFERENCE_SECTIONS: PreferenceSection[] = [
  ...APP_PREFERENCE_SECTIONS,
  ...FEATURE_PREFERENCE_SECTIONS,
];
