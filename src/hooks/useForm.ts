import type { FormInstance, Rule } from "ant-design-vue/es/form";
import { inject, reactive } from "vue";

export const DIALOG_STATE_KEY = "dialogState";

export type DialogStateLike = { close: () => void };

export interface FormOptions<TModel extends object = Record<string, unknown>> {
  model: TModel;
  rules?: Record<string, Rule[]>;
  onSubmit: (...args: unknown[]) => Promise<void>;
  onCancel?: () => void | Promise<void>;
  /** 非程序化场景下关闭浮层（如页面内 a-modal 的 v-model:open） */
  onClose?: () => void;
}

export interface FormState<TModel extends object = Record<string, unknown>> extends FormOptions<TModel> {
  setFormRef: (formRef: FormInstance | null) => void;
  formRef: FormInstance | null;
  submitLoading: boolean;
  onValidate: () => Promise<void>;
  resetFields: () => void;
  close: () => void;
}

export const useForm = <TModel extends object>(options: FormOptions<TModel>) => {
  const dialogState = inject<DialogStateLike | null>(DIALOG_STATE_KEY, null);
  const state = reactive<FormState<TModel>>({
    ...options,
    setFormRef(formRef) {
      state.formRef = formRef;
    },
    formRef: null,
    rules: options.rules ?? {},
    submitLoading: false,
    async onValidate() {
      if (!state.formRef) return;
      await state.formRef.validate();
    },
    async onSubmit(...args: unknown[]) {
      state.submitLoading = true;
      try {
        await state.onValidate();
        await options.onSubmit(...args);
      } finally {
        state.submitLoading = false;
      }
    },
    async onCancel() {
      await options.onCancel?.();
    },
    resetFields() {
      state.formRef?.resetFields();
    },
    close() {
      options.onClose?.();
      dialogState?.close();
    },
  });
  return state;
};
