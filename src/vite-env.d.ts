/// <reference types="vite/client" />

declare module "*.vue" {
  import type { DefineComponent } from "vue";
  const component: DefineComponent<object, object, unknown>;
  export default component;
}

declare module "*?worker" {
  const WorkerConstructor: new () => Worker;
  export default WorkerConstructor;
}

interface ImportMetaEnv {
  readonly VITE_APP_TITLE: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare global {
  interface Window {
    MonacoEnvironment?: {
      getWorker: (workerId: string, label: string) => Worker;
    };
  }
}

export {};
