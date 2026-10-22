import type { CodeSnippetRecord } from "@/models";
import type { CodeSnippetAppVariant } from "@/models/codeSnippets/variants";
import { validateAppVariants } from "@/models/codeSnippets/variants";
import { decryptText, encryptText } from "@/modules/crypto";

export type SnippetPlaintext = {
  plainContent: string;
  plainVariants: CodeSnippetAppVariant[];
};

/** 编辑前：并行解密默认内容与各应用覆盖（每段仍需一次 Argon2） */
export async function decryptSnippetRecord(
  record: CodeSnippetRecord,
): Promise<SnippetPlaintext> {
  const stored = Array.isArray(record.app_variants) ? record.app_variants : [];
  const [plainContent, ...variantContents] = await Promise.all([
    decryptText(record.content),
    ...stored.map((v) => decryptText(v.content)),
  ]);
  const plainVariants = stored.map((v, i) => ({
    appId: v.appId,
    appName: v.appName,
    content: variantContents[i] ?? "",
  }));
  return { plainContent, plainVariants };
}

export type EncryptedSnippetBody = {
  content: string;
  app_variants: CodeSnippetAppVariant[];
};

/** 保存前：校验并加密默认内容与覆盖 */
export async function encryptSnippetBody(
  plainContent: string,
  plainVariants: CodeSnippetAppVariant[],
): Promise<EncryptedSnippetBody> {
  const err = validateAppVariants(plainVariants);
  if (err) throw new Error(err);

  const [content, ...variantCiphers] = await Promise.all([
    encryptText(plainContent),
    ...plainVariants.map((v) => encryptText(v.content)),
  ]);
  const app_variants = plainVariants.map((v, i) => ({
    appId: v.appId,
    appName: v.appName.trim() || v.appId,
    content: variantCiphers[i] ?? "",
  }));
  return { content, app_variants };
}
