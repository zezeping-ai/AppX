import { ActiveRecordBase } from "@/models/ApplicationRecord/ActiveRecordBase";
import { mapValues } from "lodash-es";
import type {
  AnyApplicationRecord,
  ApplicationRecordCtor,
  PaginateResult,
  QueryOptions,
  RecordInput,
  RecordQueryOptions,
} from "@/models/ApplicationRecord/types";

/** 不参与 INSERT/UPDATE 的基础字段（与子类业务列区分） */
export const RECORD_BASE_ATTRIBUTE_KEYS = ["id", "created_at", "updated_at", "_meta"] as const;

type PersistPayload = {
  record: Record<string, unknown>;
  cols: string[];
  vals: unknown[];
  now: string;
};

type InsertPayload = {
  insertCols: string[];
  insertVals: unknown[];
  placeholders: string;
  quotedTableName: string;
  quotedInsertCols: string;
};

export abstract class ActiveRecordPersistence extends ActiveRecordBase {
  static readonly defaultOrderBy: string = "id DESC";
  static readonly defaults: Partial<RecordInput> = {};
  static readonly conflictColumns: readonly string[] | null = null;
  static readonly jsonColumns: readonly string[] = [];

  abstract id: number | null;

  protected static instantiateFromRow<T extends AnyApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    row: Record<string, unknown>,
  ): T {
    const C = this as unknown as new (attrs?: RecordInput) => T;
    const jsonCols = this.jsonColumnSet();
    const attrs = mapValues(row, (v, k) => (jsonCols.has(k) ? this.parseJsonValue(v) : v));
    return new C(attrs as RecordInput);
  }

  protected static recordsFromRows<T extends AnyApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    rows: Record<string, unknown>[],
  ): T[] {
    return rows.map((row) => this.instantiateFromRow(row));
  }

  protected static async selectRecords<T extends AnyApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    conditions: Record<string, unknown> = {},
    options?: QueryOptions,
  ): Promise<T[]> {
    const { whereSql, binds } = this.compileQueryWhereClause(conditions);
    const sql = this.buildSelectSql(this.tableName, whereSql, options);
    const rows = await this.selectRows<Record<string, unknown>>(sql, binds);
    return this.recordsFromRows(rows);
  }

  protected static async selectOneRecord<T extends AnyApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    conditions: Record<string, unknown> = {},
    options?: QueryOptions,
  ): Promise<T | null> {
    const rows = await this.selectRecords(conditions, { ...options, limit: 1 });
    return rows[0] ?? null;
  }

  protected static async deleteById<T extends AnyApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    id: number,
  ): Promise<number> {
    const res = await this.execute(
      `DELETE FROM ${this.quoteIdent(this.tableName)} WHERE ${this.quoteIdent("id")} = $1`,
      [id],
    );
    return res.rowsAffected;
  }

  protected static jsonColumnSet(): Set<string> {
    return new Set((this.jsonColumns ?? []) as readonly string[]);
  }

  protected getRecordCtor(): ApplicationRecordCtor<AnyApplicationRecord> {
    return this.constructor as ApplicationRecordCtor<AnyApplicationRecord>;
  }

  public abstract toDbRecord(): Record<string, unknown>;

  protected abstract beforeSave(): void | Promise<void>;

  private applySnapshot(next: this | null): this {
    if (!next) throw new Error(`${this.getRecordCtor().name} save 后未找到记录`);
    Object.assign(this, next);
    return this;
  }

  private async reloadById(C: ApplicationRecordCtor<AnyApplicationRecord>, id: number): Promise<this> {
    return this.applySnapshot((await C.selectOneRecord({ id })) as this | null);
  }

  private async reloadByConflict(
    C: ApplicationRecordCtor<AnyApplicationRecord>,
    conflict: readonly string[],
    record: Record<string, unknown>,
  ): Promise<this> {
    const conditions = Object.fromEntries(conflict.map((col) => [col, record[col]]));
    return this.applySnapshot((await C.selectOneRecord(conditions)) as this | null);
  }

  private buildPersistPayload(): PersistPayload {
    const record = this.toDbRecord();
    const cols = Object.keys(record);
    return {
      record,
      cols,
      vals: cols.map((col) => record[col]),
      now: ActiveRecordPersistence.nowIsoString(),
    };
  }

  private async persistUpdate(
    C: ApplicationRecordCtor<AnyApplicationRecord>,
    payload: PersistPayload,
  ): Promise<this> {
    const { cols, vals, now } = payload;
    const setParts = cols.map((col, index) => `${C.quoteIdent(col)} = $${index + 1}`);
    setParts.push(`${C.quoteIdent("updated_at")} = $${cols.length + 1}`);
    const binds = [...vals, now, this.id];
    const sql = `UPDATE ${C.quoteIdent(C.tableName)} SET ${setParts.join(", ")} WHERE ${C.quoteIdent("id")} = $${cols.length + 2}`;
    await C.execute(sql, binds);
    return this.reloadById(C, this.id as number);
  }

  private buildInsertPayload(
    C: ApplicationRecordCtor<AnyApplicationRecord>,
    payload: PersistPayload,
  ): InsertPayload {
    const insertCols = [...payload.cols, "created_at", "updated_at"];
    return {
      insertCols,
      insertVals: [...payload.vals, payload.now, payload.now],
      placeholders: insertCols.map((_, index) => `$${index + 1}`).join(", "),
      quotedTableName: C.quoteIdent(C.tableName),
      quotedInsertCols: insertCols.map((col) => C.quoteIdent(col)).join(", "),
    };
  }

  private async persistUpsert(
    C: ApplicationRecordCtor<AnyApplicationRecord>,
    payload: PersistPayload,
    insert: InsertPayload,
    conflict: readonly string[],
  ): Promise<this> {
    const conflictSql = conflict.map((col) => C.quoteIdent(col)).join(", ");
    const updateParts = payload.cols
      .filter((col) => !conflict.includes(col))
      .map((col) => `${C.quoteIdent(col)} = excluded.${C.quoteIdent(col)}`);
    updateParts.push(`${C.quoteIdent("updated_at")} = excluded.${C.quoteIdent("updated_at")}`);
    const sql = `INSERT INTO ${insert.quotedTableName} (${insert.quotedInsertCols}) VALUES (${insert.placeholders})
      ON CONFLICT(${conflictSql}) DO UPDATE SET ${updateParts.join(", ")}`;
    await C.execute(sql, insert.insertVals);
    return this.reloadByConflict(C, conflict, payload.record);
  }

  private async persistInsert(
    C: ApplicationRecordCtor<AnyApplicationRecord>,
    payload: PersistPayload,
  ): Promise<this> {
    const insert = this.buildInsertPayload(C, payload);
    const conflict = C.conflictColumns ?? null;

    if (conflict?.length) {
      return this.persistUpsert(C, payload, insert, conflict);
    }

    const sql = `INSERT INTO ${insert.quotedTableName} (${insert.quotedInsertCols}) VALUES (${insert.placeholders})`;
    const res = await C.execute(sql, insert.insertVals);
    if (res.lastInsertId == null) throw new Error(`${C.name} INSERT 后缺少 lastInsertId`);
    return this.reloadById(C, Number(res.lastInsertId));
  }

  private applyConventions(C: ApplicationRecordCtor<AnyApplicationRecord>): void {
    const current = this as Record<string, unknown>;
    for (const [field, value] of Object.entries(current)) {
      if (RECORD_BASE_ATTRIBUTE_KEYS.includes(field as (typeof RECORD_BASE_ATTRIBUTE_KEYS)[number])) continue;
      if (typeof value === "string") current[field] = value.trim();
    }
    for (const field of C.conflictColumns ?? []) {
      const value = current[field];
      if (typeof value === "string" && value.length === 0) throw new Error(`${C.name}.${field} 不能为空`);
      if (value == null) throw new Error(`${C.name}.${field} 不能为空`);
    }
  }

  protected async persistSave(): Promise<this> {
    await Promise.resolve(this.beforeSave());
    const C = this.getRecordCtor();
    this.applyConventions(C);
    const payload = this.buildPersistPayload();
    if (this.id != null) return this.persistUpdate(C, payload);
    return this.persistInsert(C, payload);
  }

  protected static async paginatedQuery<T extends AnyApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    options: RecordQueryOptions,
  ): Promise<PaginateResult<T>> {
    const { where = {}, pagination, orderBy = this.defaultOrderBy } = options;
    const page = pagination?.page ?? 1;
    const pageSize = pagination?.pageSize;
    const { whereSql, binds } = this.compileQueryWhereClause(where);
    const slice = await this.selectPaginated(this.tableName, whereSql, binds, orderBy, page, pageSize);
    return {
      data: this.recordsFromRows(slice.rows),
      page: slice.page,
      pageSize: slice.pageSize,
      total: slice.total,
    };
  }
}
