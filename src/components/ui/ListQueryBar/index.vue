<script setup lang="ts">
const emit = defineEmits<{ search: [] }>();
</script>

<template>
  <div class="list-query-bar flex min-w-0 shrink-0 flex-wrap items-center gap-x-3 gap-y-2">
    <a-form layout="inline" class="list-query-form min-w-0 shrink grow" @finish="emit('search')">
      <div class="list-query-form__row">
        <slot />
      </div>
    </a-form>
    <div v-if="$slots.extra" class="list-query-bar__extra">
      <slot name="extra" />
    </div>
  </div>
</template>

<style scoped>
.list-query-form:deep(.ant-form-item) {
  margin-bottom: 0;
}

.list-query-form:deep(.ant-form-item-row) {
  align-items: center;
}

.list-query-form:deep(.ant-form-item-label) {
  display: flex;
  align-items: center;
}

.list-query-form:deep(.ant-form-item-label > label) {
  height: auto;
}

.list-query-form:deep(.ant-form-item-control-input) {
  min-height: unset;
}

.list-query-form__row {
  display: flex;
  min-width: 0;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem 0.75rem;
}

.list-query-bar__extra {
  display: flex;
  min-width: 0;
  flex: 0 1 auto;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-left: auto;
}

.list-query-bar__extra:deep(.ant-space) {
  flex-wrap: wrap;
  justify-content: flex-end;
}
</style>
