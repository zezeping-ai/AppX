import { labelOfContentType } from "@/modules/clipboardAssistant/contentTypes";
import type {
  ClipboardItemSummary,
  ContentBadge,
} from "@/modules/clipboardAssistant/types";

/** 是否具备富文本复制/粘贴能力 */
export function itemHasRichFormat(item: ClipboardItemSummary): boolean {
  return item.hasRichFormat || item.badges.some((badge) => badge.kind === "rich");
}

/** 过滤与 contentType 标签重复的 badge（如类型已是「图片」时不再展示同名徽章） */
export function extraBadgesOf(item: ClipboardItemSummary): ContentBadge[] {
  const typeLabel = labelOfContentType(item.contentType);
  return item.badges.filter(
    (badge) => badge.kind !== item.contentType && badge.label !== typeLabel,
  );
}

/** 将 ISO createdAt 格式化为本地可读时间；失败则返回空串 */
export function formatItemCreatedAt(iso?: string): string {
  if (!iso?.trim()) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
