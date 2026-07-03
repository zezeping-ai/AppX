import * as monaco from "monaco-editor";
import { onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import { ensureMonacoEnvironment } from "@/components/MonacoEditor/setupMonaco";

export interface UseMonacoEditorOptions {
  language?: () => string;
  readOnly?: () => boolean;
  value: () => string;
  onChange?: (value: string) => void;
}

export function useMonacoEditor(
  containerRef: { value: HTMLElement | null },
  options: UseMonacoEditorOptions,
) {
  const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const ready = ref(false);

  onMounted(() => {
    ensureMonacoEnvironment();
    if (!containerRef.value) {
      return;
    }

    editor.value = monaco.editor.create(containerRef.value, {
      value: options.value(),
      language: options.language?.() ?? "plaintext",
      theme: "vs",
      readOnly: options.readOnly?.() ?? false,
      automaticLayout: true,
      minimap: { enabled: false },
      fontSize: 14,
      lineNumbers: "on",
      scrollBeyondLastLine: false,
      wordWrap: "on",
      tabSize: 2,
      padding: { top: 12, bottom: 12 },
    });

    editor.value.onDidChangeModelContent(() => {
      options.onChange?.(editor.value?.getValue() ?? "");
    });

    ready.value = true;
  });

  watch(
    () => options.value(),
    (nextValue) => {
      const current = editor.value;
      if (!current) {
        return;
      }
      if (current.getValue() !== nextValue) {
        current.setValue(nextValue);
      }
    },
  );

  watch(
    () => options.language?.() ?? "plaintext",
    (language) => {
      const model = editor.value?.getModel();
      if (model) {
        monaco.editor.setModelLanguage(model, language);
      }
    },
  );

  watch(
    () => options.readOnly?.() ?? false,
    (readOnly) => {
      editor.value?.updateOptions({ readOnly });
    },
  );

  onBeforeUnmount(() => {
    editor.value?.dispose();
    editor.value = null;
  });

  return {
    editor,
    ready,
  };
}
