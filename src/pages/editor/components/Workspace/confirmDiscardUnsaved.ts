import { Modal } from "ant-design-vue";

export async function confirmDiscardUnsaved(fileName: string): Promise<boolean> {
  return new Promise((resolve) => {
    Modal.confirm({
      title: "未保存的更改",
      content: `「${fileName}」尚未保存，关闭后修改将无法恢复。`,
      okText: "放弃更改",
      okType: "danger",
      cancelText: "继续编辑",
      onOk: () => resolve(true),
      onCancel: () => resolve(false),
    });
  });
}

/** 批量关闭含未保存标签时的确认 */
export async function confirmCloseUnsavedTabs(): Promise<boolean> {
  return new Promise((resolve) => {
    Modal.confirm({
      title: "关闭未保存的文件？",
      content: "有文件尚未保存，关闭后修改将丢失。",
      okText: "仍要关闭",
      okType: "danger",
      cancelText: "取消",
      onOk: () => resolve(true),
      onCancel: () => resolve(false),
    });
  });
}
