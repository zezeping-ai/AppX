<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { Icon } from "@iconify/vue";
import { useDebounceFn } from "@vueuse/core";
import {
  EditorPaletteOverlay,
  resolvePaletteFileIcon,
  segmentsFromRange,
} from "@/pages/editor/components/EditorPalette";
import { usePaletteEscape } from "@/hooks/usePaletteEscape";
import { usePaletteListNav } from "@/hooks/usePaletteListNav";
import { searchWorkspace } from "@/modules/editor/client";
import type {
  SearchInFilesSelectPayload,
  WorkspaceSearchFile,
  WorkspaceSearchMatch,
  WorkspaceSearchResult,
} from "@/modules/editor/types";

const props = defineProps<{
  open: boolean;
  workspaceRoot: string | null;
}>();

const emit = defineEmits<{
  "update:open": [value: boolean];
  select: [payload: SearchInFilesSelectPayload];
}>();

const keyword = ref("");
const caseSensitive = ref(false);
const searching = ref(false);
const result = ref<WorkspaceSearchResult | null>(null);
const errorText = ref("");
const expanded = ref<Set<string>>(new Set());
const activeKey = ref<string | null>(null);
const searchInputRef = ref<{ focus: () => void } | null>(null);
const listRef = ref<HTMLElement | null>(null);

let searchGeneration = 0;

type FlatRow =
  | { kind: "file"; file: WorkspaceSearchFile; key: string; icon: ReturnType<typeof resolvePaletteFileIcon> }
  | {
      kind: "match";
      file: WorkspaceSearchFile;
      match: WorkspaceSearchMatch;
      matchIndex: number;
      key: string;
    };

const flatRows = computed(() => {
  const files = result.value?.files ?? [];
  const rows: FlatRow[] = [];
  for (const file of files) {
    rows.push({
      kind: "file",
      file,
      key: `f:${file.path}`,
      icon: resolvePaletteFileIcon(file),
    });
    if (!expanded.value.has(file.path)) {
      continue;
    }
    file.matches.forEach((match, matchIndex) => {
      rows.push({
        kind: "match",
        file,
        match,
        matchIndex,
        key: `m:${file.path}:${match.line}:${match.column}:${matchIndex}`,
      });
    });
  }
  return rows;
});

const activeIndex = ref(0);

watch(flatRows, (rows) => {
  if (!rows.length) {
    activeIndex.value = 0;
    return;
  }
  const idx = rows.findIndex((row) => row.key === activeKey.value);
  activeIndex.value = idx >= 0 ? idx : 0;
  if (idx < 0) {
    activeKey.value = rows[0]?.key ?? null;
  }
});

watch(activeIndex, (idx) => {
  const row = flatRows.value[idx];
  if (row) {
    activeKey.value = row.key;
  }
});

const statusText = computed(() => {
  if (searching.value) {
    return "搜索中…";
  }
  if (errorText.value) {
    return errorText.value;
  }
  if (!result.value || !keyword.value.trim()) {
    return "";
  }
  const parts = [
    `${result.value.matchCount} 处匹配`,
    `${result.value.files.length} 个文件`,
  ];
  if (result.value.truncated) {
    parts.push("结果已截断");
  }
  if (result.value.skippedLocked > 0) {
    parts.push(`跳过 ${result.value.skippedLocked} 个未解锁加密文件`);
  }
  return parts.join(" · ");
});

watch(
  () => props.open,
  (open) => {
    if (!open) {
      return;
    }
    focusSearch();
    if (keyword.value.trim()) {
      void runSearch();
    }
  },
);

const debouncedSearch = useDebounceFn(() => {
  if (!props.open) {
    return;
  }
  void runSearch();
}, 220);

watch([keyword, caseSensitive], () => {
  if (!props.open) {
    return;
  }
  void debouncedSearch();
});

usePaletteEscape(() => props.open, close);

const { onPanelKeydown } = usePaletteListNav({
  listRef,
  activeIndex,
  length: () => flatRows.value.length,
  onEnter: activateCurrent,
  keyAt: (index) => flatRows.value[index]?.key ?? null,
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

async function runSearch() {
  const root = props.workspaceRoot?.trim() || null;
  const q = keyword.value.trim();
  if (!root) {
    result.value = null;
    errorText.value = "请先打开工作文件夹";
    return;
  }
  if (!q) {
    result.value = null;
    errorText.value = "";
    expanded.value = new Set();
    activeKey.value = null;
    return;
  }

  const generation = ++searchGeneration;
  searching.value = true;
  errorText.value = "";
  try {
    const next = await searchWorkspace(root, q, {
      caseSensitive: caseSensitive.value,
    });
    if (generation !== searchGeneration) {
      return;
    }
    result.value = next;
    expanded.value = new Set(next.files.map((f) => f.path));
    const first = next.files[0];
    activeKey.value = first
      ? `m:${first.path}:${first.matches[0]?.line}:${first.matches[0]?.column}:0`
      : null;
  } catch (error) {
    if (generation !== searchGeneration) {
      return;
    }
    result.value = null;
    errorText.value = String(error);
  } finally {
    if (generation === searchGeneration) {
      searching.value = false;
    }
  }
}

function toggleFile(path: string) {
  const next = new Set(expanded.value);
  if (next.has(path)) {
    next.delete(path);
  } else {
    next.add(path);
  }
  expanded.value = next;
}

function selectMatch(
  file: WorkspaceSearchFile,
  match: WorkspaceSearchMatch,
  matchIndex: number,
) {
  activeKey.value = `m:${file.path}:${match.line}:${match.column}:${matchIndex}`;
  emit("select", {
    path: file.path,
    line: match.line,
    column: match.column,
    language: file.language,
  });
  close();
}

function selectRow(row: FlatRow) {
  if (row.kind === "file") {
    const first = row.file.matches[0];
    if (first) {
      selectMatch(row.file, first, 0);
    }
    return;
  }
  selectMatch(row.file, row.match, row.matchIndex);
}

function onChevronClick(event: MouseEvent, path: string) {
  event.stopPropagation();
  toggleFile(path);
}

function activateCurrent() {
  const row = flatRows.value[activeIndex.value];
  if (!row) {
    return;
  }
  if (row.kind === "file") {
    const first = row.file.matches[0];
    if (first) {
      selectMatch(row.file, first, 0);
    }
    return;
  }
  selectMatch(row.file, row.match, row.matchIndex);
}

function setActiveRow(row: FlatRow, index: number) {
  activeKey.value = row.key;
  activeIndex.value = index;
}
</script>

<template>
  <EditorPaletteOverlay
    :open="open"
    dialog-label="在文件中搜索"
    variant="search"
    search-icon="mdi:file-find-outline"
    @close="close"
    @keydown="onPanelKeydown"
  >
    <template #input>
      <a-input
        ref="searchInputRef"
        v-model:value="keyword"
        allow-clear
        placeholder="在工作区中搜索…"
        size="large"
        :bordered="false"
        @keydown="onPanelKeydown"
      />
    </template>
    <template #searchExtras>
      <button
        type="button"
        class="search-in-files__case"
        :class="{ 'search-in-files__case--on': caseSensitive }"
        title="区分大小写"
        @click="caseSensitive = !caseSensitive"
      >
        Aa
      </button>
    </template>

    <template v-if="statusText" #status>
      <div class="search-in-files__status">{{ statusText }}</div>
    </template>

    <div v-if="!keyword.trim()" class="editor-palette__empty">输入关键词开始搜索</div>
    <div v-else-if="searching && !result" class="editor-palette__empty">搜索中…</div>
    <div v-else-if="!result?.files.length && !searching" class="editor-palette__empty">
      没有匹配结果
    </div>
    <ul v-else ref="listRef" class="editor-palette__list">
      <li
        v-for="(row, index) in flatRows"
        :key="row.key"
        :data-key="row.key"
        class="editor-palette__row search-in-files__row"
        :class="{
          'editor-palette__row--active': row.key === activeKey,
          'search-in-files__row--match': row.kind === 'match',
        }"
        @mouseenter="setActiveRow(row, index)"
        @click="selectRow(row)"
      >
        <template v-if="row.kind === 'file'">
          <button
            type="button"
            class="search-in-files__chevron-btn"
            :aria-label="expanded.has(row.file.path) ? '折叠' : '展开'"
            @click="onChevronClick($event, row.file.path)"
          >
            <Icon
              :icon="expanded.has(row.file.path) ? 'mdi:chevron-down' : 'mdi:chevron-right'"
              class="search-in-files__chevron"
              aria-hidden="true"
            />
          </button>
          <Icon
            :icon="row.icon.icon"
            class="editor-palette__file-icon search-in-files__file-icon"
            :style="{ color: row.icon.color }"
            aria-hidden="true"
          />
          <div class="search-in-files__file-meta">
            <span class="search-in-files__file-name">{{ row.file.name }}</span>
            <span class="search-in-files__file-path">{{ row.file.relativePath }}</span>
          </div>
          <span class="search-in-files__count">{{ row.file.matches.length }}</span>
        </template>
        <template v-else>
          <span class="search-in-files__line">{{ row.match.line }}</span>
          <div class="search-in-files__preview">
            <template
              v-for="(seg, si) in segmentsFromRange(
                row.match.preview,
                row.match.matchStart,
                row.match.matchEnd,
              )"
              :key="`${row.key}-s-${si}`"
            >
              <mark v-if="seg.hit" class="editor-palette__mark editor-palette__mark--preview">
                {{ seg.text }}
              </mark>
              <template v-else>{{ seg.text }}</template>
            </template>
          </div>
        </template>
      </li>
    </ul>
  </EditorPaletteOverlay>
</template>

<style scoped lang="scss">
.search-in-files__case {
  flex-shrink: 0;
  border: 1px solid var(--app-border);
  background: transparent;
  color: var(--app-fg-secondary);
  border-radius: 6px;
  padding: 4px 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  line-height: 1;
}

.search-in-files__case--on {
  background: var(--app-active-bg, rgba(59, 130, 246, 0.14));
  color: var(--app-active-fg, #2563eb);
  border-color: transparent;
}

.search-in-files__status {
  padding: 6px 14px;
  font-size: 12px;
  color: var(--app-fg-secondary);
  border-bottom: 1px solid var(--app-border);
}

.search-in-files__row--match {
  padding-left: 36px;
  gap: 8px;
}

.search-in-files__chevron-btn {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: none;
  background: transparent;
  cursor: pointer;
  color: inherit;
}

.search-in-files__chevron {
  font-size: 16px;
  color: var(--app-fg-secondary);
}

.search-in-files__file-icon {
  font-size: 16px;
}

.search-in-files__file-meta {
  min-width: 0;
  flex: 1;
  display: flex;
  gap: 8px;
  align-items: baseline;
}

.search-in-files__file-name {
  font-size: 13px;
  font-weight: 500;
  flex-shrink: 0;
}

.search-in-files__file-path {
  font-size: 12px;
  color: var(--app-fg-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.search-in-files__count {
  flex-shrink: 0;
  font-size: 11px;
  color: var(--app-fg-secondary);
}

.search-in-files__line {
  flex-shrink: 0;
  width: 36px;
  text-align: right;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  color: var(--app-fg-muted, var(--app-fg-secondary));
}

.search-in-files__preview {
  min-width: 0;
  flex: 1;
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
