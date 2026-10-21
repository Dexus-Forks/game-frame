<!-- The home page's "Last updated" date: the newest commit to any page (set in config.mts). -->
<script setup lang="ts">
import { computed } from 'vue'
import { useData } from 'vitepress'

const { page } = useData()
// Formatted in UTC, so the server render and the browser agree.
const date = computed(() => page.value.lastUpdated
  ? new Date(page.value.lastUpdated).toLocaleDateString('en-GB', { dateStyle: 'long', timeZone: 'UTC' })
  : '')
</script>

<template>
  <p v-if="date" class="home-last-updated">
    Last updated: <time :datetime="new Date(page.lastUpdated!).toISOString()">{{ date }}</time>
  </p>
</template>

<style scoped>
.home-last-updated {
  max-width: 1152px;
  margin: 32px auto 0;
  padding: 0 24px;
  font-size: 14px;
  color: var(--vp-c-text-2);
}
@media (min-width: 640px) { .home-last-updated { padding: 0 48px; } }
@media (min-width: 960px) { .home-last-updated { padding: 0 64px; } }
</style>
