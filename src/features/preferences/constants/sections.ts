import type { Component } from "vue";
import AppearanceSettingsSection from "@/features/appearance/AppearanceSettingsSection.vue";
import { EditorSettingsSection } from "@/features/editor/settings";
import { AppLockSettingsSection } from "@/features/security";

export type PreferenceSectionKey = "appearance" | "security" | "editor";

export type PreferenceSection = {
  key: PreferenceSectionKey;
  label: string;
  icon: string;
  component: Component;
};

export const PREFERENCE_SECTIONS: PreferenceSection[] = [
  {
    key: "appearance",
    label: "主题",
    icon: "mdi:palette-outline",
    component: AppearanceSettingsSection,
  },
  {
    key: "security",
    label: "安全",
    icon: "mdi:shield-lock-outline",
    component: AppLockSettingsSection,
  },
  {
    key: "editor",
    label: "Editor",
    icon: "mdi:application-edit-outline",
    component: EditorSettingsSection,
  },
];
