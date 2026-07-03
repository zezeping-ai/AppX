import type { FormInstance } from "ant-design-vue/es/form";
import type { TablePaginationConfig } from "ant-design-vue/es/table/interface";
import type { VNodeRef } from "vue";
import { message } from "ant-design-vue";
import { has, isEmpty } from "lodash-es";
import { computed, nextTick, reactive, watch } from "vue";
import { getErrorMessage } from "@/shared/error";
import { trimQuery } from "@/shared/trimQuery";

const PAGINATION_DEFAULT_PAGE_SIZE = 20;

export interface ListQuery {
  page?: number;
  pageSize?: number;
  [key: string]: unknown;
}

export interface ListPagination {
  page: number;
  pageSize: number;
  total: number;
}

export interface ListOptions {
  onLoad: (query: ListQuery, shouldReset?: boolean) => Promise<void>;
  query?: ListQuery;
  pagination?: Partial<ListPagination>;
}

export interface ListState<T = unknown> {
  setListRef: (listRef: VNodeRef) => void;
  listRef: VNodeRef | null;
  setFormRef: (formRef: FormInstance | null) => void;
  formRef: FormInstance | null;
  fetchCount: number;
  isRefreshing: boolean;
  isLoading: boolean;
  isFinished: boolean;
  errorInfo: Error | null;
  query: ListQuery;
  records: T[];
  pagination: ListPagination;
  initState: () => void;
  onRefresh: (moreQuery?: ListQuery) => Promise<void>;
  onLoad: (moreQuery?: ListQuery, shouldReset?: boolean) => Promise<void>;
  onReset: (customQuery?: ListQuery) => Promise<void>;
}

export const useList = <T = unknown>(options: ListOptions): ListState<T> => {
  const buildInitialPagination = (): ListPagination => ({
    page: 1,
    pageSize: PAGINATION_DEFAULT_PAGE_SIZE,
    total: 0,
    ...options.pagination,
  });

  const handleError = (err: unknown) => {
    const e = err instanceof Error ? err : new Error(getErrorMessage(err, "获取数据失败"));
    message.error(getErrorMessage(e, "获取数据失败"));
    state.errorInfo = e;
    state.records = [] as T[];
  };

  const initState = () => {
    Object.assign(state, {
      fetchCount: 0,
      isRefreshing: false,
      isLoading: false,
      isFinished: false,
      errorInfo: null,
      query: { ...options.query },
      records: [] as T[],
      pagination: buildInitialPagination(),
    });
  };

  /** 请求参数统一由筛选条件 + 当前分页组成，避免在 state.query 中混杂 UI 分页字段。 */
  const buildLoadQuery = (): ListQuery =>
    trimQuery({
      ...state.query,
      page: state.pagination.page,
      pageSize: state.pagination.pageSize,
    }) as ListQuery;

  const state = reactive({
    setListRef(listRef: VNodeRef) {
      state.listRef = listRef;
    },
    listRef: null as VNodeRef | null,
    setFormRef(formRef: FormInstance | null) {
      state.formRef = formRef;
    },
    formRef: null as FormInstance | null,
    fetchCount: 0,
    isRefreshing: false,
    isLoading: false,
    isFinished: false,
    errorInfo: null as Error | null,
    query: { ...options.query },
    records: [] as T[],
    pagination: {
      page: 1,
      pageSize: PAGINATION_DEFAULT_PAGE_SIZE,
      total: 0,
      ...options.pagination,
    },
    initState,
    async onRefresh(moreQuery: ListQuery = {}) {
      state.records = [] as T[];
      state.isFinished = false;
      state.isRefreshing = true;
      await state.onLoad(moreQuery);
    },
    async onLoad(moreQuery: ListQuery = {}, shouldReset = false) {
      if (state.formRef) {
        try {
          await state.formRef.validate();
        } catch {
          return;
        }
      }

      if (moreQuery instanceof Event) moreQuery = {};

      const { page, pageSize, ...otherCustomQuery } = moreQuery;

      if (shouldReset) {
        state.query = { ...options.query, ...otherCustomQuery };
        state.pagination.page = 1;
        state.pagination.pageSize = (pageSize as number) || PAGINATION_DEFAULT_PAGE_SIZE;
      } else if (!isEmpty(otherCustomQuery)) {
        state.query = { ...state.query, ...otherCustomQuery };
      }

      if (has(moreQuery, "page")) {
        state.pagination.page = page as number;
      }
      if (has(moreQuery, "pageSize")) {
        state.pagination.pageSize = pageSize as number;
      }

      state.isLoading = true;
      state.fetchCount++;

      if (state.isRefreshing) {
        state.records = [] as T[];
        state.isRefreshing = false;
      }

      try {
        await options.onLoad(buildLoadQuery(), shouldReset);
        state.errorInfo = null;
        state.isFinished = true;
      } catch (err) {
        handleError(err);
      } finally {
        state.isLoading = false;
      }
    },
    async onReset(customQuery: ListQuery = {}) {
      if (customQuery instanceof Event) customQuery = {};
      state.initState();
      if (state.formRef) {
        await nextTick(async () => {
          await state.onLoad(customQuery, true);
        });
      } else {
        await state.onLoad(customQuery, true);
      }
    },
  }) as unknown as ListState<T>;

  initState();

  watch(
    () => state.query,
    () => {
      state.pagination.page = 1;
    },
    { deep: true },
  );

  return state;
};

/** 与 {@link useList} 配套的 a-table `:pagination`，避免各页重复 onChange 样板 */
export interface UseListPaginationOptions {
  showSizeChanger?: boolean;
  pageSizeOptions?: string[];
  hideOnSinglePage?: boolean;
  showTotal?: (total: number, range: [number, number]) => string;
}

export function useListPagination<T = unknown>(list: ListState<T>, options?: UseListPaginationOptions) {
  const showSizeChanger = options?.showSizeChanger ?? true;
  const pageSizeOptions = options?.pageSizeOptions ?? ["10", "20", "50"];
  const hideOnSinglePage = options?.hideOnSinglePage ?? false;
  const showTotal = options?.showTotal ?? ((t: number, _range: [number, number]) => `共 ${t} 条`);

  const pagination = computed<TablePaginationConfig>(() => ({
    current: list.pagination.page,
    pageSize: list.pagination.pageSize,
    total: list.pagination.total,
    showSizeChanger,
    pageSizeOptions,
    hideOnSinglePage,
    showTotal,
    onChange: (page: number, pageSize?: number) => {
      const nextSize = pageSize ?? list.pagination.pageSize;
      if (nextSize !== list.pagination.pageSize) {
        list.pagination.pageSize = nextSize;
        list.pagination.page = 1;
      } else {
        list.pagination.page = page;
      }
      void list.onLoad({ page: list.pagination.page, pageSize: list.pagination.pageSize });
    },
  }));

  return { pagination };
}
