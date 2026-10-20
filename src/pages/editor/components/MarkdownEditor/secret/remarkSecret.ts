/** remark：把段落文本里的 `||value||` 拆成 secret AST 节点 */

type MdastNode = {
  type: string;
  value?: string;
  children?: MdastNode[];
};

const SECRET_RE = /\|\|([\s\S]*?)\|\|/g;

function splitText(value: string): MdastNode[] {
  const parts: MdastNode[] = [];
  let last = 0;
  for (const match of value.matchAll(SECRET_RE)) {
    const start = match.index ?? 0;
    if (start > last) {
      parts.push({ type: "text", value: value.slice(last, start) });
    }
    parts.push({ type: "secret", value: match[1] ?? "" });
    last = start + match[0].length;
  }
  if (last === 0) {
    return [{ type: "text", value }];
  }
  if (last < value.length) {
    parts.push({ type: "text", value: value.slice(last) });
  }
  return parts;
}

function transformNode(node: MdastNode): void {
  const children = node.children;
  if (!children?.length) {
    return;
  }

  const next: MdastNode[] = [];
  for (const child of children) {
    if (child.type === "text" && typeof child.value === "string" && child.value.includes("||")) {
      next.push(...splitText(child.value));
      continue;
    }
    transformNode(child);
    next.push(child);
  }
  node.children = next;
}

/** unified/remark 插件：解析阶段展开 `||...||` */
export function remarkSecret() {
  return (tree: MdastNode) => {
    transformNode(tree);
  };
}
