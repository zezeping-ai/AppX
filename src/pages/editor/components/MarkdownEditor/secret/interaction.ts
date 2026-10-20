import { NodeSelection, TextSelection, Plugin } from "@milkdown/kit/prose/state";
import type { EditorView } from "@milkdown/kit/prose/view";
import { $prose } from "@milkdown/kit/utils";
import { message } from "ant-design-vue";

const SECRET_TYPE = "secret";

function maskText(value: string) {
  return "•".repeat(Math.max(value.length, 3));
}

function chipFromEvent(event: Event): HTMLElement | null {
  const target = event.target;
  if (!(target instanceof Element)) {
    return null;
  }
  return target.closest<HTMLElement>(`span[data-type="${SECRET_TYPE}"]`);
}

function btnFromEvent(event: Event): HTMLButtonElement | null {
  const target = event.target;
  if (!(target instanceof Element)) {
    return null;
  }
  return target.closest<HTMLButtonElement>("button.md-secret__btn");
}

function posOfSecretChip(view: EditorView, chip: HTMLElement): number | null {
  try {
    const pos = view.posAtDOM(chip, 0);
    const nodeAt = view.state.doc.nodeAt(pos);
    if (nodeAt?.type.name === SECRET_TYPE) {
      return pos;
    }
    const $pos = view.state.doc.resolve(pos);
    if ($pos.nodeAfter?.type.name === SECRET_TYPE) {
      return pos;
    }
    if ($pos.nodeBefore?.type.name === SECRET_TYPE) {
      return pos - $pos.nodeBefore.nodeSize;
    }
  } catch {
    /* ignore */
  }
  return null;
}

function isSecretNodeSelection(view: EditorView) {
  const { selection } = view.state;
  return (
    selection instanceof NodeSelection &&
    selection.node.type.name === SECRET_TYPE
  );
}

function unwrapSelectedSecret(view: EditorView) {
  const { state } = view;
  const { selection } = state;
  if (
    !(selection instanceof NodeSelection) ||
    selection.node.type.name !== SECRET_TYPE
  ) {
    return;
  }
  const pos = selection.from;
  const content = String(selection.node.attrs.value ?? "");
  let tr = state.tr.delete(pos, pos + selection.node.nodeSize);
  tr = tr.insertText(content, pos);
  tr = tr.setSelection(
    TextSelection.create(tr.doc, pos, pos + content.length),
  );
  view.dispatch(tr);
  view.focus();
}

async function copyValue(value: string) {
  if (!value) {
    return;
  }
  try {
    await navigator.clipboard.writeText(value);
    message.success("已复制");
  } catch {
    message.error("复制失败");
  }
}

function toggleChipReveal(chip: HTMLElement) {
  const value = chip.dataset.value ?? "";
  const textEl = chip.querySelector(".md-secret__text");
  if (!(textEl instanceof HTMLElement)) {
    return;
  }
  const next = !chip.classList.contains("md-secret--revealed");
  chip.classList.toggle("md-secret--revealed", next);
  textEl.textContent = next ? value || "密文" : maskText(value);
  const revealBtn = chip.querySelector<HTMLButtonElement>(
    'button[data-action="reveal"]',
  );
  if (revealBtn) {
    revealBtn.title = next ? "隐藏" : "显示";
  }
}

function createFloatBar(view: EditorView) {
  const bar = document.createElement("div");
  bar.className = "md-secret-float";
  bar.style.display = "none";

  const mkBtn = (label: string, action: string) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "md-secret-float__btn";
    btn.dataset.action = action;
    btn.textContent = label;
    btn.addEventListener("mousedown", (event) => {
      event.preventDefault();
      event.stopPropagation();
    });
    return btn;
  };

  const unwrapBtn = mkBtn("取消密文", "unwrap");
  const revealBtn = mkBtn("显示", "reveal");
  const copyBtn = mkBtn("复制", "copy");
  bar.append(unwrapBtn, revealBtn, copyBtn);

  bar.addEventListener("click", (event) => {
    event.preventDefault();
    event.stopPropagation();
    const btn = (event.target as Element | null)?.closest?.("button");
    if (!(btn instanceof HTMLButtonElement)) {
      return;
    }
    const action = btn.dataset.action;
    if (action === "unwrap") {
      unwrapSelectedSecret(view);
      return;
    }
    if (!isSecretNodeSelection(view)) {
      return;
    }
    const chip = view.nodeDOM(view.state.selection.from);
    if (!(chip instanceof HTMLElement)) {
      return;
    }
    if (action === "reveal") {
      toggleChipReveal(chip);
      const revealed = chip.classList.contains("md-secret--revealed");
      btn.textContent = revealed ? "隐藏" : "显示";
      return;
    }
    if (action === "copy") {
      void copyValue(chip.dataset.value ?? "");
    }
  });

  return bar;
}

function placeFloatBar(view: EditorView, bar: HTMLElement) {
  if (!isSecretNodeSelection(view)) {
    bar.style.display = "none";
    return;
  }
  const chip = view.nodeDOM(view.state.selection.from);
  if (!(chip instanceof HTMLElement)) {
    bar.style.display = "none";
    return;
  }
  const host =
    view.dom.closest<HTMLElement>(".markdown-editor") ??
    view.dom.parentElement;
  if (!host) {
    bar.style.display = "none";
    return;
  }
  if (bar.parentElement !== host) {
    host.append(bar);
  }
  const hostRect = host.getBoundingClientRect();
  const chipRect = chip.getBoundingClientRect();
  bar.style.display = "flex";
  // 先显示再量高度
  const barH = bar.offsetHeight || 32;
  const left = chipRect.left - hostRect.left + host.scrollLeft;
  const top = chipRect.top - hostRect.top + host.scrollTop - barH - 6;
  bar.style.left = `${Math.max(8, left)}px`;
  bar.style.top = `${Math.max(8, top)}px`;

  const revealBtn = bar.querySelector<HTMLButtonElement>(
    'button[data-action="reveal"]',
  );
  if (revealBtn) {
    revealBtn.textContent = chip.classList.contains("md-secret--revealed")
      ? "隐藏"
      : "显示";
  }
}

/** 点击选中 + 悬浮操作条（Crepe 工具条不支持 NodeSelection） */
export const secretInteraction = $prose(() => {
  let floatBar: HTMLElement | null = null;

  return new Plugin({
    view: (view) => {
      floatBar = createFloatBar(view);
      placeFloatBar(view, floatBar);
      return {
        update: (v) => {
          if (floatBar) {
            placeFloatBar(v, floatBar);
          }
        },
        destroy: () => {
          floatBar?.remove();
          floatBar = null;
        },
      };
    },
    props: {
      handleDOMEvents: {
        mousedown(view, event) {
          const btn = btnFromEvent(event);
          if (btn) {
            // 芯片内按钮：不抢焦点
            event.preventDefault();
            return true;
          }
          const chip = chipFromEvent(event);
          if (!chip) {
            return false;
          }
          const pos = posOfSecretChip(view, chip);
          if (pos == null) {
            return false;
          }
          // 点击芯片本体 → NodeSelection，弹出悬浮条
          view.dispatch(
            view.state.tr.setSelection(NodeSelection.create(view.state.doc, pos)),
          );
          view.focus();
          event.preventDefault();
          return true;
        },
        click(_view, event) {
          const btn = btnFromEvent(event);
          const chip = chipFromEvent(event);
          if (!btn || !chip) {
            return false;
          }
          event.preventDefault();
          event.stopPropagation();
          const action = btn.dataset.action;
          if (action === "reveal") {
            toggleChipReveal(chip);
            return true;
          }
          if (action === "copy") {
            void copyValue(chip.dataset.value ?? "");
            return true;
          }
          return true;
        },
      },
    },
  });
});
