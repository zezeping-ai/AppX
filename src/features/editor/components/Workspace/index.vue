<script setup lang="tsx">
import { Modal, message } from "ant-design-vue";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { uniq } from "lodash-es";
import EditorTabs from "@/features/editor/components/EditorTabs/index.vue";
import FileExplorer from "@/features/editor/components/FileExplorer/index.vue";
import { toExplorerTree } from "@/features/editor/components/FileExplorer/normalizeTree";
import { confirmDiscardUnsaved } from "./confirmDiscardUnsaved";
import OpenWithRecent from "./OpenWithRecent/index.vue";
import { useRecentOpens } from "./useRecentOpens";
import { useWorkspacePersistence } from "./useWorkspacePersistence";
import MonacoEditor from "@/components/MonacoEditor/index.vue";
import {
  createDirectory,
  createFile,
  deletePath,
  inspectFile,
  listDirectory,
  pickFile,
  pickFolder,
  readFile,
  renamePath,
} from "@/modules/editor/client";
import { fileStatusMark, isEncryptedFileName } from "@/modules/editor/encryption";
import { createOpenedEditorFile } from "@/modules/editor/openedFile";
import type { OpenedEditorFile } from "@/modules/editor/types";
import { useEncryptedFileFlow } from "./useEncryptedFileFlow";

const { savedWorkspaceRoot, rememberWorkspaceRoot } = useWorkspacePersistence();
const {
  recentFolders,
  recentFiles,
  rememberRecentFolder,
  rememberRecentFile,
  removeRecentFolder,
  removeRecentFile,
} = useRecentOpens();

const workspaceRoot = ref<string | null>(null);
const treeData = ref<ReturnType<typeof toExplorerTree>>([]);
const loadingTree = ref(false);
const saving = ref(false);

const tabByPath = ref<Map<string, OpenedEditorFile>>(new Map());
const activePath = ref<string | null>(null);
const editorContent = ref("");
let unlistenDragDrop: null | (() => void) = null;

const openTabs = computed(() => [...tabByPath.value.values()]);
const activeFile = computed(() =>
  activePath.value ? (tabByPath.value.get(activePath.value) ?? null) : null,
);
const hasActiveFile = computed(() => Boolean(activeFile.value));
const activeLanguage = computed(() => activeFile.value?.language ?? "plaintext");

const statusText = computed(() => {
  if (!activeFile.value) {
    return "";
  }
  const dirtyMark = activeFile.value.dirty ? " *" : "";
  return `${activeFile.value.path}${fileStatusMark(activeFile.value)}${dirtyMark}`;
});

function syncEditorFromActiveTab() {
  editorContent.value = activeFile.value?.content ?? "";
}

function updateActiveTabContent(content: string, dirty?: boolean) {
  if (!activePath.value) {
    return;
  }
  const tab = tabByPath.value.get(activePath.value);
  if (!tab) {
    return;
  }
  const nextTab: OpenedEditorFile = {
    ...tab,
    content,
    dirty: dirty ?? content !== tab.content,
  };
  tabByPath.value = new Map(tabByPath.value).set(activePath.value, nextTab);
}

async function refreshTree() {
  if (!workspaceRoot.value) {
    treeData.value = [];
    return;
  }

  loadingTree.value = true;
  try {
    const nodes = await listDirectory(workspaceRoot.value);
    treeData.value = toExplorerTree(nodes);
  } catch (error) {
    message.error(String(error));
  } finally {
    loadingTree.value = false;
  }
}

async function ensureCanSwitchFromActiveTab(): Promise<boolean> {
  if (!activeFile.value?.dirty) {
    return true;
  }
  return confirmDiscardUnsaved(activeFile.value.name);
}

const encryptedFlow = useEncryptedFileFlow({
  tabByPath,
  activePath,
  editorContent,
  refreshTree,
});

async function openFileByPath(path: string): Promise<boolean> {
  if (activePath.value === path) {
    return true;
  }

  if (!(await ensureCanSwitchFromActiveTab())) {
    return false;
  }

  const existing = tabByPath.value.get(path);
  if (existing) {
    activePath.value = path;
    syncEditorFromActiveTab();
    return true;
  }

  try {
    const inspect = await inspectFile(path);
    if (!inspect.editable) {
      message.error("该文件无法在编辑器中打开");
      return false;
    }

    const resolved = inspect.encrypted
      ? await encryptedFlow.resolveEncryptedContent(path, inspect)
      : { path, content: await readFile(path) };
    if (!resolved) {
      return false;
    }

    const nextInspect =
      resolved.path === path ? inspect : await inspectFile(resolved.path);
    const tab = createOpenedEditorFile(resolved.path, nextInspect, resolved.content);
    tabByPath.value = new Map(tabByPath.value).set(resolved.path, tab);
    activePath.value = resolved.path;
    editorContent.value = resolved.content;
    return true;
  } catch (error) {
    message.error(String(error));
    return false;
  }
};

async function reopenAfterConvert(nextPath: string | null) {
  if (nextPath) {
    await openFileByPath(nextPath);
  }
}

async function convertFileToEncrypted(path: string) {
  await reopenAfterConvert(await encryptedFlow.convertToEncrypted(path));
}

async function convertFileToCustomEncrypted(path: string) {
  await reopenAfterConvert(await encryptedFlow.convertToCustomEncrypted(path));
}

async function convertCustomToDefaultEncrypted(path: string) {
  await reopenAfterConvert(await encryptedFlow.convertToDefaultEncrypted(path));
}

async function convertFileToPlain(path: string) {
  await reopenAfterConvert(await encryptedFlow.convertToPlain(path));
}

async function openWorkspace(path: string) {
  workspaceRoot.value = path;
  rememberWorkspaceRoot(path);
  rememberRecentFolder(path);
  activePath.value = null;
  tabByPath.value = new Map();
  editorContent.value = "";
  await refreshTree();
}

async function openFolder() {
  const picked = await pickFolder();
  if (!picked) {
    return;
  }
  await openWorkspace(picked);
}

async function openSingleFile() {
  const picked = await pickFile();
  if (!picked) {
    return;
  }
  const opened = await openFileByPath(picked);
  if (opened) {
    rememberRecentFile(picked);
  }
}

async function isDirectoryPath(path: string): Promise<boolean> {
  try {
    await listDirectory(path);
    return true;
  } catch {
    return false;
  }
}

async function handleDroppedPaths(paths: string[]) {
  if (!paths.length) {
    return;
  }

  const uniquePaths = uniq(paths.map((item) => item.trim()).filter(Boolean));
  if (!uniquePaths.length) {
    return;
  }

  const [firstPath] = uniquePaths;
  if (!firstPath) {
    return;
  }

  if (await isDirectoryPath(firstPath)) {
    await openWorkspace(firstPath);
    return;
  }

  let openedCount = 0;
  for (const path of uniquePaths) {
    const opened = await openFileByPath(path);
    if (!opened) {
      continue;
    }
    rememberRecentFile(path);
    openedCount += 1;
  }

  if (openedCount > 1) {
    message.success(`已打开 ${openedCount} 个文件`);
  }
}

async function selectTab(path: string) {
  await openFileByPath(path);
}

async function closeTab(path: string) {
  const tab = tabByPath.value.get(path);
  if (!tab) {
    return;
  }

  const wasActive = activePath.value === path;
  if (tab.dirty) {
    if (wasActive) {
      updateActiveTabContent(editorContent.value, true);
    }
    const canClose = await confirmDiscardUnsaved(tab.name);
    if (!canClose) {
      return;
    }
  }

  removeTab(path, wasActive);
}

function removeTab(path: string, wasActive = activePath.value === path) {
  const tabPaths = [...tabByPath.value.keys()];
  const closedIndex = tabPaths.findIndex((item) => item === path);
  const nextTabs = new Map(tabByPath.value);
  nextTabs.delete(path);
  tabByPath.value = nextTabs;

  if (!wasActive) {
    return;
  }

  const remaining = [...nextTabs.keys()];
  if (remaining.length === 0) {
    activePath.value = null;
    editorContent.value = "";
    return;
  }

  const nextPath = remaining[Math.min(closedIndex, remaining.length - 1)] ?? remaining[0];
  activePath.value = nextPath;
  syncEditorFromActiveTab();
}

async function onExplorerCreateFile(directory: string, fileName: string) {
  try {
    const path = await createFile(directory, {
      fileName,
      encrypted: isEncryptedFileName(fileName),
    });
    await refreshTree();
    await openFileByPath(path);
  } catch (error) {
    message.error(String(error));
  }
}

async function onExplorerCreateFolder(directory: string, folderName: string) {
  try {
    await createDirectory(directory, folderName);
    await refreshTree();
  } catch (error) {
    message.error(String(error));
  }
}

async function onExplorerRename(path: string, newName: string) {
  try {
    const newPath = await renamePath(path, newName);
    const tab = tabByPath.value.get(path);
    if (tab) {
      const nextTabs = new Map(tabByPath.value);
      nextTabs.delete(path);
      nextTabs.set(newPath, { ...tab, path: newPath, name: newName });
      tabByPath.value = nextTabs;
      if (activePath.value === path) {
        activePath.value = newPath;
      }
    }
    await refreshTree();
  } catch (error) {
    message.error(String(error));
  }
}

function confirmDelete(dirty: boolean, name: string) {
  return new Promise<boolean>((resolve) => {
    Modal.confirm({
      title: dirty ? "删除未保存文件" : "删除确认",
      content: dirty
        ? `「${name}」有未保存更改，删除后将丢失。确定删除？`
        : `确定删除「${name}」？此操作不可撤销。`,
      okText: "删除",
      okType: "danger",
      cancelText: "取消",
      onOk: () => resolve(true),
      onCancel: () => resolve(false),
    });
  });
}

async function onExplorerDelete(path: string) {
  const tab = tabByPath.value.get(path);
  const name = path.split(/[/\\]/).pop() ?? path;
  const confirmed = await confirmDelete(Boolean(tab?.dirty), name);
  if (!confirmed) {
    return;
  }

  try {
    await deletePath(path);
    if (tab) {
      removeTab(path);
    }
    await refreshTree();
    message.success("已删除");
  } catch (error) {
    message.error(String(error));
  }
}

async function saveActiveFile() {
  if (!activeFile.value || !activePath.value) {
    return;
  }

  saving.value = true;
  try {
    const saved = await encryptedFlow.saveWithPassphraseFallback(
      activePath.value,
      editorContent.value,
      activeFile.value,
    );
    if (saved) {
      updateActiveTabContent(editorContent.value, false);
    }
  } catch (error) {
    message.error(String(error));
  } finally {
    saving.value = false;
  }
}

function onEditorChange(value: string) {
  editorContent.value = value;
  updateActiveTabContent(value);
}

function onKeydown(event: KeyboardEvent) {
  const isSave = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "s";
  if (!isSave) {
    return;
  }
  event.preventDefault();
  void saveActiveFile();
}

onMounted(async () => {
  window.addEventListener("keydown", onKeydown);
  try {
    unlistenDragDrop = await getCurrentWebviewWindow().onDragDropEvent((event) => {
      if (event.payload.type !== "drop") {
        return;
      }
      void handleDroppedPaths(event.payload.paths);
    });
  } catch (error) {
    console.warn("register drag drop listener failed", error);
  }
  if (savedWorkspaceRoot.value) {
    await openWorkspace(savedWorkspaceRoot.value);
  }
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  unlistenDragDrop?.();
  unlistenDragDrop = null;
});

watch(editorContent, (value) => {
  if (!activePath.value) {
    return;
  }
  const tab = tabByPath.value.get(activePath.value);
  if (!tab) {
    return;
  }
  if (value !== tab.content && !tab.dirty) {
    updateActiveTabContent(value, true);
  }
});
</script>

<template>
  <div class="encrypted-workspace">
    <header class="encrypted-workspace__toolbar">
      <a-space wrap>
        <OpenWithRecent
          label="打开文件夹"
          icon="mdi:folder-open-outline"
          :recent-paths="recentFolders"
          @primary="openFolder"
          @select-recent="openWorkspace"
          @remove-recent="removeRecentFolder"
        />
        <OpenWithRecent
          label="打开文件"
          icon="mdi:file-document-outline"
          :recent-paths="recentFiles"
          @primary="openSingleFile"
          @select-recent="openFileByPath"
          @remove-recent="removeRecentFile"
        />
      </a-space>
      <a-typography-text
        v-if="statusText"
        type="secondary"
        class="encrypted-workspace__status"
      >
        {{ statusText }}
      </a-typography-text>
    </header>

    <div class="encrypted-workspace__body">
      <aside class="encrypted-workspace__sidebar">
        <FileExplorer
          :tree-data="treeData"
          :workspace-root="workspaceRoot"
          :active-path="activePath"
          :loading="loadingTree"
          @open="openFileByPath"
          @refresh="refreshTree"
          @create-file="onExplorerCreateFile"
          @create-folder="onExplorerCreateFolder"
          @rename="onExplorerRename"
          @delete="onExplorerDelete"
          @convert-to-encrypted="convertFileToEncrypted"
          @convert-to-custom-encrypted="convertFileToCustomEncrypted"
          @convert-custom-to-default-encrypted="convertCustomToDefaultEncrypted"
          @convert-to-plain="convertFileToPlain"
        />
      </aside>

      <main class="encrypted-workspace__editor">
        <EditorTabs
          :tabs="openTabs"
          :active-path="activePath"
          @select="selectTab"
          @close="closeTab"
        />

        <div class="encrypted-workspace__editor-main">
          <MonacoEditor
            v-if="hasActiveFile"
            :model-value="editorContent"
            :language="activeLanguage"
            @update:model-value="onEditorChange"
          />
          <div v-else class="encrypted-workspace__placeholder" />
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped lang="scss">
.encrypted-workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f8fafc;
}

.encrypted-workspace__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
  background: #fff;
}

.encrypted-workspace__status {
  font-size: 12px;
  max-width: 50%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.encrypted-workspace__body {
  display: grid;
  grid-template-columns: 280px 1fr;
  min-height: 0;
  flex: 1;
}

.encrypted-workspace__sidebar {
  border-right: 1px solid #e5e7eb;
  background: #fff;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.encrypted-workspace__editor {
  min-height: 0;
  display: flex;
  flex-direction: column;
  background: #fff;
}

.encrypted-workspace__editor-main {
  flex: 1;
  min-height: 0;
  padding: 12px;
  display: flex;
  flex-direction: column;
  background: #f8fafc;

  :deep(.monaco-editor-host),
  .encrypted-workspace__placeholder {
    flex: 1;
    min-height: 0;
  }
}

.encrypted-workspace__placeholder {
  flex: 1;
  min-height: 0;
  background: #f8fafc;
}
</style>
