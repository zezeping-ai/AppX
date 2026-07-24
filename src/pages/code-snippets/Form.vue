<script setup lang="ts">
import { computed, watch } from "vue";
import { message } from "ant-design-vue";
import MonacoEditor from "@/components/MonacoEditor/index.vue";
import ShortcutRecorder from "@/components/ShortcutRecorder/index.vue";
import { useForm } from "@/hooks";
import { CodeSnippetRecord } from "@/models";
import {
  CODE_SNIPPET_GROUPS,
  DEFAULT_CODE_SNIPPET_GROUP,
  syncAllSnippetsToRuntime,
  formatAbbreviationTrigger,
  inlineExpansionTrigger,
  normalizeAbbreviationInput,
  validateAbbreviation,
  assertSnippetShortcutAvailable,
  type CodeSnippetGroup,
} from "@/modules/codeSnippets";
import { encryptText } from "@/modules/crypto";
import { getErrorMessage } from "@/shared/error";
import { normalizeGlobalShortcut } from "@/shared/shortcut";
import { setGlobalShortcutsPaused } from "@/modules/globalShortcut";

const props = defineProps<{
  record: CodeSnippetRecord | null;
  plainContent: string;
}>();

const emit = defineEmits<{ submitted: [] }>();

const abbreviationTrigger = computed(() =>
  formatAbbreviationTrigger(form.model.abbreviation, inlineExpansionTrigger.value),
);

function onAbbreviationInput(value: string) {
  form.model.abbreviation = normalizeAbbreviationInput(value);
}

async function assertUniqueAbbreviation(abbreviation: string, excludeId?: number) {
  const existing = await CodeSnippetRecord.findBy({ abbreviation });
  if (existing && existing.id !== excludeId) {
    throw new Error(`缩写「${abbreviation}」已存在`);
  }
}

const form = useForm({
  model: {
    name: props.record?.name ?? "",
    group: (props.record?.meta.group as CodeSnippetGroup | undefined) ?? DEFAULT_CODE_SNIPPET_GROUP,
    abbreviation: props.record?.abbreviation ?? "",
    shortcut: props.record?.shortcut ?? "",
    language: props.record?.meta.language ?? "plaintext",
    content: props.plainContent,
    note: props.record?.meta.note ?? "",
  },
  rules: {
    name: [{ required: true, message: "请填写代码段名称" }],
    abbreviation: [
      { required: true, message: "请填写缩写" },
      {
        validator: async (_rule, value: string) => {
          const err = validateAbbreviation(String(value ?? ""));
          if (err) throw new Error(err);
        },
      },
      {
        validator: async (_rule, value: string) => {
          const abbreviation = String(value ?? "")
            .trim()
            .toLowerCase();
          if (!abbreviation) return;
          await assertUniqueAbbreviation(abbreviation, props.record?.id as number | undefined);
        },
      },
    ],
    shortcut: [
      {
        validator: async (_rule, value: string) => {
          const err = await assertSnippetShortcutAvailable(
            value,
            props.record?.id as number | undefined,
          );
          if (err) throw new Error(err);
        },
      },
    ],
    content: [{ required: true, message: "请填写内容" }],
  },
  async onSubmit() {
    const name = form.model.name.trim();
    const abbreviation = form.model.abbreviation.trim().toLowerCase();
    const content = form.model.content;
    const editingId = props.record?.id as number | undefined;

    try {
      await assertUniqueAbbreviation(abbreviation, editingId);
      const shortcut = normalizeGlobalShortcut(form.model.shortcut);
      const shortcutErr = await assertSnippetShortcutAvailable(shortcut, editingId);
      if (shortcutErr) throw new Error(shortcutErr);

      const encryptedContent = await encryptText(content);
      const payload = {
        name,
        abbreviation,
        shortcut,
        content: encryptedContent,
        meta: {
          group: form.model.group,
          language: form.model.language.trim() || null,
          note: form.model.note.trim(),
          order: props.record?.meta.order ?? 0,
        },
      };

      if (editingId) {
        await CodeSnippetRecord.update(editingId, payload);
      } else {
        await CodeSnippetRecord.create(payload);
      }
      await syncAllSnippetsToRuntime();
      message.success(editingId ? "已更新" : "已创建");
      emit("submitted");
    } catch (error) {
      message.error(getErrorMessage(error, "保存失败"));
      throw error;
    }
  },
});

// 录制变更后立刻校验占用
watch(
  () => form.model.shortcut,
  async (shortcut) => {
    if (!shortcut) return;
    const err = await assertSnippetShortcutAvailable(
      shortcut,
      props.record?.id as number | undefined,
    );
    if (err) {
      message.warning(err);
    }
    void form.formRef?.validateFields(["shortcut"]);
  },
);
</script>

<template>
  <div class="snippet-form">
    <a-form
      :ref="form.setFormRef"
      :model="form.model"
      :rules="form.rules"
      layout="vertical"
    >
      <a-row :gutter="12">
        <a-col :span="12">
          <a-form-item name="name" label="代码段名称">
            <a-input v-model:value="form.model.name" placeholder="便于识别的名称" />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item label="分组">
            <a-select v-model:value="form.model.group">
              <a-select-option
                v-for="item in CODE_SNIPPET_GROUPS"
                :key="item.value"
                :value="item.value"
              >
                {{ item.label }}
              </a-select-option>
            </a-select>
          </a-form-item>
        </a-col>
      </a-row>

      <a-row :gutter="12">
        <a-col :span="12">
          <a-form-item name="abbreviation">
            <template #label>
              <span class="snippet-form__label-row">
                <span>缩写</span>
                <span class="snippet-form__label-hint">使用时输入 {{ abbreviationTrigger }}</span>
              </span>
            </template>
            <a-input
              :value="form.model.abbreviation"
              placeholder="addr 或 my_snippet"
              @update:value="onAbbreviationInput"
            />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item name="shortcut" label="快捷键">
            <ShortcutRecorder
              v-model:value="form.model.shortcut"
              :on-recording-change="setGlobalShortcutsPaused"
            />
          </a-form-item>
        </a-col>
      </a-row>

      <a-form-item label="语法高亮">
        <a-select v-model:value="form.model.language">
          <a-select-option value="plaintext">Plain Text</a-select-option>
          <a-select-option value="javascript">JavaScript</a-select-option>
          <a-select-option value="typescript">TypeScript</a-select-option>
          <a-select-option value="html">HTML</a-select-option>
          <a-select-option value="css">CSS</a-select-option>
          <a-select-option value="json">JSON</a-select-option>
          <a-select-option value="sql">SQL</a-select-option>
          <a-select-option value="shell">Shell</a-select-option>
          <a-select-option value="markdown">Markdown</a-select-option>
        </a-select>
      </a-form-item>

      <a-form-item name="content" label="内容">
        <MonacoEditor
          v-model="form.model.content"
          :language="form.model.language"
          auto-height
          :min-height="160"
        />
      </a-form-item>

      <a-form-item label="备注" class="snippet-form__note">
        <a-textarea v-model:value="form.model.note" :rows="2" placeholder="可选说明" />
      </a-form-item>
    </a-form>

    <div class="snippet-form__footer">
      <a-button @click="form.close">取消</a-button>
      <a-button type="primary" :loading="form.submitLoading" @click="form.onSubmit">保存</a-button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.snippet-form {
  display: flex;
  flex-direction: column;
}

.snippet-form__label-row {
  display: inline-flex;
  align-items: baseline;
  gap: 6px;
}

.snippet-form__label-hint {
  font-size: 11px;
  font-weight: 400;
  color: #6b7280;
}

[data-theme="dark"] .snippet-form__label-hint {
  color: rgba(255, 255, 255, 0.45);
}

/* 左右抵消 drawer body 内边距，做成底部操作栏 */
.snippet-form__footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin: 0 -24px;
  padding: 12px 24px 20px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  background: #fff;
}

.snippet-form__note {
  margin-bottom: 0;
}

[data-theme="dark"] .snippet-form__footer {
  border-top-color: rgba(255, 255, 255, 0.08);
  background: #141414;
}
</style>
