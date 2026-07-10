export interface ApplicationRecordAttributes {
  id?: number | null;
  created_at?: string | null;
  updated_at?: string | null;
}

export type RecordInput = ApplicationRecordAttributes & Record<string, unknown>;

export type RecordDefaults<T extends ApplicationRecordAttributes> = Required<
  Omit<T, keyof ApplicationRecordAttributes>
>;

export type QueryOptions = {
  orderBy?: string;
  limit?: number;
  offset?: number;
};

export interface RecordQueryPagination {
  page: number;
  pageSize: number;
}

export interface RecordQueryOptions {
  where?: Record<string, unknown>;
  pagination?: Partial<RecordQueryPagination>;
  orderBy?: string;
}

export interface PaginateResult<T = unknown> {
  data: T[];
  page: number;
  pageSize: number;
  total: number;
}

export type AnyApplicationRecord = import("@/models/ApplicationRecord/ApplicationRecord").ApplicationRecord;

export type ApplicationRecordCtor<T extends AnyApplicationRecord> =
  typeof import("@/models/ApplicationRecord/ApplicationRecord").ApplicationRecord & {
    readonly prototype: T;
    readonly tableName: string;
    readonly defaultOrderBy: string;
    readonly defaults?: Partial<RecordInput>;
    readonly conflictColumns?: readonly string[] | null;
    readonly jsonColumns?: readonly string[];
    normalizeAttributes?(attributes: RecordInput): RecordInput;
  };
