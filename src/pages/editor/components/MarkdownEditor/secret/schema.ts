import { commandsCtx, editorViewCtx } from "@milkdown/kit/core";
import type { Ctx } from "@milkdown/kit/ctx";
import type { Node as PmNode, NodeType } from "@milkdown/kit/prose/model";
import { InputRule } from "@milkdown/kit/prose/inputrules";
import { NodeSelection, TextSelection } from "@milkdown/kit/prose/state";
import {
  $command,
  $inputRule,
  $nodeSchema,
  $remark,
} from "@milkdown/kit/utils";
import { remarkSecret } from "./remarkSecret";
import { secretInteraction } from "./interaction";

/** 去掉选区内已有密文，避免包第二段时误删同行密文 */
function clipRangeExcludingType(
  doc: PmNode,
  from: number,
  to: number,
  type: NodeType,
): { from: number; to: number } {
  let nextFrom = from;
  let nextTo = to;
  doc.nodesBetween(from, to, (node, pos) => {
    if (node.type !== type) {
      return true;
    }
    const end = pos + node.nodeSize;
    if (pos <= nextFrom && end > nextFrom) {
      nextFrom = end;
    }
    if (pos < nextTo && end >= nextTo) {
      nextTo = pos;
    }
    if (pos > nextFrom && end < nextTo) {
      nextTo = pos;
    }
    return true;
  });
  if (nextFrom > nextTo) {
    return { from, to: from };
  }
  return { from: nextFrom, to: nextTo };
}

function maskText(value: string) {
  return "•".repeat(Math.max(value.length, 3));
}

const EYE_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"><path fill="currentColor" d="M12 9a3 3 0 0 0-3 3a3 3 0 0 0 3 3a3 3 0 0 0 3-3a3 3 0 0 0-3-3m0 8a5 5 0 0 1-5-5a5 5 0 0 1 5-5a5 5 0 0 1 5 5a5 5 0 0 1-5 5m0-12.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5"/></svg>';
const COPY_SVG =
  '<svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24"><path fill="currentColor" d="M19 21H8V7h11m0-2H8a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h11a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2m-3-4H4a2 2 0 0 0-2 2v14h2V3h12V1Z"/></svg>';

function createSecretDom(value: string): HTMLSpanElement {
  const root = document.createElement("span");
  root.dataset.type = SECRET_NODE_ID;
  root.dataset.value = value;
  root.className = "md-secret";
  root.contentEditable = "false";

  const text = document.createElement("span");
  text.className = "md-secret__text";
  text.textContent = maskText(value);
  root.append(text);

  const reveal = document.createElement("button");
  reveal.type = "button";
  reveal.className = "md-secret__btn";
  reveal.dataset.action = "reveal";
  reveal.tabIndex = -1;
  reveal.title = "显示";
  reveal.setAttribute("aria-label", "显示");
  reveal.innerHTML = EYE_SVG;
  root.append(reveal);

  const copy = document.createElement("button");
  copy.type = "button";
  copy.className = "md-secret__btn";
  copy.dataset.action = "copy";
  copy.tabIndex = -1;
  copy.title = "复制";
  copy.setAttribute("aria-label", "复制");
  copy.innerHTML = COPY_SVG;
  root.append(copy);

  return root;
}

export const SECRET_NODE_ID = "secret";

export const remarkSecretPlugin = $remark("remarkSecret", () => remarkSecret);

export const secretSchema = $nodeSchema(SECRET_NODE_ID, () => ({
  group: "inline",
  inline: true,
  atom: true,
  selectable: true,
  draggable: false,
  attrs: {
    value: { default: "" },
  },
  parseDOM: [
    {
      tag: `span[data-type="${SECRET_NODE_ID}"]`,
      getAttrs: (dom) => {
        if (!(dom instanceof HTMLElement)) {
          return false;
        }
        return { value: dom.dataset.value ?? "" };
      },
    },
  ],
  toDOM: (node) => createSecretDom(String(node.attrs.value ?? "")),
  parseMarkdown: {
    match: (node) => node.type === SECRET_NODE_ID,
    runner: (state, node, type) => {
      state.addNode(type, { value: String(node.value ?? "") });
    },
  },
  toMarkdown: {
    match: (node) => node.type.name === SECRET_NODE_ID,
    runner: (state, node) => {
      const value = String(node.attrs.value ?? "");
      state.addNode("html", undefined, `||${value}||`);
    },
  },
}));

/**
 * 文本选区 → 包成密文；仅 NodeSelection 落在密文上时还原。
 * 包完后光标落在密文后（TextSelection），不强制 NodeSelection，避免键盘选区异常。
 */
export const toggleSecretCommand = $command(
  "ToggleSecret",
  (ctx) => () => (state, dispatch) => {
    const type = secretSchema.type(ctx);
    const { selection } = state;

    if (selection instanceof NodeSelection && selection.node.type === type) {
      const pos = selection.from;
      const content = String(selection.node.attrs.value ?? "");
      let next = state.tr.delete(pos, pos + selection.node.nodeSize);
      next = next.insertText(content, pos);
      if (dispatch) {
        dispatch(
          next.setSelection(
            TextSelection.create(next.doc, pos, pos + content.length),
          ),
        );
      }
      return true;
    }

    const clipped = clipRangeExcludingType(
      state.doc,
      selection.from,
      selection.to,
      type,
    );
    const text = state.doc.textBetween(clipped.from, clipped.to);
    const node = type.create({ value: text });
    const next = state.tr.replaceWith(clipped.from, clipped.to, node);
    if (dispatch) {
      // 选中新建密文，便于立刻用悬浮条「取消密文」
      dispatch(
        next.setSelection(NodeSelection.create(next.doc, clipped.from)),
      );
    }
    return true;
  },
);

/** 斜杠菜单：删掉 `/query` 后插入空密文 */
export function insertSecretFromSlash(ctx: Ctx) {
  const view = ctx.get(editorViewCtx);
  const { state } = view;
  const { $from } = state.selection;
  const parentStart = $from.start();
  const text = $from.parent.textBetween(0, $from.parentOffset, "\n", "\0");
  const slashAt = text.lastIndexOf("/");
  const type = secretSchema.type(ctx);
  let tr = state.tr;

  if (slashAt >= 0) {
    const from = parentStart + slashAt;
    const to = $from.pos;
    tr = tr.delete(from, to);
    tr = tr.insert(from, type.create({ value: "" }));
    tr = tr.setSelection(NodeSelection.create(tr.doc, from));
  } else {
    const from = state.selection.from;
    tr = tr.replaceSelectionWith(type.create({ value: "" }));
    tr = tr.setSelection(NodeSelection.create(tr.doc, from));
  }
  view.dispatch(tr.scrollIntoView());
}

export function runToggleSecret(ctx: Ctx) {
  ctx.get(commandsCtx).call(toggleSecretCommand.key);
}

export const secretInputRule = $inputRule(
  (ctx) =>
    new InputRule(/(?:\|\|)([^\n|]+)(?:\|\|)$/, (state, match, start, end) => {
      const value = match[1] ?? "";
      const node = secretSchema.type(ctx).create({ value });
      return state.tr.replaceWith(start, end, node);
    }),
);

export const secretPlugins = [
  remarkSecretPlugin,
  secretSchema,
  secretInteraction,
  toggleSecretCommand,
  secretInputRule,
].flat();
