<!-- A place to drop or choose a file. The file is read in the browser with FileReader and
     handed to the page as bytes; it is never sent anywhere. -->
<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{ label?: string, accept?: string }>()
const emit = defineEmits<{ (e: 'file', bytes: Uint8Array, name: string): void }>()
const over = ref(false)
const name = ref('')

async function take(file: File | undefined) {
  if (!file) return
  name.value = `${file.name} (${file.size.toLocaleString()} bytes)`
  emit('file', new Uint8Array(await file.arrayBuffer()), file.name)
}
</script>

<template>
  <label class="gf-drop" :class="{ over }" @dragover.prevent="over = true" @dragleave="over = false"
         @drop.prevent="over = false; take($event.dataTransfer?.files[0])">
    <input type="file" :accept="props.accept" @change="take(($event.target as HTMLInputElement).files?.[0])" />
    <span>{{ name || props.label || 'Drop a file here, or click to choose one' }}</span>
    <small>Read in your browser only. Nothing is uploaded.</small>
  </label>
</template>

<style scoped>
.gf-drop {
  display: flex; flex-direction: column; align-items: center; gap: 4px;
  padding: 20px; border: 2px dashed var(--vp-c-divider); border-radius: 10px;
  cursor: pointer; background: var(--vp-c-bg-soft); text-align: center; margin: 12px 0;
}
.gf-drop.over { border-color: var(--vp-c-brand-1); background: var(--vp-c-brand-soft); }
.gf-drop input { display: none; }
.gf-drop small { color: var(--vp-c-text-2); }
</style>
