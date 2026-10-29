// The rendering pipeline in miniature, for the pipeline lab page: one textured square, taken
// through stages 1 to 3 once and drawn by three devices. Stage 2 projects its corners to the
// screen with 1/w, as a game of 1997 did on the CPU. Stage 4 rasterises the two triangles with a
// chosen texture mapping (affine, perspective every 16 pixels, or exact) and filter. Stage 5
// turns the colour into what each device stored: an 8-bit palette index, 16-bit RGB565 with a
// 4 x 4 ordered dither, or 24-bit colour. The 16-bit device is an illustration of the Voodoo's
// look, not a model of the chip: crates/voodoo is that model.

export const TEX = 64
export type Mapping = 'affine' | 'span16' | 'perspective'
export type Filter = 'point' | 'bilinear'
export type Store = 'palette8' | 'rgb565' | 'rgb888'

export interface Scene { turn: number; tilt: number; distance: number }
export interface Vertex { x: number; y: number; oow: number; sow: number; tow: number }

// Stage 1: our own texture, made here: a chequer of 16-texel squares with a gradient and
// gold lines between the squares.
export function makeTexture(): Uint8ClampedArray {
  const t = new Uint8ClampedArray(TEX * TEX * 3)
  for (let y = 0; y < TEX; y++) {
    for (let x = 0; x < TEX; x++) {
      const i = (y * TEX + x) * 3
      const check = ((x >> 4) ^ (y >> 4)) & 1
      const line = (x & 15) === 0 || (y & 15) === 0
      t[i] = line ? 250 : check ? 40 + x : 235
      t[i + 1] = line ? 210 : check ? 120 + y : 200 - y
      t[i + 2] = line ? 80 : check ? 150 : 120
    }
  }
  return t
}

// Stage 2: the square's corners turned about the vertical, tilted back like a floor, moved
// away and projected. The texture repeats
// twice across it, so s and t run from 0 to 128.
export function geometry(scene: Scene, w: number, h: number): Vertex[] {
  const corners = [[-1, -1, 0, 0], [1, -1, 128, 0], [1, 1, 128, 128], [-1, 1, 0, 128]]
  const ct = Math.cos(scene.turn), st = Math.sin(scene.turn)
  const cp = Math.cos(scene.tilt), sp = Math.sin(scene.tilt)
  const focal = h
  return corners.map(([cx, cy, s, t]) => {
    const x = cx * ct, z0 = cx * st
    const y = cy * cp - z0 * sp, z = cy * sp + z0 * cp + scene.distance
    const oow = 1 / z
    return { x: w / 2 + focal * x * oow, y: h / 2 - focal * y * oow, oow, sow: s * oow, tow: t * oow }
  })
}

// Stage 3: the calls a frame hands over, in the style of a card API.
export function calls(v: Vertex[], store: Store): string[] {
  const fmt = (n: number, d = 1) => n.toFixed(d).padStart(6)
  const vx = (k: number) => `{x:${fmt(v[k].x)} y:${fmt(v[k].y)} 1/w:${v[k].oow.toFixed(4)} s/w:${fmt(v[k].sow, 2)} t/w:${fmt(v[k].tow, 2)}}`
  return [
    `bufferClear(colour 0x10202a)`,
    `texSource(tex0, ${TEX} x ${TEX}, ${store === 'palette8' ? '8-bit indices' : store === 'rgb565' ? 'RGB565' : 'RGB888'})`,
    `drawTriangle(${vx(0)},`, `             ${vx(1)},`, `             ${vx(2)})`,
    `drawTriangle(${vx(0)},`, `             ${vx(2)},`, `             ${vx(3)})`,
    `bufferSwap()`,
  ]
}

function sample(tex: Uint8ClampedArray, s: number, t: number, filter: Filter, out: number[]) {
  const wrap = (n: number) => ((n % TEX) + TEX) % TEX
  if (filter === 'point') {
    const i = (wrap(Math.floor(t)) * TEX + wrap(Math.floor(s))) * 3
    out[0] = tex[i]; out[1] = tex[i + 1]; out[2] = tex[i + 2]
    return
  }
  const fs = s - 0.5, ft = t - 0.5
  const x0 = Math.floor(fs), y0 = Math.floor(ft), ax = fs - x0, ay = ft - y0
  for (let c = 0; c < 3; c++) {
    const at = (x: number, y: number) => tex[(wrap(y) * TEX + wrap(x)) * 3 + c]
    const top = at(x0, y0) * (1 - ax) + at(x0 + 1, y0) * ax
    const bottom = at(x0, y0 + 1) * (1 - ax) + at(x0 + 1, y0 + 1) * ax
    out[c] = top * (1 - ay) + bottom * ay
  }
}

// Stage 4: fill both triangles into a float RGB buffer, pixel centres at x + 0.5.
export function rasterise(v: Vertex[], w: number, h: number, tex: Uint8ClampedArray,
                          mapping: Mapping, filter: Filter): Float32Array {
  const out = new Float32Array(w * h * 3)
  for (let i = 0; i < w * h; i++) { out[i * 3] = 16; out[i * 3 + 1] = 32; out[i * 3 + 2] = 42 }
  const colour = [0, 0, 0]
  for (const [a, b, c] of [[v[0], v[1], v[2]], [v[0], v[2], v[3]]]) {
    const area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    if (Math.abs(area) < 1e-6) continue
    const bary = (px: number, py: number) => {
      const w0 = ((b.x - px) * (c.y - py) - (b.y - py) * (c.x - px)) / area
      const w1 = ((c.x - px) * (a.y - py) - (c.y - py) * (a.x - px)) / area
      return [w0, w1, 1 - w0 - w1]
    }
    const exact = (px: number, py: number) => {
      const [w0, w1, w2] = bary(px, py)
      const oow = w0 * a.oow + w1 * b.oow + w2 * c.oow
      return [(w0 * a.sow + w1 * b.sow + w2 * c.sow) / oow, (w0 * a.tow + w1 * b.tow + w2 * c.tow) / oow]
    }
    const x0 = Math.max(0, Math.floor(Math.min(a.x, b.x, c.x))), x1 = Math.min(w - 1, Math.ceil(Math.max(a.x, b.x, c.x)))
    const y0 = Math.max(0, Math.floor(Math.min(a.y, b.y, c.y))), y1 = Math.min(h - 1, Math.ceil(Math.max(a.y, b.y, c.y)))
    for (let y = y0; y <= y1; y++) {
      const py = y + 0.5
      for (let x = x0; x <= x1; x++) {
        const px = x + 0.5
        const [w0, w1, w2] = bary(px, py)
        if (w0 < 0 || w1 < 0 || w2 < 0) continue
        let s: number, t: number
        if (mapping === 'perspective') {
          [s, t] = exact(px, py)
        } else if (mapping === 'affine') {
          s = w0 * a.sow / a.oow + w1 * b.sow / b.oow + w2 * c.sow / c.oow
          t = w0 * a.tow / a.oow + w1 * b.tow / b.oow + w2 * c.tow / c.oow
        } else {
          const xa = Math.floor(x / 16) * 16 + 0.5, f = (px - xa) / 16
          const [sa, ta] = exact(xa, py), [sb, tb] = exact(xa + 16, py)
          s = sa + (sb - sa) * f
          t = ta + (tb - ta) * f
        }
        sample(tex, s, t, filter, colour)
        const i = (y * w + x) * 3
        out[i] = colour[0]; out[i + 1] = colour[1]; out[i + 2] = colour[2]
      }
    }
  }
  return out
}

const DITHER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]

// Stage 5: what the device stores for each pixel, shown as the colour the monitor got.
export function store(rgb: Float32Array, w: number, h: number, kind: Store): Uint8ClampedArray {
  const out = new Uint8ClampedArray(w * h * 4)
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const i = y * w + x
      let r = rgb[i * 3], g = rgb[i * 3 + 1], b = rgb[i * 3 + 2]
      if (kind === 'palette8') {
        // A 6 x 6 x 6 colour cube, the kind of fixed palette a 256-colour renderer shared.
        r = Math.round(r / 51) * 51; g = Math.round(g / 51) * 51; b = Math.round(b / 51) * 51
      } else if (kind === 'rgb565') {
        const d = (DITHER[y & 3][x & 3] + 0.5) / 16
        const r5 = Math.min(31, Math.floor(r * 31 / 255 + d)), g6 = Math.min(63, Math.floor(g * 63 / 255 + d))
        const b5 = Math.min(31, Math.floor(b * 31 / 255 + d))
        r = (r5 << 3) | (r5 >> 2); g = (g6 << 2) | (g6 >> 4); b = (b5 << 3) | (b5 >> 2)
      }
      out[i * 4] = r; out[i * 4 + 1] = g; out[i * 4 + 2] = b; out[i * 4 + 3] = 255
    }
  }
  return out
}

// Compare a device's picture with the reference. A larger picture is averaged down first.
// Returns the pixels that differ by more than `tolerance` in any channel, and a picture of them.
export function difference(ref: Uint8ClampedArray, other: Uint8ClampedArray, w: number, h: number,
                           scale: number, tolerance: number): { count: number; image: Uint8ClampedArray } {
  const image = new Uint8ClampedArray(w * h * 4)
  let count = 0
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const sum = [0, 0, 0]
      for (let dy = 0; dy < scale; dy++) {
        for (let dx = 0; dx < scale; dx++) {
          const j = ((y * scale + dy) * w * scale + x * scale + dx) * 4
          sum[0] += other[j]; sum[1] += other[j + 1]; sum[2] += other[j + 2]
        }
      }
      const i = (y * w + x) * 4, n = scale * scale
      const most = Math.max(...[0, 1, 2].map((c) => Math.abs(sum[c] / n - ref[i + c])))
      const off = most > tolerance
      if (off) count++
      const v = off ? Math.min(255, 80 + most * 2) : 0
      image[i] = v; image[i + 1] = off ? v * 0.6 : 18; image[i + 2] = off ? 0 : 24; image[i + 3] = 255
    }
  }
  return { count, image }
}
