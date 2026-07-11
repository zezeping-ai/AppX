import { Modal } from "ant-design-vue";
import { h, nextTick, ref } from "vue";
import { Input } from "ant-design-vue";

export interface PassphrasePromptOptions {
  title: string;
  content?: string;
  okText?: string;
}

export function promptPassphrase(options: PassphrasePromptOptions): Promise<string | null> {
  const passphrase = ref("");
  const inputRef = ref<{ focus?: () => void } | null>(null);

  function focusInput() {
    nextTick(() => {
      requestAnimationFrame(() => {
        inputRef.value?.focus?.();
      });
    });
  }

  return new Promise((resolve) => {
    Modal.confirm({
      title: options.title,
      autoFocusButton: null,
      content: () =>
        h("div", { class: "flex flex-col gap-2" }, [
          options.content
            ? h("p", { class: "m-0 text-[12px] text-black/55" }, options.content)
            : null,
          h(Input.Password, {
            ref: inputRef,
            autofocus: true,
            value: passphrase.value,
            placeholder: "请输入加密口令",
            "onUpdate:value": (value: string) => {
              passphrase.value = value;
            },
            onVnodeMounted: focusInput,
            onPressEnter: () => {
              const trimmed = passphrase.value.trim();
              if (!trimmed) return;
              Modal.destroyAll();
              resolve(trimmed);
            },
          }),
        ]),
      okText: options.okText ?? "确认",
      cancelText: "取消",
      onOk: () => {
        const trimmed = passphrase.value.trim();
        if (!trimmed) {
          return Promise.reject(new Error("口令不能为空"));
        }
        resolve(trimmed);
      },
      onCancel: () => resolve(null),
    });
  });
}
