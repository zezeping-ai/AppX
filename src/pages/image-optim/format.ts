export type FormatableItem = {
  status: string;
  savedBytes: number;
  beforeBytes: number;
};

export function formatBytes(n: number): string {
  if (n < 0 || Number.isNaN(n)) {
    return "—";
  }
  if (n < 1024) {
    return `${n} B`;
  }
  if (n < 1024 * 1024) {
    return `${(n / 1024).toFixed(1)} KB`;
  }
  return `${(n / (1024 * 1024)).toFixed(2)} MB`;
}

export function formatSaved(item: FormatableItem): string {
  if (item.status === "queued" || item.status === "running" || item.status === "error") {
    return "—";
  }
  if (item.status === "alreadyOptimal" || item.savedBytes <= 0) {
    return "0%";
  }
  if (item.beforeBytes <= 0) {
    return "—";
  }
  const pct = (item.savedBytes / item.beforeBytes) * 100;
  return `${pct.toFixed(1)}%`;
}

export function fileNameFromPath(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] || path;
}
