import { useEditor, type Editor } from "@tiptap/vue-3";
import { tryOnScopeDispose } from "@vueuse/core";
import { computed, ref, watch, watchEffect, type Ref } from "vue";
import {
  axdocDocsEqual,
  parseAxdocContent,
  serializeAxdoc,
} from "@/modules/editor/axdoc";
import { createRichDocExtensions } from "./extensions";

export type UseRichDocEditorOptions = {
  modelValue: Ref<string> | (() => string);
  readOnly?: Ref<boolean> | (() => boolean);
  onUpdate?: (serialized: string) => void;
};

function readRef<T>(source: Ref<T> | (() => T)): T {
  return typeof source === "function" ? source() : source.value;
}

/** 富文档编辑器生命周期：扩展、axdoc 同步、损坏锁定 */
export function useRichDocEditor(options: UseRichDocEditorOptions) {
  const getModel = () => readRef(options.modelValue);
  const getReadOnly = () =>
    options.readOnly ? readRef(options.readOnly) : false;

  const initialParse = parseAxdocContent(getModel());
  const parseError = ref<string | null>(
    initialParse.ok ? null : initialParse.error,
  );
  const broken = computed(() => Boolean(parseError.value));
  const canEdit = computed(() => !getReadOnly() && !broken.value);

  // TipTap Table 仅在「创建时 editable=true」才会注册 columnResizing；
  // 先以 true 创建，再按 canEdit 同步，避免列宽拖拽永久失效。
  const editor = useEditor({
    content: initialParse.doc,
    editable: true,
    extensions: createRichDocExtensions(),
    onUpdate: ({ editor: current }) => {
      if (broken.value) {
        return;
      }
      const doc = current.getJSON() as Record<string, unknown>;
      const baseline = parseAxdocContent(getModel()).doc;
      if (axdocDocsEqual(doc, baseline)) {
        return;
      }
      options.onUpdate?.(serializeAxdoc(doc));
    },
  });

  watch(
    () => getModel(),
    (value) => {
      const current = editor.value;
      if (!current) {
        return;
      }

      const parsed = parseAxdocContent(value);
      if (!parsed.ok) {
        // 外部写入损坏内容：锁定编辑，保留磁盘原串
        parseError.value = parsed.error;
        current.setEditable(false);
        return;
      }

      parseError.value = null;
      const prev = current.getJSON() as Record<string, unknown>;
      if (axdocDocsEqual(parsed.doc, prev)) {
        return;
      }
      current.commands.setContent(parsed.doc, { emitUpdate: false });
    },
  );

  watchEffect(() => {
    editor.value?.setEditable(canEdit.value);
  });

  tryOnScopeDispose(() => {
    editor.value?.destroy();
  });

  return {
    editor: editor as Ref<Editor | undefined>,
    parseError,
    broken,
    canEdit,
  };
}
