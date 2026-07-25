import type { Editor } from "@tiptap/vue-3";
import type { Node as PmNode } from "@tiptap/pm/model";

function isEmptyParagraph(node: PmNode): boolean {
  return node.type.name === "paragraph" && node.content.size === 0;
}

/**
 * 去掉 blockquote 末尾连续空段落（包进引用时易把列表后的占位空行一并裹入）。
 * 从后往前删，避免 position 失效。
 */
function trimTrailingEmptyParagraphsInBlockquotes(editor: Editor) {
  const { state } = editor;
  const ranges: { from: number; to: number }[] = [];

  state.doc.descendants((node, pos) => {
    if (node.type.name !== "blockquote") return true;

    const victims: { from: number; to: number }[] = [];
    node.forEach((child, offset) => {
      if (isEmptyParagraph(child)) {
        victims.push({
          from: pos + 1 + offset,
          to: pos + 1 + offset + child.nodeSize,
        });
      } else {
        victims.length = 0;
      }
    });

    // 至少保留一个子节点（blockquote 需要 block+）
    const keptChildren = node.childCount - victims.length;
    if (keptChildren < 1) {
      victims.shift();
    }
    ranges.push(...victims);
    return false;
  });

  if (!ranges.length) return;

  let { tr } = state;
  for (let i = ranges.length - 1; i >= 0; i -= 1) {
    const { from, to } = ranges[i]!;
    tr = tr.delete(from, to);
  }
  editor.view.dispatch(tr);
}

/** 引用切换：避免列表后的空行被包进引用造成大块空白 */
export function toggleBlockquoteSmart(editor: Editor) {
  editor.chain().focus().toggleBlockquote().run();
  trimTrailingEmptyParagraphsInBlockquotes(editor);
}
