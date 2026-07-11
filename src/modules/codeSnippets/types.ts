import type { CodeSnippetGroup } from "@/modules/codeSnippets/groups";

export type CodeSnippetSyncItem = {
  id: number;
  name: string;
  abbreviation: string;
  shortcut?: string | null;
  content: string;
  group?: CodeSnippetGroup;
};

export type CodeSnippetFormValues = {
  name: string;
  group: CodeSnippetGroup;
  abbreviation: string;
  shortcut: string;
  language: string;
  content: string;
  note: string;
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
