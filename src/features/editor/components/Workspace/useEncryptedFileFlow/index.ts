import { message } from "ant-design-vue";
import type { Ref } from "vue";
import {
  convertCustomToDefaultEncryptedFile,
  convertToCustomEncryptedFile,
  convertToEncryptedFile,
  convertToPlainFile,
  inspectFile,
  readFile,
  unlockEncryptedFile,
  writeFile,
} from "@/modules/editor/client";
import {
  convertUnlockPromptFor,
  saveSuccessLabel,
  unlockPromptFor,
} from "@/modules/editor/encryption";
import type { FileInspect, OpenedEditorFile } from "@/modules/editor/types";
import { isDecryptPassphraseRequired } from "@/modules/editor/types";
import { promptPassphrase } from "../promptPassphrase";

interface EncryptedFileFlowDeps {
  tabByPath: Ref<Map<string, OpenedEditorFile>>;
  activePath: Ref<string | null>;
  editorContent: Ref<string>;
  refreshTree: () => Promise<void>;
}

export function useEncryptedFileFlow(deps: EncryptedFileFlowDeps) {
  function removeTabAt(path: string) {
    const nextTabs = new Map(deps.tabByPath.value);
    nextTabs.delete(path);
    deps.tabByPath.value = nextTabs;
  }

  async function flushDirtyTab(path: string) {
    const tab = deps.tabByPath.value.get(path);
    if (tab?.dirty && deps.activePath.value === path) {
      await writeFile(path, deps.editorContent.value);
    }
  }

  async function withUnlockRetry<T>(
    path: string,
    prompt: { title: string; content: string },
    action: () => Promise<T>,
  ): Promise<T | null> {
    try {
      return await action();
    } catch (error) {
      if (!isDecryptPassphraseRequired(error)) {
        throw error;
      }
      const passphrase = await promptPassphrase(prompt);
      if (!passphrase) {
        return null;
      }
      await unlockEncryptedFile(path, passphrase);
      return action();
    }
  }

  async function convertAndGetNextPath(
    path: string,
    convert: () => Promise<string>,
  ): Promise<string | null> {
    await flushDirtyTab(path);
    const nextPath = await convert();
    removeTabAt(path);
    await deps.refreshTree();
    return nextPath;
  }

  async function resolveEncryptedContent(
    path: string,
    inspect: FileInspect,
  ): Promise<{ path: string; content: string } | null> {
    try {
      return { path, content: await readFile(path) };
    } catch (error) {
      if (!isDecryptPassphraseRequired(error)) {
        throw error;
      }

      const passphrase = await promptPassphrase(unlockPromptFor(inspect));
      if (!passphrase) {
        return null;
      }

      const unlocked = await unlockEncryptedFile(path, passphrase);
      if (unlocked.path !== path) {
        await deps.refreshTree();
      }
      return { path: unlocked.path, content: unlocked.content };
    }
  }

  async function saveWithPassphraseFallback(
    path: string,
    content: string,
    file: OpenedEditorFile,
  ): Promise<boolean> {
    try {
      await writeFile(path, content);
      message.success(saveSuccessLabel(file));
      return true;
    } catch (error) {
      if (!isDecryptPassphraseRequired(error)) {
        throw error;
      }
      const passphrase = await promptPassphrase({
        title: "输入独立口令",
        content: "保存该 .x0 文件需要独立口令",
      });
      if (!passphrase) {
        return false;
      }
      await unlockEncryptedFile(path, passphrase);
      await writeFile(path, content);
      message.success(saveSuccessLabel({ ...file, customEncrypted: true }));
      return true;
    }
  }

  async function convertToDefaultEncrypted(path: string): Promise<string | null> {
    try {
      return await withUnlockRetry(
        path,
        convertUnlockPromptFor({ customEncrypted: true }, "default"),
        () =>
          convertAndGetNextPath(path, () => convertCustomToDefaultEncryptedFile(path)),
      );
    } catch (error) {
      message.error(String(error));
      return null;
    }
  }

  async function convertToPlain(path: string): Promise<string | null> {
    try {
      const inspect = await inspectFile(path);
      return await withUnlockRetry(path, convertUnlockPromptFor(inspect, "plain"), () =>
        convertAndGetNextPath(path, () => convertToPlainFile(path)),
      );
    } catch (error) {
      message.error(String(error));
      return null;
    }
  }

  async function convertToEncrypted(path: string): Promise<string | null> {
    try {
      return await convertAndGetNextPath(path, () => convertToEncryptedFile(path));
    } catch (error) {
      message.error(String(error));
      return null;
    }
  }

  async function convertToCustomEncrypted(path: string): Promise<string | null> {
    const passphrase = await promptPassphrase({
      title: "设置独立口令",
      content: "文件将加密为 .x0 格式，口令仅在本次运行期间记住",
    });
    if (!passphrase) {
      return null;
    }
    try {
      return await convertAndGetNextPath(path, () =>
        convertToCustomEncryptedFile(path, passphrase),
      );
    } catch (error) {
      message.error(String(error));
      return null;
    }
  }

  return {
    resolveEncryptedContent,
    saveWithPassphraseFallback,
    convertToEncrypted,
    convertToCustomEncrypted,
    convertToDefaultEncrypted,
    convertToPlain,
  };
}
