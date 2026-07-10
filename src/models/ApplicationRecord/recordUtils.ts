export function isPlainObject(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function omit<T extends Record<string, unknown>>(
  obj: T,
  keys: readonly string[],
): Record<string, unknown> {
  const skip = new Set(keys);
  return Object.fromEntries(Object.entries(obj).filter(([key]) => !skip.has(key)));
}

export function pickBy<T extends Record<string, unknown>>(
  obj: T,
  predicate: (value: unknown, key: string) => boolean,
): Record<string, unknown> {
  return Object.fromEntries(Object.entries(obj).filter(([key, value]) => predicate(value, key)));
}

export function mapValues<T extends Record<string, unknown>>(
  obj: T,
  mapper: (value: unknown, key: string) => unknown,
): Record<string, unknown> {
  return Object.fromEntries(Object.entries(obj).map(([key, value]) => [key, mapper(value, key)]));
}
