import type { VNode } from "vue";
import {
  ConfigProvider,
  Drawer as ADrawer,
  type DrawerProps,
} from "ant-design-vue";
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

export type DrawerSlotMap = Record<string, Slot | (() => VNode)>;

export type DrawerCreateOptions = Omit<DrawerProps, "open"> & {
  slots: DrawerSlotMap;
  onClosed?: () => void;
  /** 拖拽左缘调整宽度（仅 right 抽屉，默认开启） */
  resizable?: boolean;
  minWidth?: number;
  maxWidth?: number;
};

/** 将 Drawer width（数字或百分比字符串）解析为像素初值 */
function resolveInitialWidth(raw: DrawerProps["width"]): number {
  if (typeof raw === "number" && Number.isFinite(raw)) return raw;
  if (typeof raw === "string") {
    const pct = raw.trim().match(/^(\d+(?:\.\d+)?)%$/);
    if (pct) return Math.round((window.innerWidth * Number(pct[1])) / 100);
    const num = Number(raw);
    if (Number.isFinite(num)) return num;
  }
  return 520;
}

const DrawerShell = defineComponent({
  name: "ProgrammaticDrawerHost",
  props: {
    bindOpen: { type: Object, required: true },
    drawerProps: { type: Object, required: true },
    slotsMap: { type: Object, required: true },
    providerProps: { type: Object, default: null },
    onTeardown: { type: Function, required: true },
  },
  setup(props) {
    const bindOpen = props.bindOpen as ReturnType<typeof ref<boolean>>;
    const drawerProps = props.drawerProps as DrawerProps & {
      resizable?: boolean;
      minWidth?: number;
      maxWidth?: number;
    };
    const rawWidth = drawerProps.width;
    const initialWidth = resolveInitialWidth(rawWidth);
    const minWidth = drawerProps.minWidth ?? 400;
    const maxWidth = drawerProps.maxWidth ?? Math.round(window.innerWidth * 0.92);
    const placement = drawerProps.placement ?? "right";
    const resizable = drawerProps.resizable !== false && placement === "right";

    function clampWidth(next: number) {
      return Math.min(maxWidth, Math.max(minWidth, next));
    }

    const width = ref(clampWidth(initialWidth));

    const dialogState: DialogStateLike = reactive({
      close: () => {
        bindOpen.value = false;
      },
    });
    provide(DIALOG_STATE_KEY, dialogState);

    function onResizeStart(event: MouseEvent) {
      event.preventDefault();
      const startX = event.clientX;
      const startWidth = width.value;
      document.body.style.cursor = "col-resize";
      document.body.style.userSelect = "none";

      const stopMove = useEventListener(document, "mousemove", (moveEvent: MouseEvent) => {
        width.value = clampWidth(startWidth + (startX - moveEvent.clientX));
      });
      const stopUp = useEventListener(document, "mouseup", () => {
        stopMove();
        stopUp();
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
      });
    }

    useEventListener(window, ROUTE_CHANGE_EVENT, dialogState.close);

    return () => {
      const slots = props.slotsMap as DrawerSlotMap;
      const normalized = Object.fromEntries(
        Object.keys(slots).map((key) => [
          key,
          () => {
            const s = slots[key];
            const content = typeof s === "function" ? (s as () => VNode)() : null;
            if (!resizable || key !== "default" || !content) return content;
            return createVNode(
              "div",
              { class: "relative flex min-h-0 min-w-0 flex-1 flex-col" },
              [
                createVNode("div", {
                  class:
                    "absolute inset-y-0 left-0 z-20 w-1.5 shrink-0 cursor-col-resize select-none hover:bg-black/10 dark:hover:bg-white/15",
                  onMousedown: onResizeStart,
                }),
                content,
              ],
            );
          },
        ]),
      );
      const {
        resizable: _resizable,
        minWidth: _minWidth,
        maxWidth: _maxWidth,
        width: _rawWidth,
        bodyStyle: rawBodyStyle,
        ...dp
      } = drawerProps;
      const bodyStyle = resizable
        ? {
            display: "flex",
            flexDirection: "column",
            minHeight: 0,
            overflow: "auto",
            ...(typeof rawBodyStyle === "object" && rawBodyStyle ? rawBodyStyle : {}),
          }
        : rawBodyStyle;
      const drawer = createVNode(
        ADrawer,
        {
          getContainer: false,
          ...dp,
          bodyStyle,
          // 非可拖拽场景需透传原始宽度（支持 "100%" 等字符串），否则会被误置为 undefined。
          width: resizable ? width.value : _rawWidth,
          open: bindOpen.value,
          "onUpdate:open": (v: boolean) => {
            bindOpen.value = v;
          },
          onAfterOpenChange: (opened: boolean) => {
            dp.onAfterOpenChange?.(opened);
            if (!opened) (props.onTeardown as () => void)();
          },
        },
        normalized,
      );
      const pp = props.providerProps as Record<string, unknown> | null;
      return pp && !isEmpty(pp) ? createVNode(ConfigProvider, pp, { default: () => drawer }) : drawer;
    };
  },
});

export const useDrawer = (defaultOptions: Partial<DrawerCreateOptions> = {}) => {
  const instance = getCurrentInstance();
  console.assert(!!instance, "useDrawer 需在 setup 内调用");
  const rootConfig = inject(antdProgrammaticRootConfigKey, null);

  return {
    create(options: DrawerCreateOptions) {
      const div = document.createElement("div");
      div.className = "antd-programmatic-drawer-root";
      document.body.appendChild(div);

      const bindOpen = ref(true);
      const { slots, onClosed, ...rest } = options;
      const defs = omit(defaultOptions, ["slots", "onClosed"]) as Partial<DrawerProps>;

      const teardown = () => {
        render(null, div);
        div.parentNode?.removeChild(div);
        onClosed?.();
      };

      const vnode = createVNode(DrawerShell, {
        bindOpen,
        drawerProps: { ...defs, ...rest } as DrawerProps,
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
