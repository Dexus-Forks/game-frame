<!-- Look at a file: what kind it is, a map of its bytes, its entropy along its length, and the
     bytes at any place you click. The browser version of filetypes.py and binview.py. -->
<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import FileDrop from './FileDrop.vue'
import { CLASS_COLOURS, entropy, guessKind, hex, hilbert, kindOf, ramp } from './bytes'

const bytes = ref<Uint8Array | null>(null)
const name = ref('')
const layout = ref<'rows' | 'hilbert'>('rows')
const colour = ref<'class' | 'entropy'>('class')
const width = ref(256)
const offset = ref(0)
const map = ref<HTMLCanvasElement>()
const graph = ref<HTMLCanvasElement>()
const MAX = 1 << 20

const per = computed(() => bytes.value ? Math.max(1, Math.ceil(bytes.value.length / MAX)) : 1)
const kind = computed(() => bytes.value ? guessKind(bytes.value) : '')
const total = computed(() => bytes.value ? entropy(bytes.value).toFixed(2) : '')
const dump = computed(() => bytes.value ? hex(bytes.value, offset.value) : [])
const blockSize = computed(() => bytes.value ? Math.max(1024, Math.ceil(bytes.value.length / 600)) : 1024)
// The most entropy a window of n bytes can show: random data reaches it.
const most = (n: number) => Math.log2(Math.min(256, Math.max(2, n)))
const blocks = computed(() => {
  const b = bytes.value
  if (!b) return [] as number[]
  const out: number[] = []
  for (let i = 0; i < b.length; i += blockSize.value) out.push(entropy(b, i, Math.min(b.length, i + blockSize.value)))
  return out
})

function onFile(b: Uint8Array, n: string) {
  bytes.value = b
  name.value = n
  offset.value = 0
}

function colourAt(i: number): [number, number, number] {
  const b = bytes.value!
  if (colour.value === 'entropy') {
    const mid = i * per.value
    const lo = Math.max(0, mid - 512), hi = Math.min(b.length, mid + 512)
    return ramp(entropy(b, lo, hi) / most(hi - lo))
  }
  if (per.value === 1) return CLASS_COLOURS[kindOf(b[i])]
  const tally: Record<string, number> = {}
  for (let k = i * per.value; k < Math.min(b.length, (i + 1) * per.value); k++) {
    const c = kindOf(b[k])
    tally[c] = (tally[c] ?? 0) + 1
  }
  return CLASS_COLOURS[Object.entries(tally).sort((a, c) => c[1] - a[1])[0][0]]
}

function drawMap() {
  const b = bytes.value, canvas = map.value
  if (!b || !canvas) return
  const n = Math.ceil(b.length / per.value)
  let w: number, h: number
  const order = Math.max(1, Math.ceil(Math.log2(Math.sqrt(n))))
  if (layout.value === 'hilbert') { w = h = 1 << order } else { w = width.value; h = Math.ceil(n / w) }
  canvas.width = w
  canvas.height = h
  const ctx = canvas.getContext('2d')!
  const img = ctx.createImageData(w, h)
  img.data.fill(40)
  for (let i = 0; i < n; i++) {
    const [x, y] = layout.value === 'hilbert' ? hilbert(i, order) : [i % w, Math.floor(i / w)]
    const c = colourAt(i), p = (y * w + x) * 4
    img.data[p] = c[0]; img.data[p + 1] = c[1]; img.data[p + 2] = c[2]; img.data[p + 3] = 255
  }
  ctx.putImageData(img, 0, 0)
}

function drawGraph() {
  const canvas = graph.value
  if (!canvas || !blocks.value.length) return
  const w = canvas.width = 800, h = canvas.height = 140
  const ctx = canvas.getContext('2d')!
  ctx.clearRect(0, 0, w, h)
  ctx.fillStyle = 'rgba(128,128,128,0.15)'
  ctx.fillRect(0, h - 0.94 * h, w, 1)
  const n = blocks.value.length, top = most(blockSize.value)
  blocks.value.forEach((e, i) => {
    const f = Math.min(1, e / top)
    const [r, g, b2] = ramp(f)
    ctx.fillStyle = `rgb(${r},${g},${b2})`
    const x = Math.floor(i * w / n), x2 = Math.floor((i + 1) * w / n)
    ctx.fillRect(x, h - f * h, Math.max(1, x2 - x), f * h)
  })
  const at = Math.floor((offset.value / bytes.value!.length) * w)
  ctx.fillStyle = 'rgba(255,255,255,0.9)'
  ctx.fillRect(at, 0, 1, h)
}

function clickMap(ev: MouseEvent) {
  const canvas = map.value!, rect = canvas.getBoundingClientRect()
  const x = Math.floor((ev.clientX - rect.left) * canvas.width / rect.width)
  const y = Math.floor((ev.clientY - rect.top) * canvas.height / rect.height)
  let i: number
  if (layout.value === 'hilbert') {
    const order = Math.log2(canvas.width)
    i = 0
    for (let d = 0; d < canvas.width * canvas.width; d++) { const [hx, hy] = hilbert(d, order); if (hx === x && hy === y) { i = d; break } }
  } else i = y * canvas.width + x
  offset.value = Math.min(bytes.value!.length - 1, i * per.value) & ~15
}

function clickGraph(ev: MouseEvent) {
  const rect = graph.value!.getBoundingClientRect()
  offset.value = Math.floor(((ev.clientX - rect.left) / rect.width) * bytes.value!.length) & ~15
}

watch([bytes, layout, colour, width], async () => { await nextTick(); drawMap(); drawGraph() })
watch(offset, drawGraph)
</script>

<template>
  <div class="gf-lab">
    <FileDrop @file="onFile" />
    <template v-if="bytes">
      <p><strong>{{ name }}</strong>: {{ bytes.length.toLocaleString() }} bytes. Signature: <strong>{{ kind }}</strong>.
        Entropy over the whole file: <strong>{{ total }}</strong> bits per byte.</p>
      <div class="gf-controls">
        <label>Layout <select v-model="layout"><option value="rows">rows</option><option value="hilbert">Hilbert curve</option></select></label>
        <label>Colour <select v-model="colour"><option value="class">kind of byte</option><option value="entropy">entropy</option></select></label>
        <label v-if="layout === 'rows'">Width <input type="number" v-model.number="width" min="8" max="4096" step="8" /></label>
        <span v-if="per > 1">{{ per }} bytes per pixel</span>
      </div>
      <p class="gf-legend" v-if="colour === 'class'">
        <span style="--c:#000">0x00</span><span style="--c:#fff">0xFF</span><span style="--c:#377edc">text</span>
        <span style="--c:#4daf4a">control</span><span style="--c:#e41a1c">0x80 to 0xFE</span>
        Click the map or the graph to see the bytes there.
      </p>
      <canvas ref="map" class="gf-map" @click="clickMap" />
      <h3>Entropy along the file</h3>
      <p>Each bar is {{ blockSize.toLocaleString() }} bytes, drawn against the most entropy a block of
        that size can have. Red bars that reach the faint line near the top are compressed or encrypted.</p>
      <canvas ref="graph" class="gf-graph" @click="clickGraph" />
      <h3>Bytes at 0x{{ offset.toString(16) }}</h3>
      <div class="gf-controls">
        <button @click="offset = Math.max(0, offset - 256)">Back 256</button>
        <button @click="offset = Math.min(bytes.length - 16, offset + 256)">On 256</button>
        <label>Offset <input type="text" :value="'0x' + offset.toString(16)"
          @change="offset = (parseInt(($event.target as HTMLInputElement).value) || 0) & ~15" /></label>
      </div>
      <pre class="gf-hex">{{ dump.join('\n') }}</pre>
    </template>
  </div>
</template>

<style scoped>
.gf-controls { display: flex; flex-wrap: wrap; gap: 12px; align-items: center; margin: 8px 0; }
.gf-controls select, .gf-controls input, .gf-controls button {
  border: 1px solid var(--vp-c-divider); border-radius: 6px; padding: 2px 8px; background: var(--vp-c-bg);
}
.gf-controls input[type=number] { width: 90px; }
.gf-map { width: 100%; max-width: 768px; image-rendering: pixelated; border: 1px solid var(--vp-c-divider); cursor: crosshair; }
.gf-graph { width: 100%; height: 140px; border: 1px solid var(--vp-c-divider); background: var(--vp-c-bg-soft); cursor: crosshair; }
.gf-hex { font-size: 12px; line-height: 1.5; overflow-x: auto; background: var(--vp-c-bg-soft); padding: 12px; border-radius: 8px; }
.gf-legend span { display: inline-flex; align-items: center; gap: 4px; margin-right: 12px; }
.gf-legend span::before { content: ''; width: 12px; height: 12px; background: var(--c); border: 1px solid var(--vp-c-divider); }
</style>
