import {
  ActiveRecordPersistence,
  RECORD_BASE_ATTRIBUTE_KEYS,
} from "@/models/ApplicationRecord/ActiveRecordPersistence";
import { isPlainObject, mapValues, omit, pickBy } from "lodash-es";
import type {
  ApplicationRecordAttributes,
  ApplicationRecordCtor,
  PaginateResult,
  QueryOptions,
  RecordInput,
  RecordQueryOptions,
} from "@/models/ApplicationRecord/types";

export abstract class ApplicationRecord extends ActiveRecordPersistence {
  declare static readonly tableName: string;
  static readonly jsonColumns: readonly string[] = ["meta"];

  static normalizeAttributes(attributes: RecordInput): RecordInput {
    return attributes;
  }

  static fromRow<T extends ApplicationRecord>(this: ApplicationRecordCtor<T>, row: Record<string, unknown>): T {
    return this.instantiateFromRow(row);
  }

  id: number | null;
  created_at: string | null;
  updated_at: string | null;
  _meta?: Record<string, unknown> | null;

  protected constructor(attributes: ApplicationRecordAttributes = {}) {
    super();
    const C = this.constructor as typeof ApplicationRecord;
    const normalized = C.normalizeAttributes({
      ...C.defaults,
      ...attributes,
    } as RecordInput);
    this.id = (normalized.id as number | null | undefined) ?? null;
    this.created_at = (normalized.created_at as string | null | undefined) ?? null;
    this.updated_at = (normalized.updated_at as string | null | undefined) ?? null;
    this._meta = (normalized._meta as Record<string, unknown> | null | undefined) ?? null;
    Object.assign(this, normalized);
  }

  toDbRecord(): Record<string, unknown> {
    const C = this.getRecordCtor();
    const jsonCols = C.jsonColumnSet();
    const filtered = pickBy(
      omit(this as unknown as Record<string, unknown>, [...RECORD_BASE_ATTRIBUTE_KEYS]),
      (v) => v !== undefined,
    ) as Record<string, unknown>;
    return mapValues(filtered, (v, k) =>
      jsonCols.has(k) && (isPlainObject(v) || Array.isArray(v)) ? JSON.stringify(v) : v,
    );
  }

  protected beforeSave(): void | Promise<void> {}

  async save(): Promise<this> {
    return this.persistSave();
  }

  async destroy(): Promise<void> {
    const C = this.getRecordCtor();
    if (this.id == null) return;
    await C.deleteById(this.id);
    this.id = null;
  }

  static async all<T extends ApplicationRecord>(this: ApplicationRecordCtor<T>): Promise<T[]> {
    return this.where({}, { orderBy: this.defaultOrderBy });
  }

  static async find<T extends ApplicationRecord>(this: ApplicationRecordCtor<T>, id: number): Promise<T | null> {
    return this.selectOneRecord({ id });
  }

  static async first<T extends ApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    conditions: Record<string, unknown> = {},
    orderBy = "id ASC",
  ): Promise<T | null> {
    return this.selectOneRecord(conditions, { orderBy });
  }

  static async last<T extends ApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    conditions: Record<string, unknown> = {},
    orderBy = "id DESC",
  ): Promise<T | null> {
    return this.selectOneRecord(conditions, { orderBy });
  }

  static async create<T extends ApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    attrs: RecordInput,
  ): Promise<T> {
    const Ctor = this as unknown as new (a?: RecordInput) => T;
    const inst = new Ctor(attrs);
    await inst.save();
    return inst;
  }

  static async update<T extends ApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    id: number,
    attrs: RecordInput,
  ): Promise<T | null> {
    const row = await this.find(id);
    if (!row) return null;
    Object.assign(row, attrs);
    await row.save();
    return row;
  }

  static async where<T extends ApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    conditions: Record<string, unknown> = {},
    options?: QueryOptions,
  ): Promise<T[]> {
    return this.selectRecords(conditions, options);
  }

  static async findBy<T extends ApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    conditions: Record<string, unknown>,
  ): Promise<T | null> {
    return this.selectOneRecord(conditions);
  }

  static async query<T extends ApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    options: RecordQueryOptions,
  ): Promise<PaginateResult<T>> {
    return this.paginatedQuery(options);
  }

  static async updateAll<T extends ApplicationRecord>(
    this: ApplicationRecordCtor<T>,
    conditions: Record<string, unknown>,
    setAttrs: RecordInput,
    opts?: { allowEmptyWhere?: boolean },
  ): Promise<number> {
    if (Object.keys(conditions).length === 0 && !opts?.allowEmptyWhere) {
      throw new Error(`${this.name}.updateAll 需要非空 conditions，或传入 { allowEmptyWhere: true }`);
    }
    const { clause: wClause, binds: wBinds } = this.buildWhere(conditions);
    const { clause: sClause, binds: sBinds } = this.buildSet(setAttrs);
    const whereSql = wClause ? `WHERE ${wClause}` : "";
    const sql = `UPDATE ${this.quoteIdent(this.tableName)} SET ${sClause} ${whereSql}`;
    const res = await this.execute(sql, [...sBinds, ...wBinds]);
    return res.rowsAffected;
  }

  static async destroy<T extends ApplicationRecord>(this: ApplicationRecordCtor<T>, id: number): Promise<number> {
    return this.deleteById(id);
  }
}
