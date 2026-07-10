import { useLocalStorage } from "@vueuse/core";

const WORKSPACE_ROOT_KEY = "appx.workspace.root";

export function useWorkspacePersistence() {
  const savedWorkspaceRoot = useLocalStorage<string | null>(WORKSPACE_ROOT_KEY, null);

  function rememberWorkspaceRoot(path: string | null) {
    savedWorkspaceRoot.value = path;
  }

  return {
    savedWorkspaceRoot,
    rememberWorkspaceRoot,
  };
}
