import { useLocalStorage } from "@vueuse/core";
import { without } from "lodash-es";

const MAX_RECENT = 10;
const RECENT_FOLDERS_KEY = "appx.recent.folders";
const RECENT_FILES_KEY = "appx.recent.files";

function bumpRecent(list: string[], path: string): string[] {
  return [path, ...without(list, path)].slice(0, MAX_RECENT);
}

export function useRecentOpens() {
  const recentFolders = useLocalStorage<string[]>(RECENT_FOLDERS_KEY, []);
  const recentFiles = useLocalStorage<string[]>(RECENT_FILES_KEY, []);

  function rememberRecentFolder(path: string) {
    recentFolders.value = bumpRecent(recentFolders.value, path);
  }

  function rememberRecentFile(path: string) {
    recentFiles.value = bumpRecent(recentFiles.value, path);
  }

  function removeRecentFolder(path: string) {
    recentFolders.value = without(recentFolders.value, path);
  }

  function removeRecentFile(path: string) {
    recentFiles.value = without(recentFiles.value, path);
  }

  return {
    recentFolders,
    recentFiles,
    rememberRecentFolder,
    rememberRecentFile,
    removeRecentFolder,
    removeRecentFile,
  };
}
