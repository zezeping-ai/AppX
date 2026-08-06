export type HighlightSegment = {
  text: string;
  hit: boolean;
};

/** 将布尔 marks 压成连续高亮段 */
export function segmentsFromMarks(text: string, marks: boolean[]): HighlightSegment[] {
  const out: HighlightSegment[] = [];
  let buf = "";
  let hit = false;
  for (let i = 0; i < text.length; i++) {
    const m = Boolean(marks[i]);
    if (i === 0) {
      hit = m;
      buf = text[i]!;
      continue;
    }
    if (m === hit) {
      buf += text[i]!;
    } else {
      out.push({ text: buf, hit });
      buf = text[i]!;
      hit = m;
    }
  }
  if (buf) {
    out.push({ text: buf, hit });
  }
  return out;
}

/** 字符下标区间 `[start, end)` → 高亮段（按 Unicode 码点） */
export function segmentsFromRange(
  text: string,
  startChar: number,
  endChar: number,
): HighlightSegment[] {
  const chars = Array.from(text);
  const start = Math.max(0, Math.min(startChar, chars.length));
  const end = Math.max(start, Math.min(endChar, chars.length));
  const out: HighlightSegment[] = [];
  if (start > 0) {
    out.push({ text: chars.slice(0, start).join(""), hit: false });
  }
  if (end > start) {
    out.push({ text: chars.slice(start, end).join(""), hit: true });
  }
  if (end < chars.length) {
    out.push({ text: chars.slice(end).join(""), hit: false });
  }
  return out;
}
