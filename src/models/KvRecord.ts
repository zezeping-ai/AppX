import {
  ApplicationRecord,
  type ApplicationRecordAttributes,
  type RecordDefaults,
} from "@/models/ApplicationRecord";

export interface KvRecordAttributes extends ApplicationRecordAttributes {
  namespace?: string;
  key?: string;
  value?: string;
  meta?: Record<string, unknown>;
}

export const defaults = {
  namespace: "default",
  key: "",
  value: "",
  meta: {},
} satisfies RecordDefaults<KvRecordAttributes>;

export class KvRecord extends ApplicationRecord {
  static readonly tableName = "kvs";
  static readonly defaults = defaults;
  static readonly conflictColumns = ["namespace", "key"] as const;
}

export interface KvRecord extends Required<RecordDefaults<KvRecordAttributes>> {}
