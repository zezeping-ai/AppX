import { Crepe } from "@milkdown/crepe";
import { EditorStatus } from "@milkdown/kit/core";
import { replaceAll } from "@milkdown/kit/utils";
import { Milkdown, MilkdownProvider, useEditor } from "@milkdown/vue";
import { tryOnScopeDispose } from "@vueuse/core";
import { defineComponent, h, watch, type Ref } from "vue";

export type UseMarkdownEditorOptions = {
  modelValue: Ref<string> | (() => string);
  readOnly?: Ref<boolean> | (() => boolean);
  onUpdate?: (markdown: string) => void;
};

function readRef<T>(source: Ref<T> | (() => T)): T {
  return typeof source === "function" ? source() : source.value;
}

/** Milkdown Crepe 生命周期：markdown 双向同步、只读 */
export function useMarkdownEditor(options: UseMarkdownEditorOptions) {
  const getModel = () => readRef(options.modelValue);
  const getReadOnly = () =>
    options.readOnly ? readRef(options.readOnly) : false;

  let crepe: Crepe | null = null;
  /** 外部回写时跳过 onUpdate，避免脏标记抖动 */
  let applyingExternal = false;

  const MilkdownEditor = defineComponent({
    name: "MilkdownEditorCore",
    setup() {
      useEditor((root) => {
        const instance = new Crepe({
          root,
          defaultValue: getModel(),
          features: {
            [Crepe.Feature.AI]: false,
            [Crepe.Feature.Latex]: false,
            [Crepe.Feature.TopBar]: false,
          },
        });
        instance.setReadonly(getReadOnly());
        instance.on((listener) => {
          listener.markdownUpdated((_ctx, markdown, prevMarkdown) => {
            if (applyingExternal || markdown === prevMarkdown) {
              return;
            }
            if (markdown === getModel()) {
              return;
            }
            options.onUpdate?.(markdown);
          });
        });
        crepe = instance;
        return instance;
      });
      return () => h(Milkdown);
    },
  });

  const MarkdownEditorHost = defineComponent({
    name: "MarkdownEditorHost",
    setup() {
      return () => h(MilkdownProvider, null, () => h(MilkdownEditor));
    },
  });

  watch(
    () => getModel(),
    (value) => {
      const current = crepe;
      if (!current || current.editor.status !== EditorStatus.Created) {
        return;
      }
      if (current.getMarkdown() === value) {
        return;
      }
      applyingExternal = true;
      try {
        current.editor.action(replaceAll(value));
      } finally {
        applyingExternal = false;
      }
    },
  );

  watch(
    () => getReadOnly(),
    (readonly) => {
      crepe?.setReadonly(readonly);
    },
  );

  tryOnScopeDispose(() => {
    crepe = null;
  });

  return { MarkdownEditorHost };
}
