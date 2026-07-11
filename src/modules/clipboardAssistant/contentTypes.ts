import type { ContentType } from "@/modules/clipboardAssistant/types";

export type ContentTypeMeta = {
  value: ContentType;
  label: string;
  icon: string;
  tintVar: string;
};

export const CLIPBOARD_CONTENT_TYPES: ContentTypeMeta[] = [
  { value: "text", label: "文本", icon: "mdi:text", tintVar: "--clipboard-tint-text" },
  { value: "link", label: "链接", icon: "mdi:link-variant", tintVar: "--clipboard-tint-link" },
  { value: "image", label: "图片", icon: "mdi:image-outline", tintVar: "--clipboard-tint-image" },
  { value: "file", label: "文件", icon: "mdi:file-outline", tintVar: "--clipboard-tint-file" },
  { value: "code", label: "代码", icon: "mdi:code-tags", tintVar: "--clipboard-tint-code" },
  { value: "json", label: "JSON", icon: "mdi:code-json", tintVar: "--clipboard-tint-code" },
  { value: "color", label: "颜色", icon: "mdi:palette-outline", tintVar: "--clipboard-tint-text" },
];

export function labelOfContentType(type: ContentType): string {
  return CLIPBOARD_CONTENT_TYPES.find((t) => t.value === type)?.label ?? type;
}

export function iconOfContentType(type: ContentType): string {
  return CLIPBOARD_CONTENT_TYPES.find((t) => t.value === type)?.icon ?? "mdi:clipboard-text-outline";
}

export function tintVarOfContentType(type: ContentType): string {
  return CLIPBOARD_CONTENT_TYPES.find((t) => t.value === type)?.tintVar ?? "--clipboard-tint-text";
}
