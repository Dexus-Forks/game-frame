<!-- Where each stage of a frame ran, machine by machine: pick a machine and see who did each
     stage, what crossed the seam, what to port exactly and what to check. The same table as
     docs/assets/diagrams/pipeline-eras.svg, with the advice of docs/hardware/rendering-pipeline.md. -->
<script setup lang="ts">
import { computed, ref } from 'vue'

type Who = 'CPU' | 'driver' | 'split' | 'card' | 'program' | 'VGA' | 'calls' | 'inside'
interface Machine { name: string; years: string; who: Who[]; seam: string; calls: string; port: string; check: string }

const STAGES = ['game state', 'geometry', 'the seam', 'rasterising', 'display']
const MACHINES: Machine[] = [
  { name: 'Software renderer', years: '1993-1998', who: ['CPU', 'CPU', 'inside', 'CPU', 'VGA'],
    seam: 'Inside the program: the game hands spans or polygons to its own drawing routines.',
    calls: 'draw_span(y, x0, x1, s, t, ds, dt)  (names vary by game)',
    port: 'Everything, the drawing routines too. This renderer is the best reference: the same input gives the same pixels on any machine.',
    check: 'Pixels, exactly, against frames captured from the original.' },
  { name: 'S3 ViRGE, ATI Rage', years: '1995-1997', who: ['CPU', 'CPU', 'calls', 'split', 'card'],
    seam: 'Faces in the card maker\'s own format. The builds of the time often gave the chip only some faces, untextured or blended, and drew the rest with the CPU.',
    calls: 'S3DTK_TriangleSet(...)  /  ATI3DCIF RenderPrimList(...)',
    port: 'Stages 1 to 3 exactly, and the list of what each build gives the chip.',
    check: 'The call stream; the pixels against gameframe-virge or gameframe-rage.' },
  { name: '3dfx Voodoo (Glide)', years: '1996-1998', who: ['CPU', 'CPU', 'calls', 'card', 'card'],
    seam: 'Triangles already projected to the screen: x and y, 1/w, s/w and t/w per vertex, with the states set before them.',
    calls: 'grTexSource(...); grDrawTriangle(&a, &b, &c); grBufferSwap(1)',
    port: 'Stages 1 to 3 exactly. The game\'s own maths decides every vertex.',
    check: 'The call stream, call for call; the pixels against gameframe-voodoo, which follows 3dfx\'s register specification.' },
  { name: 'PowerVR (SGL)', years: '1997', who: ['CPU', 'driver', 'calls', 'card', 'card'],
    seam: 'Higher than usual: a scene graph of transforms, meshes and lights that SGL\'s library transforms on the CPU, or faces already projected (sgltri_*).',
    calls: 'sgl_modify_transform(...); sgl_render(...)',
    port: 'Stage 1 and the scene the game builds. Stage 2 is the library\'s work, so the port must do what SGL did.',
    check: 'The scene graph and its changes each frame; then the faces it produces.' },
  { name: 'Direct3D 3 to 6', years: '1996-1998', who: ['CPU', 'CPU', 'calls', 'card', 'card'],
    seam: 'Execute buffers (Direct3D 3), then DrawPrimitive (5): vertices the game transformed itself, or ones Direct3D transformed on the CPU.',
    calls: 'Execute(buffer) with D3DOP_TRIANGLE  /  DrawPrimitive(D3DPT_TRIANGLELIST, ...)',
    port: 'Stages 1 to 3, and Direct3D\'s own transform when the game used it.',
    check: 'The call stream; the pixels with a tolerance, since each card and driver drew differently.' },
  { name: 'Direct3D 7, GeForce 256', years: '1999', who: ['CPU', 'card', 'calls', 'card', 'card'],
    seam: 'Untransformed vertices, with the matrices and lights set as states. The card does transform and lighting.',
    calls: 'SetTransform(...); SetLight(...); DrawIndexedPrimitive(...)',
    port: 'Stage 1 and the states exactly; stage 2 is now fixed-function maths the port reproduces.',
    check: 'The states and calls; the pixels with a tolerance.' },
  { name: 'Shaders (Direct3D 8 on)', years: '2001-', who: ['CPU', 'program', 'calls', 'program', 'card'],
    seam: 'Vertex buffers, constants and compiled shader programs.',
    calls: 'SetVertexShader(...); SetVertexShaderConstant(...); DrawPrimitive(...)',
    port: 'Stage 1, the constants and the shaders, translated from their bytecode.',
    check: 'The constants and calls; the pixels with a tolerance.' },
]
const COLOURS: Record<Who, string> = { CPU: '#2dd4bf', driver: '#c084fc', split: '#f87171', card: '#60a5fa', program: '#4ade80', VGA: '#64748b',
  calls: '#f59e0b', inside: '#f59e0b' }
const MEANING: Record<Who, string> = { CPU: 'the game\'s own code', driver: 'a library on the CPU', split: 'some faces each',
  card: 'fixed hardware', program: 'a shader program', VGA: 'the 2D card shows the frame',
  calls: 'the hand-over', inside: 'calls within one program' }

const picked = ref(2)
const m = computed(() => MACHINES[picked.value])
</script>

<template>
  <div class="gf-ex">
    <div class="gf-tabs">
      <button v-for="(x, k) in MACHINES" :key="x.name" :class="{ on: k === picked }" @click="picked = k">
        {{ x.name }}<small>{{ x.years }}</small>
      </button>
    </div>
    <div class="gf-strip">
      <div v-for="(stage, k) in STAGES" :key="stage" class="gf-stage" :class="{ seam: k === 2 }"
           :style="{ borderColor: COLOURS[m.who[k]], background: COLOURS[m.who[k]] + '26' }">
        <small>{{ k + 1 }}</small>
        <b>{{ stage }}</b>
        <span :style="{ color: COLOURS[m.who[k]] }">{{ m.who[k] }}</span>
        <em>{{ MEANING[m.who[k]] }}</em>
      </div>
    </div>
    <dl>
      <dt>What crosses the seam</dt><dd>{{ m.seam }}</dd>
      <dt>Typical calls</dt><dd><code>{{ m.calls }}</code></dd>
      <dt>Port exactly</dt><dd>{{ m.port }}</dd>
      <dt>Check</dt><dd>{{ m.check }}</dd>
    </dl>
  </div>
</template>

<style scoped>
.gf-ex { border: 1px solid var(--vp-c-divider); border-radius: 10px; padding: 12px 16px; margin: 16px 0; }
.gf-tabs { display: flex; flex-wrap: wrap; gap: 6px; }
.gf-tabs button { border: 1px solid var(--vp-c-divider); border-radius: 8px; padding: 4px 10px; font-size: 13px;
  display: flex; flex-direction: column; align-items: flex-start; background: var(--vp-c-bg); }
.gf-tabs button small { color: var(--vp-c-text-3); font-size: 11px; }
.gf-tabs button.on { border-color: var(--vp-c-brand-1); background: var(--vp-c-brand-soft); }
.gf-strip { display: grid; grid-template-columns: repeat(5, 1fr); gap: 6px; margin: 14px 0; }
.gf-stage { border: 2px solid; border-radius: 8px; padding: 8px; display: flex; flex-direction: column; gap: 2px;
  font-size: 13px; transition: background 0.25s, border-color 0.25s; min-width: 0; }
.gf-stage.seam { border-style: dashed; }
.gf-stage small { color: var(--vp-c-text-3); }
.gf-stage span { font-weight: 700; }
.gf-stage em { font-size: 11px; color: var(--vp-c-text-2); font-style: normal; }
dl { margin: 0; font-size: 14px; }
dt { font-weight: 600; margin-top: 8px; }
dd { margin: 2px 0 0; color: var(--vp-c-text-2); }
@media (max-width: 640px) { .gf-strip { grid-template-columns: 1fr; } }
</style>
