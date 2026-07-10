import type Database from "@tauri-apps/plugin-sql";
import type { QueryResult } from "@/shared/tauri";
import { loadAppDatabase } from "@/shared/tauri";

export abstract class ActiveRecordCore {
  protected static async database(): Promise<Database> {
    return loadAppDatabase();
  }

  protected static async selectRows<TRow>(sql: string, bindValues: unknown[] = []): Promise<TRow[]> {
    const db = await this.database();
    return db.select<TRow[]>(sql, bindValues);
  }

  protected static async execute(sql: string, bindValues: unknown[] = []): Promise<QueryResult> {
    const db = await this.database();
    return db.execute(sql, bindValues);
  }
}
