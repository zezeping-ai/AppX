<script setup lang="ts">
import EditorEncryptionSection from "./EditorEncryptionSection.vue";
</script>

<template>
  <a-space direction="vertical" size="middle" class="editor-settings">
    <EditorEncryptionSection />
  </a-space>
</template>

<style scoped lang="scss">
.editor-settings {
  width: 100%;
  max-width: 760px;
}
</style>
