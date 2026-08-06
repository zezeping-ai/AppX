/** 简易 fuzzy：子序列匹配，文件名优先，返回分值（越大越好）；不匹配返回 null */

export interface FuzzyMatchResult {
  score: number;
  /** 相对路径上每个字符是否高亮 */
  pathMatches: boolean[];
  /** 文件名上每个字符是否高亮 */
  nameMatches: boolean[];
}

function scoreSubsequence(
  query: string,
  target: string,
): { score: number; matches: boolean[] } | null {
  if (!query) {
    return { score: 0, matches: Array(target.length).fill(false) };
  }

  const q = query.toLowerCase();
  const t = target.toLowerCase();
  const matches = Array(target.length).fill(false);

  let qi = 0;
  let score = 0;
  let consecutive = 0;
  let lastMatch = -2;

  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] !== q[qi]) {
      consecutive = 0;
      continue;
    }

    matches[ti] = true;
    consecutive += 1;
    // 连续命中、路径段起点、文件名开头加权
    score += 10;
    score += consecutive * 5;
    if (ti === 0 || target[ti - 1] === "/" || target[ti - 1] === "-" || target[ti - 1] === "_" || target[ti - 1] === ".") {
      score += 8;
    }
    if (ti === lastMatch + 1) {
      score += 2;
    }
    lastMatch = ti;
    qi += 1;
  }

  if (qi < q.length) {
    return null;
  }

  // 越短越优；越早匹配越优
  score -= target.length;
  score -= lastMatch;
  return { score, matches };
}

export function fuzzyMatchFile(
  query: string,
  name: string,
  relativePath: string,
): FuzzyMatchResult | null {
  const trimmed = query.trim();
  if (!trimmed) {
    return {
      score: -relativePath.length,
      pathMatches: Array(relativePath.length).fill(false),
      nameMatches: Array(name.length).fill(false),
    };
  }

  const nameHit = scoreSubsequence(trimmed, name);
  const pathHit = scoreSubsequence(trimmed, relativePath);
  if (!nameHit && !pathHit) {
    return null;
  }

  // 文件名命中大幅加分（类似 VS Code Quick Open）
  const nameScore = nameHit ? nameHit.score + 40 : Number.NEGATIVE_INFINITY;
  const pathScore = pathHit ? pathHit.score : Number.NEGATIVE_INFINITY;
  const useName = nameScore >= pathScore;

  return {
    score: Math.max(nameScore, pathScore),
    pathMatches: pathHit?.matches ?? Array(relativePath.length).fill(false),
    nameMatches: useName
      ? (nameHit?.matches ?? Array(name.length).fill(false))
      : Array(name.length).fill(false),
  };
}

export function filterFilesByFuzzy<T extends { name: string; relativePath: string }>(
  files: T[],
  query: string,
  limit = 80,
): Array<T & { match: FuzzyMatchResult }> {
  const scored: Array<T & { match: FuzzyMatchResult }> = [];
  for (const file of files) {
    const match = fuzzyMatchFile(query, file.name, file.relativePath);
    if (!match) {
      continue;
    }
    scored.push({ ...file, match });
  }
  scored.sort((a, b) => b.match.score - a.match.score || a.relativePath.localeCompare(b.relativePath));
  return scored.slice(0, limit);
}
