import { ActiveRecordCore } from "@/models/ApplicationRecord/ActiveRecordCore";

export const PAGINATION_DEFAULT_PAGE_SIZE = 20;
export const PAGINATION_MAX_PAGE_SIZE = 100;

export type PaginationSlice = {
  page: number;
  pageSize: number;
  total: number;
};

export abstract class ActiveRecordBase extends ActiveRecordCore {
  protected static parseJsonValue(value: unknown): unknown {
    if (typeof value !== "string") return value;
    const s = value.trim();
    if (!s || !/^[\[{"]/.test(s)) return value;
    try {
      return JSON.parse(s) as unknown;
    } catch {
      return value;
    }
  }

  protected static quoteIdent(name: unknown, value?: unknown): string {
    if (typeof name !== "string") throw new Error(`列名必须是字符串: ${String(name)}`);
    const ident = name.trim();
    if (!ident) throw new Error("列名不能为空");
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(ident)) throw new Error(`非法列名: ${ident}`);
    if (value !== undefined && value !== null && !Array.isArray(value) && typeof value === "object") {
      throw new Error(`列 ${ident} 的条件值不支持对象类型`);
    }
    return `"${ident}"`;
  }

  protected static buildWhere(conditions: Record<string, unknown>): { clause: string; binds: unknown[] } {
    const binds: unknown[] = [];
    const parts: string[] = [];
    for (const [colKey, v] of Object.entries(conditions)) {
      if (v === undefined) continue;
      const col = this.quoteIdent(colKey, v);
      if (v === null) {
        parts.push(`${col} IS NULL`);
        continue;
      }
      if (Array.isArray(v)) {
        if (v.length === 0) {
          parts.push("0");
          continue;
        }
        parts.push(`${col} IN (${v.map(() => "?").join(", ")})`);
        binds.push(...v);
        continue;
      }
      parts.push(`${col} = ?`);
      binds.push(v);
    }
    return { clause: parts.join(" AND "), binds };
  }

  protected static buildSet(map: Record<string, unknown>): { clause: string; binds: unknown[] } {
    const binds: unknown[] = [];
    const parts: string[] = [];
    for (const [colKey, v] of Object.entries(map)) {
      if (v === undefined) continue;
      parts.push(`${this.quoteIdent(colKey, v)} = ?`);
      binds.push(v);
    }
    if (!parts.length) throw new Error("updateAll / SET 至少需要一列");
    return { clause: parts.join(", "), binds };
  }

  /** 按顶层逗号拆分 ORDER BY，忽略括号/引号内的逗号 */
  protected static splitOrderBySegments(orderBy: string): string[] {
    const segments: string[] = [];
    let current = "";
    let depth = 0;
    let quote: '"' | "'" | null = null;

    for (const ch of orderBy) {
      if (quote) {
        current += ch;
        if (ch === quote) quote = null;
        continue;
      }
      if (ch === '"' || ch === "'") {
        quote = ch;
        current += ch;
        continue;
      }
      if (ch === "(") {
        depth++;
        current += ch;
        continue;
      }
      if (ch === ")") {
        depth--;
        current += ch;
        continue;
      }
      if (ch === "," && depth === 0) {
        if (current.trim()) segments.push(current.trim());
        current = "";
        continue;
      }
      current += ch;
    }

    if (current.trim()) segments.push(current.trim());
    return segments;
  }

  protected static buildOrderBy(orderBy: string): string {
    const segments = this.splitOrderBySegments(orderBy);
    if (!segments.length) throw new Error(`非法 orderBy: ${orderBy}`);
    return segments
      .map((segment) => {
        const dirMatch = segment.match(/\s+(ASC|DESC)$/i);
        const direction = dirMatch?.[1]?.toUpperCase();
        const expr = (dirMatch ? segment.slice(0, dirMatch.index) : segment).trim();
        if (!expr) throw new Error(`非法 orderBy: ${orderBy}`);

        if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(expr)) {
          const colSql = this.quoteIdent(expr);
          return direction ? `${colSql} ${direction}` : colSql;
        }

        // json_extract 等排序表达式：仅允许安全字符，且必须带 ASC/DESC
        if (!/^[A-Za-z0-9_"'.,$():+\-*/\s]+$/.test(expr)) {
          throw new Error(`非法 orderBy: ${orderBy}`);
        }
        if (direction !== "ASC" && direction !== "DESC") {
          throw new Error(`非法 orderBy: ${orderBy}`);
        }
        return `${expr} ${direction}`;
      })
      .join(", ");
  }

  protected static nowIsoString(): string {
    return new Date().toISOString();
  }

  protected static expandQueryWhere(where: Record<string, unknown>): {
    columns: Record<string, unknown>;
    fragments: { sql: string; binds: unknown[] }[];
  } {
    return { columns: { ...where }, fragments: [] };
  }

  protected static compileQueryWhereClause(where: Record<string, unknown>): { whereSql: string; binds: unknown[] } {
    const { columns, fragments } = this.expandQueryWhere(where);
    const { clause, binds: colBinds } = this.buildWhere(columns);
    const pieces: string[] = [];
    const fragBinds: unknown[] = [];
    if (clause) pieces.push(`(${clause})`);
    for (const f of fragments) {
      const s = f.sql.trim();
      if (s) pieces.push(`(${s})`);
      fragBinds.push(...f.binds);
    }
    const merged = pieces.join(" AND ");
    let i = 1;
    const whereSql = merged.replace(/\?/g, () => `$${i++}`);
    return { whereSql, binds: [...colBinds, ...fragBinds] };
  }

  protected static buildSelectSql(
    tableName: string,
    whereSql = "",
    options?: { orderBy?: string; limit?: number | string; offset?: number | string },
  ): string {
    const parts = [`SELECT * FROM ${this.quoteIdent(tableName)}`];
    const fragment = whereSql.trim();
    if (fragment) parts.push(`WHERE ${fragment}`);
    if (options?.orderBy) parts.push(`ORDER BY ${this.buildOrderBy(options.orderBy)}`);
    if (options?.limit != null) parts.push(`LIMIT ${String(options.limit)}`);
    if (options?.offset != null) parts.push(`OFFSET ${String(options.offset)}`);
    return parts.join(" ");
  }

  protected static normalizePagination(
    page: unknown,
    pageSize: unknown,
    opts?: { maxPageSize?: number },
  ): { page: number; pageSize: number; offset: number } {
    const max = opts?.maxPageSize ?? PAGINATION_MAX_PAGE_SIZE;
    const pageNum = Math.max(1, Math.floor(Number(page)) || 1);
    const ps = Math.min(max, Math.max(1, Math.floor(Number(pageSize)) || PAGINATION_DEFAULT_PAGE_SIZE));
    return { page: pageNum, pageSize: ps, offset: (pageNum - 1) * ps };
  }

  protected static async selectPaginated(
    tableName: string,
    whereSql: string,
    binds: unknown[],
    orderBy: string,
    page: unknown,
    pageSize: unknown,
  ): Promise<{ rows: Record<string, unknown>[] } & PaginationSlice> {
    const { page: pageNum, pageSize: ps, offset } = this.normalizePagination(page, pageSize);
    const table = this.quoteIdent(tableName);
    const n = binds.length;
    const fragment = whereSql.trim();
    const whereClause = fragment ? `WHERE ${fragment}` : "";
    const cntSql = `SELECT COUNT(*) as cnt FROM ${table} ${whereClause}`;
    const dataSql = this.buildSelectSql(tableName, whereSql, {
      orderBy,
      limit: `$${n + 1}`,
      offset: `$${n + 2}`,
    });
    const dataBinds = [...binds, ps, offset];
    const [cntRows, rows] = await Promise.all([
      this.selectRows<{ cnt: number }>(cntSql, binds),
      this.selectRows<Record<string, unknown>>(dataSql, dataBinds),
    ]);
    const total = Number(cntRows[0]?.cnt ?? 0);
    return { rows, page: pageNum, pageSize: ps, total };
  }
}
