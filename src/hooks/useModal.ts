import type { VNode } from "vue";
import { ConfigProvider, Modal as AModal, type ModalProps } from "ant-design-vue";
import { isEmpty, omit } from "lodash-es";
import { useEventListener } from "@vueuse/core";
import {
  createVNode,
  defineComponent,
  getCurrentInstance,
  inject,
  provide,
  reactive,
  ref,
  render,
  unref,
  type Slot,
} from "vue";
import { antdProgrammaticRootConfigKey } from "@/hooks/antdProgrammaticContext";
import { DIALOG_STATE_KEY, type DialogStateLike } from "@/hooks/useForm";
import { ROUTE_CHANGE_EVENT } from "@/router";

export type ModalSlotMap = Record<string, Slot | (() => VNode)>;

export type ModalCreateOptions = Omit<ModalProps, "open"> & {
  slots: ModalSlotMap;
  onClosed?: () => void;
};

const ModalShell = defineComponent({
  name: "ProgrammaticModalHost",
  props: {
    bindOpen: { type: Object, required: true },
    modalProps: { type: Object, required: true },
    slotsMap: { type: Object, required: true },
    providerProps: { type: Object, default: null },
    onTeardown: { type: Function, required: true },
  },
  setup(props) {
    const bindOpen = props.bindOpen as ReturnType<typeof ref<boolean>>;
    const dialogState: DialogStateLike = reactive({
      close: () => {
        bindOpen.value = false;
      },
    });
    provide(DIALOG_STATE_KEY, dialogState);
    useEventListener(window, ROUTE_CHANGE_EVENT, dialogState.close);

    return () => {
      const slots = props.slotsMap as ModalSlotMap;
      const normalized = Object.fromEntries(
        Object.keys(slots).map((key) => [
          key,
          () => {
            const s = slots[key];
            if (typeof s === "function") return (s as () => VNode)();
            return null;
          },
        ]),
      );
      const mp = props.modalProps as ModalProps;
      const modal = createVNode(
        AModal,
        {
          getContainer: false,
          ...mp,
          open: bindOpen.value,
          "onUpdate:open": (v: boolean) => {
            bindOpen.value = v;
          },
          afterClose: () => {
            mp.afterClose?.();
            (props.onTeardown as () => void)();
          },
        },
        normalized,
      );
      const pp = props.providerProps as Record<string, unknown> | null;
      return pp && !isEmpty(pp) ? createVNode(ConfigProvider, pp, { default: () => modal }) : modal;
    };
  },
});

export const useModal = (defaultOptions: Partial<ModalCreateOptions> = {}) => {
  const instance = getCurrentInstance();
  console.assert(!!instance, "useModal 需在 setup 内调用");
  const rootConfig = inject(antdProgrammaticRootConfigKey, null);

  return {
    create(options: ModalCreateOptions) {
      const div = document.createElement("div");
      div.className = "antd-programmatic-modal-root";
      document.body.appendChild(div);

      const bindOpen = ref(true);
      const { slots, onClosed, ...rest } = options;
      const defs = omit(defaultOptions, ["slots", "onClosed"]) as Partial<ModalProps>;

      const teardown = () => {
        render(null, div);
        div.parentNode?.removeChild(div);
        onClosed?.();
      };

      const vnode = createVNode(ModalShell, {
        bindOpen,
        modalProps: { ...defs, ...rest } as ModalProps,
        slotsMap: slots,
        providerProps: rootConfig ? unref(rootConfig) : null,
        onTeardown: teardown,
      });
      vnode.appContext = instance!.appContext;
      render(vnode, div);

      return {
        close: () => {
          bindOpen.value = false;
        },
      };
    },
  };
};
