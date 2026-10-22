<!-- Try bytes as a picture: change the mode, the width, the offset and the palette, and see
     the result at once. The browser version of tools/formats/rawimage.py. -->
<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import FileDrop from './FileDrop.vue'
import { CGA1, GREYS, Mode, RGB, decode, egaPalette, paletteFrom, vgaPalette } from './bytes'

const bytes = ref<Uint8Array | null>(null)
const name = ref('')
const palBytes = ref<Uint8Array | null>(null)
const mode = ref<Mode>('linear8')
const width = ref(320)
const height = ref(200)
const offset = ref(0)
const palOffset = ref(0)
const start = ref(0)
const scale = ref(2)
const paletteKind = ref<'default' | 'greys' | 'file'>('default')
const canvas = ref<HTMLCanvasElement>()

const modes: [Mode, string][] = [
  ['linear8', '8 bits per pixel (mode 13h)'], ['modex', 'Mode X (4 planes, 256 colours)'],
  ['planar4', 'EGA/VGA 16 colours (4 bit planes)'], ['cga2', 'CGA 4 colours (interleaved lines)'],
  ['packed4', '4 bits per pixel'], ['packed2', '2 bits per pixel'], ['packed1', '1 bit per pixel'],
  ['rgb565', '16-bit RGB 565'], ['rgb555', '16-bit RGB 555'],
]

const bytesPerRow = computed(() => ({ linear8: width.value, modex: width.value, planar4: width.value / 8,
  cga2: width.value / 4, packed4: width.value / 2, packed2: width.value / 4, packed1: width.value / 8,
  rgb565: width.value * 2, rgb555: width.value * 2 })[mode.value])

const palette = computed<RGB[]>(() => {
  if (paletteKind.value === 'file' && palBytes.value) return paletteFrom(palBytes.value, palOffset.value)
  if (paletteKind.value === 'greys') return GREYS
  if (mode.value === 'planar4' || mode.value === 'packed4') return egaPalette()
  if (mode.value === 'cga2' || mode.value === 'packed2') return CGA1
  if (mode.value === 'packed1') return [[0, 0, 0], [255, 255, 255]]
  return vgaPalette()
})

function onFile(b: Uint8Array, n: string) { bytes.value = b; name.value = n; offset.value = 0 }
function onPalette(b: Uint8Array) { palBytes.value = b; paletteKind.value = 'file' }

function draw() {
  const c = canvas.value
  if (!c || !bytes.value) return
  const w = Math.max(1, Math.min(4096, width.value | 0)), h = Math.max(1, Math.min(4096, height.value | 0))
  c.width = w
  c.height = h
  c.style.width = `${w * scale.value}px`
  const pixels = decode(bytes.value.subarray(offset.value), mode.value, w, h, palette.value, start.value)
  c.getContext('2d')!.putImageData(new ImageData(pixels, w, h), 0, 0)
}

watch([bytes, mode, width, height, offset, start, palette, scale], async () => { await nextTick(); draw() }, { deep: true })
</script>

<template>
  <div class="gf-lab">
    <FileDrop label="Drop the data here: a memory dump, a screen file, a block of a data file" @file="onFile" />
    <template v-if="bytes">
      <div class="gf-controls">
        <label>Mode <select v-model="mode"><option v-for="[m, t] in modes" :key="m" :value="m">{{ t }}</option></select></label>
        <label>Width <input type="number" v-model.number="width" min="1" max="4096" /></label>
        <button @click="width -= 1">−1</button><button @click="width += 1">+1</button>
        <button @click="width -= 8">−8</button><button @click="width += 8">+8</button>
        <label>Height <input type="number" v-model.number="height" min="1" max="4096" /></label>
      </div>
      <div class="gf-controls">
        <label>Offset <input type="number" v-model.number="offset" min="0" :max="bytes.length" /></label>
        <button @click="offset = Math.max(0, offset - 1)">−1 byte</button><button @click="offset += 1">+1 byte</button>
        <button @click="offset = Math.max(0, offset - bytesPerRow)">−1 row</button><button @click="offset += bytesPerRow">+1 row</button>
        <label v-if="mode === 'modex' || mode === 'planar4'">First row <input type="number" v-model.number="start" min="0" /></label>
        <label>Zoom <select v-model.number="scale"><option :value="1">1×</option><option :value="2">2×</option><option :value="3">3×</option><option :value="4">4×</option></select></label>
      </div>
      <div class="gf-controls">
        <label>Palette <select v-model="paletteKind"><option value="default">default for the mode</option><option value="greys">greys</option><option value="file">from a file</option></select></label>
        <label v-if="paletteKind === 'file'">Palette offset <input type="number" v-model.number="palOffset" min="0" /></label>
      </div>
      <FileDrop v-if="paletteKind === 'file'" label="Drop a palette: 768 bytes of RGB, or any file that holds one (set the offset)" @file="onPalette" />
      <p>{{ name }}: {{ bytes.length.toLocaleString() }} bytes; one row is {{ bytesPerRow }} bytes; the picture uses
        {{ (bytesPerRow * height).toLocaleString() }} bytes from offset {{ offset.toLocaleString() }} (0x{{ offset.toString(16) }}).</p>
      <div class="gf-frame"><canvas ref="canvas" /></div>
    </template>
  </div>
</template>

<style scoped>
.gf-controls { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; margin: 8px 0; }
.gf-controls select, .gf-controls input, .gf-controls button {
  border: 1px solid var(--vp-c-divider); border-radius: 6px; padding: 2px 8px; background: var(--vp-c-bg);
}
.gf-controls input[type=number] { width: 96px; }
.gf-frame { overflow: auto; max-width: 100%; border: 1px solid var(--vp-c-divider); background: #222; padding: 8px; border-radius: 8px; }
.gf-frame canvas { image-rendering: pixelated; display: block; }
</style>
