<!-- Many records or messages side by side: one row each, one column per byte, coloured by
     value, with what each column holds above it. The browser version of
     tools/formats/fields.py and tools/formats/pcap.py. -->
<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import FileDrop from './FileDrop.vue'
import { Flow, classify, flowsFrom, strides, wheel } from './records'
import { hex } from './bytes'

const bytes = ref<Uint8Array | null>(null)
const name = ref('')
const flows = ref<Flow[] | null>(null)
const flowIndex = ref(0)
const direction = ref<'a' | 'b'>('b')
const stride = ref(16)
const offset = ref(0)
const typeAt = ref(-1)
const typeValue = ref(-1)
const picked = ref(-1)
const grid = ref<HTMLCanvasElement>()
const CELL = 10
const COLOURS: Record<string, string> = { constant: '#8a8a8a', counter: '#2ca02c', length: '#1f77b4', type: '#ff7f0e', varies: 'transparent' }

const suggestions = computed(() => bytes.value && !flows.value ? strides(bytes.value.subarray(offset.value)) : [])
const allRows = computed<Uint8Array[]>(() => {
  if (flows.value) {
    const f = flows.value[flowIndex.value]
    return f ? (direction.value === 'a' ? f.a : f.b) : []
  }
  const b = bytes.value
  if (!b || stride.value < 1) return []
  const rows: Uint8Array[] = []
  for (let at = offset.value; at + stride.value <= b.length && rows.length < 2000; at += stride.value) rows.push(b.subarray(at, at + stride.value))
  return rows
})
const types = computed(() => typeAt.value < 0 ? [] : [...new Set(allRows.value.filter(r => r.length > typeAt.value).map(r => r[typeAt.value]))].sort((a, b) => a - b))
const rows = computed(() => typeAt.value < 0 || typeValue.value < 0 ? allRows.value
  : allRows.value.filter(r => r.length > typeAt.value && r[typeAt.value] === typeValue.value))
const notes = computed(() => rows.value.length ? classify(rows.value) : [])
const width = computed(() => Math.min(256, Math.max(0, ...rows.value.map(r => r.length))))

function onFile(b: Uint8Array, n: string) {
  bytes.value = b
  name.value = n
  flows.value = flowsFrom(b)
  flowIndex.value = 0
  offset.value = 0
  typeAt.value = -1
  picked.value = -1
  if (!flows.value && suggestions.value.length) stride.value = suggestions.value[0].size
}

function draw() {
  const c = grid.value
  if (!c) return
  const n = Math.min(rows.value.length, 400)
  c.width = width.value * CELL
  c.height = (n + 1) * CELL
  const ctx = c.getContext('2d')!
  ctx.clearRect(0, 0, c.width, c.height)
  notes.value.forEach((ns, x) => {
    const top = ns[0]
    if (x < width.value && top && COLOURS[top.kind] !== 'transparent') {
      ctx.fillStyle = COLOURS[top.kind]
      ctx.fillRect(x * CELL, 0, CELL, CELL - 2)
    }
  })
  for (let y = 0; y < n; y++) {
    const r = rows.value[y]
    for (let x = 0; x < Math.min(r.length, width.value); x++) {
      const [cr, cg, cb] = wheel(r[x])
      ctx.fillStyle = `rgb(${cr},${cg},${cb})`
      ctx.fillRect(x * CELL, (y + 1) * CELL, CELL - 1, CELL - 1)
    }
  }
}

function pick(ev: MouseEvent) {
  const c = grid.value!, rect = c.getBoundingClientRect()
  const y = Math.floor(((ev.clientY - rect.top) * c.height / rect.height) / CELL) - 1
  if (y >= 0 && y < rows.value.length) picked.value = y
}

watch([rows, width], async () => { await nextTick(); draw() })
</script>

<template>
  <div class="gf-lab">
    <FileDrop label="Drop a data file of records, or a network capture (.pcap, .pcapng)" @file="onFile" />
    <template v-if="bytes">
      <p><strong>{{ name }}</strong>: {{ bytes.length.toLocaleString() }} bytes{{ flows ? `, a capture with ${flows.length} flows` : '' }}.</p>
      <div class="gf-controls" v-if="flows">
        <label>Flow <select v-model.number="flowIndex"><option v-for="(f, i) in flows" :key="f.key" :value="i">{{ f.label }} ({{ f.a.length }} / {{ f.b.length }})</option></select></label>
        <label>Direction <select v-model="direction"><option value="a">first to second</option><option value="b">second to first</option></select></label>
      </div>
      <template v-else>
        <div class="gf-controls">
          <label>Record size <input type="number" v-model.number="stride" min="1" max="4096" /></label>
          <button @click="stride = Math.max(1, stride - 1)">−1</button><button @click="stride += 1">+1</button>
          <label>Start <input type="number" v-model.number="offset" min="0" /></label>
        </div>
        <p class="gf-hint">Likely record sizes:
          <button v-for="s in suggestions" :key="s.size" class="gf-chip" @click="stride = s.size">{{ s.size }} ({{ Math.round(s.score * 100) }}%)</button>
        </p>
      </template>
      <div class="gf-controls">
        <label>Group by the byte at <input type="number" v-model.number="typeAt" min="-1" /></label>
        <label v-if="typeAt >= 0">value <select v-model.number="typeValue"><option :value="-1">all</option><option v-for="t in types" :key="t" :value="t">0x{{ t.toString(16) }}</option></select></label>
      </div>
      <p class="gf-legend">
        <span style="--c:#8a8a8a">constant</span><span style="--c:#2ca02c">counter</span>
        <span style="--c:#1f77b4">length</span><span style="--c:#ff7f0e">few values (a type?)</span>
        {{ rows.length }} rows. Click a row to see its bytes.
      </p>
      <div class="gf-frame"><canvas ref="grid" @click="pick" /></div>
      <pre v-if="picked >= 0" class="gf-hex">{{ hex(rows[picked], 0, 16).join('\n') }}</pre>
      <h3>Each offset</h3>
      <table class="gf-table">
        <tr v-for="(ns, i) in notes.slice(0, 64)" :key="i"><td>+0x{{ i.toString(16) }}</td><td>{{ ns.map(n => n.text).join('; ') }}</td></tr>
      </table>
    </template>
  </div>
</template>

<style scoped>
.gf-controls { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; margin: 8px 0; }
.gf-controls select, .gf-controls input, .gf-controls button, .gf-chip {
  border: 1px solid var(--vp-c-divider); border-radius: 6px; padding: 2px 8px; background: var(--vp-c-bg);
}
.gf-controls input[type=number] { width: 90px; }
.gf-chip { margin: 2px 4px; cursor: pointer; }
.gf-frame { overflow: auto; max-height: 520px; border: 1px solid var(--vp-c-divider); background: #222; padding: 8px; border-radius: 8px; }
.gf-frame canvas { display: block; image-rendering: pixelated; cursor: pointer; }
.gf-hex { font-size: 12px; background: var(--vp-c-bg-soft); padding: 12px; border-radius: 8px; overflow-x: auto; }
.gf-table td { font-size: 13px; padding: 2px 8px; }
.gf-legend span { display: inline-flex; align-items: center; gap: 4px; margin-right: 12px; }
.gf-legend span::before { content: ''; width: 12px; height: 12px; background: var(--c); }
</style>
