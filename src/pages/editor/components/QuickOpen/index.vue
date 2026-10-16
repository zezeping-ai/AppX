<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { Icon } from "@/components/Icon";
import {
  EditorPaletteOverlay,
  resolvePaletteFileIcon,
  segmentsFromMarks,
} from "@/pages/editor/components/EditorPalette";
import { usePaletteEscape } from "@/hooks/usePaletteEscape";
import { usePaletteListNav } from "@/hooks/usePaletteListNav";
import { listWorkspaceFiles } from "@/modules/editor/client";
import type { EditorFileEntry } from "@/modules/editor/types";
import { filterFilesByFuzzy } from "./fuzzyMatch";

const props = defineProps<{
  open: boolean;
  workspaceRoot: string | null;
  /** 变更时强制刷新文件索引（树刷新 / 增删改后） */
  indexVersion?: number;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  select: [path: string];
}>();

const keyword = ref("");
const loading = ref(false);
const files = ref<EditorFileEntry[]>([]);
const activeIndex = ref(0);
const errorText = ref("");
const searchInputRef = ref<{ focus: () => void } | null>(null);
const listRef = ref<HTMLElement | null>(null);

let cachedRoot: string | null = null;
let cachedVersion = -1;
let loadGeneration = 0;

const filtered = computed(() =>
  filterFilesByFuzzy(files.value, keyword.value, 80).map((item) => ({
    ...item,
    icon: resolvePaletteFileIcon(item),
  })),
);

watch(
  () => [filtered.value, keyword.value] as const,
  () => {
    activeIndex.value = 0;
  },
);

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }
    keyword.value = "";
    activeIndex.value = 0;
    errorText.value = "";
    void ensureIndex().then(() => focusSearch());
  },
);

usePaletteEscape(() => props.open, close);

const { onPanelKeydown } = usePaletteListNav({
  listRef,
  activeIndex,
  length: () => filtered.value.length,
  onEnter: () => selectAt(activeIndex.value),
});

function close() {
  emit("update:open", false);
}

function focusSearch() {
  void nextTick(() => {
    requestAnimationFrame(() => {
      searchInputRef.value?.focus();
    });
  });
}

async function ensureIndex() {
  const root = props.workspaceRoot?.trim() || null;
  if (!root) {
    files.value = [];
    errorText.value = "请先打开工作文件夹";
    return;
  }

  const version = props.indexVersion ?? 0;
  if (cachedRoot === root && cachedVersion === version && files.value.length) {
    return;
  }

  const generation = ++loadGeneration;
  loading.value = true;
  errorText.value = "";
  try {
    const next = await listWorkspaceFiles(root);
    if (generation !== loadGeneration) {
      return;
    }
    files.value = next;
    cachedRoot = root;
    cachedVersion = version;
  } catch (error) {
    if (generation !== loadGeneration) {
      return;
    }
    files.value = [];
    errorText.value = String(error);
  } finally {
    if (generation === loadGeneration) {
      loading.value = false;
    }
  }
}

function selectAt(index: number) {
  const item = filtered.value[index];
  if (!item) {
    return;
  }
  emit("select", item.path);
  close();
}
</script>

<template>
  <EditorPaletteOverlay
    :open="open"
    dialog-label="快速打开文件"
    variant="quick"
    search-icon="mdi:file-search-outline"
    @close="close"
    @keydown="onPanelKeydown"
  >
    <template #input>
      <a-input
        ref="searchInputRef"
        v-model:value="keyword"
        allow-clear
        placeholder="按文件名搜索…"
        size="large"
        :bordered="false"
        @keydown="onPanelKeydown"
      />
    </template>

    <div v-if="loading" class="editor-palette__empty">索引文件中…</div>
    <div v-else-if="errorText" class="editor-palette__empty">{{ errorText }}</div>
    <div v-else-if="!filtered.length" class="editor-palette__empty">没有匹配的文件</div>
    <ul v-else ref="listRef" class="editor-palette__list">
      <li
        v-for="(item, index) in filtered"
        :key="item.path"
        :data-index="index"
        class="editor-palette__row"
        :class="{ 'editor-palette__row--active': index === activeIndex }"
        @mouseenter="activeIndex = index"
        @click="selectAt(index)"
      >
        <Icon
          :icon="item.icon.icon"
          class="editor-palette__file-icon"
          :style="{ color: item.icon.color }"
        />
        <div class="quick-open__meta">
          <div class="quick-open__name">
            <template
              v-for="(seg, si) in segmentsFromMarks(item.name, item.match.nameMatches)"
              :key="`n-${si}`"
            >
              <mark v-if="seg.hit" class="editor-palette__mark">{{ seg.text }}</mark>
              <template v-else>{{ seg.text }}</template>
            </template>
          </div>
          <div class="quick-open__path">
            <template
              v-for="(seg, si) in segmentsFromMarks(item.relativePath, item.match.pathMatches)"
              :key="`p-${si}`"
            >
              <mark v-if="seg.hit" class="editor-palette__mark">{{ seg.text }}</mark>
              <template v-else>{{ seg.text }}</template>
            </template>
          </div>
        </div>
      </li>
    </ul>

    <template #footer>
      <span>↑↓ 选择</span>
      <span>Enter 打开</span>
      <span>Esc 关闭</span>
      <span v-if="files.length" class="editor-palette__footer-count">{{ files.length }} 个文件</span>
    </template>
  </EditorPaletteOverlay>
</template>

<style scoped lang="scss">
.quick-open__meta {
  min-width: 0;
  flex: 1;
}

.quick-open__name {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.quick-open__path {
  margin-top: 2px;
  font-size: 12px;
  color: var(--app-fg-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
