<script setup lang="ts">
import EditorEncryptionSection from "./EditorEncryptionSection.vue";
</script>

<template>
  <a-space direction="vertical" size="small" class="editor-settings">
    <EditorEncryptionSection />
  </a-space>
</template>

<style scoped lang="scss">
.editor-settings {
  width: 100%;
  max-width: 640px;
}
</style>
