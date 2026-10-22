import type { CodeSnippetGroup } from "@/modules/codeSnippets/groups";
import type { CodeSnippetAppVariant } from "@/models/codeSnippets/variants";

/** 同步到 Rust 的轻量索引（不含密文） */
export type CodeSnippetSyncItem = {
  id: number;
  name: string;
  abbreviation: string;
  shortcut?: string | null;
  group?: CodeSnippetGroup;
};

export type CodeSnippetFrontmostApp = {
  appId: string | null;
  appName: string | null;
  isOwn: boolean;
};

export type CodeSnippetFormValues = {
  name: string;
  group: CodeSnippetGroup;
  abbreviation: string;
  shortcut: string;
  language: string;
  content: string;
  note: string;
  appVariants: CodeSnippetAppVariant[];
};

export type CodeSnippetListItem = {
  id: number;
  name: string;
  group: CodeSnippetGroup;
  abbreviation: string;
  shortcut: string | null;
  language: string | null;
  note: string;
  order: number;
};

export type CodeSnippetPlatform = "macos" | "windows" | "linux" | "unknown";

export type CodeSnippetSettingsView = {
  enabled: boolean;
  inlineExpansionEnabled: boolean;
  inlineExpansionTrigger: string;
  shortcutsEnabled: boolean;
  paletteEnabled: boolean;
};

export type CodeSnippetPaletteItem = {
  id: number;
  name: string;
  abbreviation: string;
  group: CodeSnippetGroup;
};

export type CodeSnippetPermissionsView = {
  platform: CodeSnippetPlatform;
  accessibilityGranted: boolean | null;
  abbreviationSupported: boolean;
  enabled: boolean;
  inlineExpansionEnabled: boolean;
  inlineExpansionTrigger: string;
  shortcutsEnabled: boolean;
  paletteEnabled: boolean;
  paletteShortcut: string;
  listenerActive: boolean;
  registeredAbbreviationCount: number;
  registeredAbbreviations: string[];
};
