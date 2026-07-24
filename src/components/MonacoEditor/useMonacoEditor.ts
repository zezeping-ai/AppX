import * as monaco from "monaco-editor";
import { onBeforeUnmount, onMounted, ref, shallowRef, watch } from "vue";
import { ensureMonacoEnvironment } from "@/components/MonacoEditor/setupMonaco";
import { useThemePreferences } from "@/features/appearance";

export interface UseMonacoEditorOptions {
  language?: () => string;
  readOnly?: () => boolean;
  value: () => string;
  onChange?: (value: string) => void;
  /** 按内容高度自适应容器，关掉编辑器内部纵向滚动 */
  autoHeight?: () => boolean;
  minHeight?: () => number;
  maxHeight?: () => number;
}

export function useMonacoEditor(
  containerRef: { value: HTMLElement | null },
  options: UseMonacoEditorOptions,
) {
  const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const ready = ref(false);
  const { isDark } = useThemePreferences();
  let resizeObserver: ResizeObserver | null = null;

  function monacoTheme() {
    return isDark.value ? "vs-dark" : "vs";
  }

  function syncAutoHeight() {
    const ed = editor.value;
    const el = containerRef.value;
    if (!ed || !el || !options.autoHeight?.()) return;

    const minH = options.minHeight?.() ?? 120;
    const maxH = options.maxHeight?.() ?? Number.POSITIVE_INFINITY;
    const next = Math.min(maxH, Math.max(minH, ed.getContentHeight()));
    if (el.style.height !== `${next}px`) {
      el.style.height = `${next}px`;
    }
    ed.layout({ width: el.clientWidth, height: next });
  }

  onMounted(() => {
    ensureMonacoEnvironment();
    if (!containerRef.value) {
      return;
    }

    const autoHeight = options.autoHeight?.() ?? false;

    editor.value = monaco.editor.create(containerRef.value, {
      value: options.value(),
      language: options.language?.() ?? "plaintext",
      theme: monacoTheme(),
      readOnly: options.readOnly?.() ?? false,
      automaticLayout: !autoHeight,
      minimap: { enabled: false },
      fontSize: 14,
      lineNumbers: "on",
      scrollBeyondLastLine: false,
      wordWrap: "on",
      tabSize: 2,
      padding: { top: 12, bottom: 12 },
      ...(autoHeight
        ? {
            scrollbar: {
              vertical: "hidden",
              horizontal: "auto",
              alwaysConsumeMouseWheel: false,
            },
            overviewRulerLanes: 0,
            hideCursorInOverviewRuler: true,
            overviewRulerBorder: false,
          }
        : {}),
    });

    editor.value.onDidChangeModelContent(() => {
      options.onChange?.(editor.value?.getValue() ?? "");
    });

    if (autoHeight) {
      editor.value.onDidContentSizeChange(() => syncAutoHeight());
      syncAutoHeight();
      // drawer 拖宽时换行高度会变
      resizeObserver = new ResizeObserver(() => syncAutoHeight());
      resizeObserver.observe(containerRef.value);
    }

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
      syncAutoHeight();
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

  watch(isDark, () => {
    monaco.editor.setTheme(monacoTheme());
  });

  onBeforeUnmount(() => {
    resizeObserver?.disconnect();
    resizeObserver = null;
    editor.value?.dispose();
    editor.value = null;
  });

  return {
    editor,
    ready,
  };
}
