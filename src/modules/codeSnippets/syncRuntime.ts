import { CodeSnippetRecord } from "@/models";
import { syncSnippetRegistry } from "@/modules/codeSnippets/client";

/** 将数据库中的 snippet 索引同步到 Rust（不含密文，正文触发时查库） */
export async function syncAllSnippetsToRuntime(): Promise<void> {
  const rows = await CodeSnippetRecord.all();
  await syncSnippetRegistry(
    rows.map((row) => ({
      id: row.id as number,
      name: row.name,
      abbreviation: row.abbreviation,
      shortcut: row.shortcut,
      group: row.meta.group,
    })),
  );
}
