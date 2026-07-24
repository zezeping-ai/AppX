/** AppX 富文档（.axdoc）格式与 TipTap 内容约定 */

import { cloneDeep, isEqual, isPlainObject } from "lodash-es";

export const APPXDOC_LANGUAGE = "appxdoc";
export const AXDOC_EXTENSION = "axdoc";
export const DEFAULT_AXDOC_FILE_NAME = "untitled.axdoc.x";
export const DEFAULT_TEXT_FILE_NAME = "untitled.txt.x";

const AXDOC_VERSION = 1;

const AXDOC_NAME_SUFFIXES = [
  `.${AXDOC_EXTENSION}`,
  `.${AXDOC_EXTENSION}.x`,
  `.${AXDOC_EXTENSION}.x0`,
] as const;

export type AxdocEnvelope = {
  v: number;
  doc: Record<string, unknown>;
};

const EMPTY_AXDOC_DOC: Record<string, unknown> = {
  type: "doc",
  content: [{ type: "paragraph" }],
};

export type AxdocParseResult =
  | { ok: true; doc: Record<string, unknown> }
  | { ok: false; error: string; doc: Record<string, unknown> };

function emptyDoc(): Record<string, unknown> {
  return cloneDeep(EMPTY_AXDOC_DOC);
}

function fail(error: string): AxdocParseResult {
  return { ok: false, error, doc: emptyDoc() };
}

/** 新建文档写入磁盘的默认内容 */
export function emptyAxdocContent(): string {
  return serializeAxdoc(emptyDoc());
}

export function isAppxdocLanguage(language: string | undefined | null): boolean {
  return language === APPXDOC_LANGUAGE;
}

/** 按文件名判断是否为 axdoc（含 .axdoc.x / .axdoc.x0） */
export function isAxdocFileName(fileName: string): boolean {
  const lower = fileName.trim().toLowerCase();
  return AXDOC_NAME_SUFFIXES.some((suffix) => lower.endsWith(suffix));
}

export function serializeAxdoc(doc: Record<string, unknown>): string {
  const envelope: AxdocEnvelope = { v: AXDOC_VERSION, doc };
  return JSON.stringify(envelope);
}

export function axdocDocsEqual(
  a: Record<string, unknown>,
  b: Record<string, unknown>,
): boolean {
  return isEqual(a, b);
}

/**
 * 解析磁盘内容为 TipTap doc。
 * 空内容 → 合法空文档；非空但无法识别 → ok:false（调用方勿静默覆盖保存）。
 * 返回的 doc 均为深拷贝，避免 TipTap 改写共享常量。
 */
export function parseAxdocContent(raw: string): AxdocParseResult {
  const trimmed = raw.trim();
  if (!trimmed) {
    return { ok: true, doc: emptyDoc() };
  }

  try {
    const parsed = JSON.parse(trimmed) as unknown;
    if (!isPlainObject(parsed)) {
      return fail("富文本文档格式无效（根节点不是对象）");
    }

    const obj = parsed as Record<string, unknown>;
    if (isPlainObject(obj.doc)) {
      const doc = obj.doc as Record<string, unknown>;
      if (doc.type === "doc") {
        return { ok: true, doc: cloneDeep(doc) };
      }
      return fail("富文本文档格式无效（缺少 type: doc）");
    }

    if (obj.type === "doc") {
      return { ok: true, doc: cloneDeep(obj) };
    }

    return fail("不是有效的 AppX 富文本文档");
  } catch {
    return fail("富文本文档不是合法 JSON，无法解析");
  }
}
