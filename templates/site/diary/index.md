# Diary

A dated log of the work: what was tried, what was found, and how each finding is known. The
reference documents are under [Findings](/findings), and what is done and open is under
[Status](/status).

<script setup>
import { data as posts } from './posts.data.mts'
</script>

<div v-for="p in posts" :key="p.url" class="entry">
  <p><strong><a :href="p.url">{{ p.title }}</a></strong> · {{ p.date }}</p>
  <p>{{ p.summary }}</p>
</div>
