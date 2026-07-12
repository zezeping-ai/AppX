export type PaletteLayout = "topPanel" | "bottomPanel" | "leftPanel" | "rightPanel";

/** 兼容旧版布局值 */
export function normalizePaletteLayout(layout?: string): PaletteLayout {
  if (layout === "bottomStrip" || layout === "bottomList") return "bottomPanel";
  if (
    layout === "topPanel" ||
    layout === "bottomPanel" ||
    layout === "leftPanel" ||
    layout === "rightPanel"
  ) {
    return layout;
  }
  return "bottomPanel";
}

export type PaletteGeometry = {
  layout?: PaletteLayout;
  x?: number;
  y?: number;
  width?: number;
  height?: number;
};

export type PayloadKind = "inline" | "blob" | "fileRef";

export type ContentType =
  | "text"
  | "link"
  | "image"
  | "file"
  | "code"
  | "color"
  | "json";

export type ContentBadge = {
  kind: string;
  label: string;
};

export type ClipboardItemSummary = {
  id: number;
  contentType: ContentType;
  preview: string;
  sourceAppBundle?: string;
  sourceAppName?: string;
  sourceAppIconUrl?: string;
  groupKey: string;
  pinned: boolean;
  createdAt: string;
  accentColor: string;
  charCount?: number;
  tags: string[];
  badges: ContentBadge[];
  thumbUrl?: string;
  relativeTime: string;
  hasRichFormat: boolean;
};

export type ListItemsQuery = {
  keyword?: string;
  contentType?: ContentType;
  groupKey?: string;
  sourceAppBundle?: string;
  pinnedOnly?: boolean;
  limit?: number;
  offset?: number;
  preferCache?: boolean;
};

export type ListItemsResult = {
  items: ClipboardItemSummary[];
  total: number;
};

export type ApplyAction = "paste" | "copy";

export type ApplyFormat = "plain" | "rich";

export type MutateOp = "pin" | "unpin" | "delete" | "clearUnpinned";

export type GetContentResult = {
  contentType: ContentType;
  payloadKind: PayloadKind;
  text?: string;
  filePaths?: string[];
  hasBlob: boolean;
};

export type ClipboardAssistantStatus = {
  monitoringActive: boolean;
  paletteActive: boolean;
  paletteShortcut: string;
  totalCount: number;
  unpinnedCount: number;
  pinnedCount: number;
  blobBytes: number;
  cacheRevision: number;
};

export type ClipboardAssistantSettings = {
  enabled: boolean;
  monitoringEnabled: boolean;
  paletteEnabled: boolean;
  paletteShortcut: string;
  maxHistoryItems: number;
  paletteLayout: PaletteLayout;
  paletteAnchor: string;
  paletteWidth: number;
  paletteHeight: number;
  paletteEdgeMargin: number;
  rememberWindowPosition: boolean;
  autoHideOnPaste: boolean;
  autoHideOnClickOutside: boolean;
  openSearchOnShow: boolean;
  dedupeMode: string;
  paletteMaxItems: number;
  showSourceAppIcon: boolean;
  autoSweepOrphansOnStartup: boolean;
  textInlineThreshold: number;
  maxTextBytes: number;
  maxImageBlobBytes: number;
  maxImageBlobHardBytes: number;
  compressOversizedImages: boolean;
  excludedApps: string[];
  clearOnLock: boolean;
};

export type SaveClipboardAssistantSettingsInput = ClipboardAssistantSettings;
