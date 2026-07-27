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
