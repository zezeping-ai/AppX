import { addAPIProvider, addCollection } from "@iconify/vue";
import { getIcons } from "@iconify/utils";
import mdi from "@iconify-json/mdi/icons.json";

/** 项目内用到的 MDI 图标；新增图标时同步补充此列表 */
const USED_MDI_ICONS = [
  "api",
  "application-edit-outline",
  "chevron-down",
  "chevron-left",
  "chevron-right",
  "close",
  "code-tags",
  "comment-text-outline",
  "console",
  "database-outline",
  "email-outline",
  "file-document-outline",
  "file-key-outline",
  "file-lock-outline",
  "file-plus-outline",
  "folder-open-outline",
  "folder-outline",
  "folder-plus-outline",
  "git",
  "key-outline",
  "lightning-bolt-outline",
  "map-marker-outline",
  "markdown-outline",
  "palette-outline",
  "plus",
  "refresh",
  "shield-lock-outline",
  "tag-outline",
  "unfold-less-horizontal",
] as const;

/** 离线注册图标，避免 Tauri/WebKit 请求 Iconify CDN 触发 CORS */
export function setupLocalIcons() {
  const collection = getIcons(mdi, [...USED_MDI_ICONS]);
  if (!collection) return;

  addCollection(collection);

  // 禁用远程 API，缺失图标时不再发起网络请求
  addAPIProvider("", { resources: [] });
}
