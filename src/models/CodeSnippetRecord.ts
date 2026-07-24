import {
  ApplicationRecord,
  type ApplicationRecordAttributes,
  type RecordDefaults,
} from "@/models/ApplicationRecord";
import type { CodeSnippetGroup } from "@/models/codeSnippets/groups";
import { DEFAULT_CODE_SNIPPET_GROUP } from "@/models/codeSnippets/groups";

export interface CodeSnippetMeta {
  group: CodeSnippetGroup;
  language: string | null;
  note: string;
  order: number;
}

export interface CodeSnippetRecordAttributes extends ApplicationRecordAttributes {
  name?: string;
  abbreviation?: string;
  shortcut?: string | null;
  content?: string;
  meta?: CodeSnippetMeta;
}

export const defaults = {
  name: "",
  abbreviation: "",
  shortcut: null,
  content: "",
  meta: {
    group: DEFAULT_CODE_SNIPPET_GROUP,
    language: null,
    note: "",
    order: 0,
  } satisfies CodeSnippetMeta,
} satisfies RecordDefaults<CodeSnippetRecordAttributes>;

export class CodeSnippetRecord extends ApplicationRecord {
  static readonly tableName = "code_snippets";
  static readonly defaults = defaults;
  static override readonly defaultOrderBy =
    `json_extract("meta", '$.order') ASC, id ASC` as string;

  protected static expandQueryWhere(where: Record<string, unknown>): {
    columns: Record<string, unknown>;
    fragments: { sql: string; binds: unknown[] }[];
  } {
    const columns = { ...where };
    const fragments: { sql: string; binds: unknown[] }[] = [];

    const keywordRaw = columns.keyword;
    delete columns.keyword;
    const keyword = typeof keywordRaw === "string" ? keywordRaw.trim() : "";
    if (keyword) {
      const like = `%${keyword}%`;
      fragments.push({
        sql: `(
          LOWER(${this.quoteIdent("name")}) LIKE LOWER(?)
          OR LOWER(${this.quoteIdent("abbreviation")}) LIKE LOWER(?)
        )`,
        binds: [like, like],
      });
    }

    // 快捷键筛选：由录入器产出规范化串，精确匹配
    const shortcutRaw = columns.shortcut;
    delete columns.shortcut;
    const shortcut = typeof shortcutRaw === "string" ? shortcutRaw.trim() : "";
    if (shortcut) {
      fragments.push({
        sql: `${this.quoteIdent("shortcut")} = ?`,
        binds: [shortcut],
      });
    }

    const group = columns.group;
    delete columns.group;
    if (group !== undefined && group !== null && group !== "") {
      fragments.push({
        sql: `json_extract(${this.quoteIdent("meta")}, '$.group') = ?`,
        binds: [group],
      });
    }
    return { columns, fragments };
  }
}

export interface CodeSnippetRecord extends Required<RecordDefaults<CodeSnippetRecordAttributes>> {}
