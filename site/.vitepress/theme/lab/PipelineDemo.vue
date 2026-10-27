<!-- One frame through the five stages of the pipeline, drawn by three devices side by side:
     the game's state (stage 1), its projected vertices (2), the calls it hands over (3), and the
     pictures a software renderer, a 16-bit card and a modern device make of the same calls (4
     and 5), with the difference from the software reference. The maths is in pipeline.ts. -->
<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { Filter, Mapping, Scene, Store, calls, difference, geometry, makeTexture, rasterise, store } from './pipeline'

const W = 160, H = 120, BIG = 4
const turn = ref(25), tilt = ref(55), distance = ref(2.5)
const mapping = ref<Mapping>('affine')
const filter = ref<Filter>('point')
const showDiff = ref(false)
const tolerance = ref(24)
const canvases = [ref<HTMLCanvasElement>(), ref<HTMLCanvasElement>(), ref<HTMLCanvasElement>()]
const texture = makeTexture()

const scene = computed<Scene>(() => ({ turn: turn.value * Math.PI / 180, tilt: tilt.value * Math.PI / 180,
  distance: distance.value }))
const vertices = computed(() => geometry(scene.value, W, H))
const callList = computed(() => calls(vertices.value, 'rgb565'))

const devices: { name: string; note: string; store: Store }[] = [
  { name: 'Software renderer', note: '160 x 120, 8-bit palette, the mapping and filter you choose', store: 'palette8' },
  { name: '16-bit card', note: '160 x 120, perspective-correct, bilinear, RGB565 with a 4 x 4 dither', store: 'rgb565' },
  { name: 'Modern device', note: '640 x 480, perspective-correct, bilinear, 24-bit colour', store: 'rgb888' },
]
const counts = ref<(number | null)[]>([null, null, null])

function put(c: HTMLCanvasElement | undefined, pixels: Uint8ClampedArray, w: number, h: number) {
  if (!c) return
  c.width = w
  c.height = h
  c.getContext('2d')!.putImageData(new ImageData(pixels, w, h), 0, 0)
}

function draw() {
  const soft = store(rasterise(vertices.value, W, H, texture, mapping.value, filter.value), W, H, 'palette8')
  const card = store(rasterise(vertices.value, W, H, texture, 'perspective', 'bilinear'), W, H, 'rgb565')
  const big = geometry(scene.value, W * BIG, H * BIG)
  const modern = store(rasterise(big, W * BIG, H * BIG, texture, 'perspective', 'bilinear'), W * BIG, H * BIG, 'rgb888')
  const d1 = difference(soft, card, W, H, 1, tolerance.value)
  const d2 = difference(soft, modern, W, H, BIG, tolerance.value)
  counts.value = [0, d1.count, d2.count]
  put(canvases[0].value, soft, W, H)
  if (showDiff.value) {
    put(canvases[1].value, d1.image, W, H)
    put(canvases[2].value, d2.image, W, H)
  } else {
    put(canvases[1].value, card, W, H)
    put(canvases[2].value, modern, W * BIG, H * BIG)
  }
}

onMounted(draw)
watch([scene, mapping, filter, showDiff, tolerance], async () => { await nextTick(); draw() })
</script>

<template>
  <div class="gf-pipe">
    <section>
      <h4><span class="n s1">1</span> Game state</h4>
      <p>The game decides where things are. Here it is one square with a texture, and a camera.</p>
      <div class="gf-controls">
        <label>turn <input v-model.number="turn" type="range" min="-60" max="60" /> {{ turn }}°</label>
        <label>tilt <input v-model.number="tilt" type="range" min="0" max="75" /> {{ tilt }}°</label>
        <label>distance <input v-model.number="distance" type="range" min="2" max="5" step="0.1" /> {{ distance }}</label>
      </div>
    </section>

    <section>
      <h4><span class="n s2">2</span> Geometry, on the CPU</h4>
      <p>The game turns the corners, moves them away from the camera and divides by their depth. Each
        corner keeps <code>1/w</code> and its texture place divided by <code>w</code>, so a card can
        correct the perspective later. These numbers are what a port must match bit for bit.</p>
      <table class="gf-table">
        <thead><tr><th>corner</th><th>x</th><th>y</th><th>1/w</th><th>s/w</th><th>t/w</th></tr></thead>
        <tbody>
          <tr v-for="(v, k) in vertices" :key="k">
            <td>{{ k }}</td><td>{{ v.x.toFixed(2) }}</td><td>{{ v.y.toFixed(2) }}</td>
            <td>{{ v.oow.toFixed(5) }}</td><td>{{ v.sow.toFixed(3) }}</td><td>{{ v.tow.toFixed(3) }}</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section>
      <h4><span class="n s3">3</span> The seam: the calls of one frame</h4>
      <p>Everything the device draws arrives through calls like these. Record them from the original
        and from the port, and compare them call for call.</p>
      <pre class="gf-calls">{{ callList.join('\n') }}</pre>
    </section>

    <section>
      <h4><span class="n s4">4</span> Rasterising, and <span class="n s5">5</span> what each device stores</h4>
      <p>The same calls, drawn by three devices. Try the software renderer's mapping: <b>affine</b>
        bends the chequer, as many renderers of the early 1990s did; <b>every 16 pixels</b>
        corrects it at the start of each 16-pixel run, as Quake's software renderer did.</p>
      <div class="gf-controls">
        <label>software mapping
          <select v-model="mapping">
            <option value="affine">affine</option>
            <option value="span16">perspective every 16 pixels</option>
            <option value="perspective">perspective at every pixel</option>
          </select>
        </label>
        <label>software filter
          <select v-model="filter"><option value="point">nearest texel</option><option value="bilinear">bilinear</option></select>
        </label>
        <label><input v-model="showDiff" type="checkbox" /> show the difference from the software renderer</label>
        <label>tolerance <input v-model.number="tolerance" type="range" min="0" max="64" /> {{ tolerance }}</label>
      </div>
      <div class="gf-devices">
        <figure v-for="(d, k) in devices" :key="d.name">
          <canvas :ref="(el) => (canvases[k].value = el as HTMLCanvasElement)" :class="{ big: k === 2 && !showDiff }" />
          <figcaption>
            <b>{{ d.name }}</b><br />{{ d.note }}<br />
            <span v-if="k === 0" class="ref">the reference</span>
            <span v-else-if="counts[k] !== null">{{ counts[k] }} of {{ W * H }} pixels differ by more than {{ tolerance }}</span>
          </figcaption>
        </figure>
      </div>
      <p class="gf-note">Set the tolerance to 0 to see why a card can only be compared exactly with a
        model of the same card: almost every pixel differs from the 8-bit reference.
        Stages 1 to 3 are the same for all three. Only the device changed. The 16-bit
        card here shows the kind of picture a Voodoo made; it is not the chip. GameFrame's
        <code>gameframe-voodoo</code> crate models the chip from 3dfx's specification.</p>
    </section>
  </div>
</template>

<style scoped>
.gf-pipe section { border: 1px solid var(--vp-c-divider); border-radius: 10px; padding: 12px 16px; margin: 16px 0; }
.gf-pipe h4 { margin: 0 0 6px; display: flex; align-items: center; gap: 8px; }
.n { display: inline-flex; width: 24px; height: 24px; border-radius: 6px; align-items: center; justify-content: center;
  font-size: 13px; font-weight: 700; color: #0d1b1e; }
.s1, .s2 { background: #2dd4bf; } .s3 { background: #f59e0b; } .s4 { background: #c084fc; } .s5 { background: #60a5fa; }
.gf-controls { display: flex; flex-wrap: wrap; gap: 12px; align-items: center; margin: 8px 0; font-size: 14px; }
.gf-controls select { border: 1px solid var(--vp-c-divider); border-radius: 6px; padding: 2px 8px; background: var(--vp-c-bg); }
.gf-table { font-family: var(--vp-font-family-mono); font-size: 13px; }
.gf-calls { font-size: 12px; overflow-x: auto; background: var(--vp-c-bg-soft); padding: 10px; border-radius: 8px; }
.gf-devices { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 12px; }
.gf-devices figure { margin: 0; }
.gf-devices canvas { width: 100%; aspect-ratio: 4 / 3; image-rendering: pixelated; border-radius: 6px; background: #222; display: block; }
.gf-devices canvas.big { image-rendering: auto; }
.gf-devices figcaption { font-size: 13px; line-height: 1.45; margin-top: 6px; color: var(--vp-c-text-2); }
.ref { color: #2dd4bf; font-weight: 600; }
.gf-note { font-size: 13px; color: var(--vp-c-text-2); }
</style>
