import { omitBy } from "lodash-es";

/** 去掉空值查询字段，避免无意义参数参与请求或 URL 序列化 */
export function trimQuery(query: Record<string, unknown>): Record<string, unknown> {
  return omitBy(query, (value) => value === undefined || value === null || value === "") as Record<string, unknown>;
}
