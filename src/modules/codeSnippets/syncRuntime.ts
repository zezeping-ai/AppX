import { CodeSnippetRecord } from "@/models";
import { syncSnippetRegistry } from "@/modules/codeSnippets/client";

/** 将数据库中的 snippet 同步到 Rust 运行时注册表 */
export async function syncAllSnippetsToRuntime(): Promise<void> {
  const rows = await CodeSnippetRecord.all();
  await syncSnippetRegistry(
    rows.map((row) => ({
      id: row.id as number,
      name: row.name,
      abbreviation: row.abbreviation,
      shortcut: row.shortcut,
      content: row.content,
      group: row.meta.group,
    })),
  );
}
