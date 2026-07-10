import type { CodeSnippetGroup } from "@/modules/codeSnippets/groups";

export type CodeSnippetSyncItem = {
  id: number;
  name: string;
  abbreviation: string;
  shortcut?: string | null;
  content: string;
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
  inlineExpansionEnabled: boolean;
};

export type CodeSnippetPaletteItem = {
  id: number;
  name: string;
  abbreviation: string;
};

export type CodeSnippetPermissionsView = {
  platform: CodeSnippetPlatform;
  accessibilityGranted: boolean | null;
  abbreviationSupported: boolean;
  inlineExpansionEnabled: boolean;
  paletteShortcut: string;
  listenerActive: boolean;
  registeredAbbreviationCount: number;
  registeredAbbreviations: string[];
};
