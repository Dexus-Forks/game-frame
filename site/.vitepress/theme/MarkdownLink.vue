<script setup lang="ts">
// A small link under each doc page to its plain Markdown copy (written by llms.mts).
import { useData, withBase } from 'vitepress'
const { page } = useData()
</script>

<template>
  <p class="markdown-link">
    <a :href="withBase('/' + page.relativePath)" target="_blank" rel="alternate" type="text/markdown">View this page as Markdown</a>
    · <a :href="withBase('/llms.txt')" target="_blank">llms.txt</a>
  </p>
</template>

<style scoped>
.markdown-link {
  margin-top: 24px;
  font-size: 13px;
  color: var(--vp-c-text-3);
}
.markdown-link a {
  color: var(--vp-c-text-2);
  text-decoration: underline dotted;
}
.markdown-link a:hover {
  color: var(--vp-c-brand-1);
}
</style>
