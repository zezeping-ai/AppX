import { Crepe, type CrepeConfig } from "@milkdown/crepe";
import {
  commandsCtx,
  schemaCtx,
  type Editor,
} from "@milkdown/kit/core";
import type { Ctx } from "@milkdown/kit/ctx";
import { isNodeSelectedCommand } from "@milkdown/kit/preset/commonmark";
import { secretToolbarIcon } from "./icons";
import {
  insertSecretFromSlash,
  runToggleSecret,
  SECRET_NODE_ID,
  secretPlugins,
} from "./schema";

type CrepeFeatureConfigs = NonNullable<CrepeConfig["featureConfigs"]>;

/** Crepe.addFeature：注册密文 schema / remark / view / command */
export function secretFeature(editor: Editor) {
  editor.use(secretPlugins);
}

function isSecretNodeActive(ctx: Ctx): boolean {
  const nodeType = ctx.get(schemaCtx).nodes[SECRET_NODE_ID];
  if (!nodeType) {
    return false;
  }
  return Boolean(
    ctx.get(commandsCtx).call(isNodeSelectedCommand.key, nodeType),
  );
}

/** 挂到 Crepe Toolbar / Slash（仅 MarkdownEditor 使用） */
export const secretCrepeFeatureConfigs: CrepeFeatureConfigs = {
  [Crepe.Feature.Toolbar]: {
    buildToolbar: (builder) => {
      builder.getGroup("function").addItem("secret", {
        icon: secretToolbarIcon,
        // 文字选区：设为密文；若已是密文 NodeSelection（少见，Crepe 条通常不出现）则取消
        label: "密文",
        active: isSecretNodeActive,
        onRun: runToggleSecret,
      });
    },
  },
  [Crepe.Feature.BlockEdit]: {
    buildMenu: (builder) => {
      builder.addGroup("secret", "密文").addItem("secret", {
        label: "密文",
        icon: secretToolbarIcon,
        onRun: insertSecretFromSlash,
      });
    },
  },
};
