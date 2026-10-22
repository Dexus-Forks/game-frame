// The byte maths of the browser tools (/lab/). The same rules as the Python tools:
// tools/triage/binview.py (byte classes, entropy, Hilbert curve), tools/triage/filetypes.py
// (signatures) and tools/formats/rawimage.py (pixel modes and palettes). Everything runs in
// the reader's browser; no file leaves it.

export type RGB = [number, number, number]

export const CLASS_COLOURS: Record<string, RGB> = {
  zero: [0, 0, 0], ff: [255, 255, 255], text: [55, 126, 220], low: [77, 175, 74], high: [228, 26, 28],
}

export function kindOf(b: number): string {
  if (b === 0) return 'zero'
  if (b === 0xff) return 'ff'
  if ((b >= 32 && b < 127) || b === 9 || b === 10 || b === 13) return 'text'
  if (b < 32 || b === 127) return 'low'
  return 'high'
}

export function entropy(bytes: Uint8Array, start = 0, end = bytes.length): number {
  const counts = new Uint32Array(256)
  for (let i = start; i < end; i++) counts[bytes[i]]++
  const n = end - start
  if (n <= 0) return 0
  let h = 0
  for (const c of counts) if (c) h -= (c / n) * Math.log2(c / n)
  return h
}

/** 0..1 to a colour from dark blue (ordered) to red (compressed or encrypted). */
export function ramp(x: number): RGB {
  const stops: [number, RGB][] = [[0, [20, 20, 90]], [0.35, [30, 160, 200]], [0.6, [60, 180, 60]],
    [0.85, [240, 220, 40]], [1, [220, 30, 30]]]
  for (let i = 0; i < stops.length - 1; i++) {
    const [a, ca] = stops[i], [b, cb] = stops[i + 1]
    if (x <= b) {
      const t = b > a ? (x - a) / (b - a) : 0
      return [0, 1, 2].map(k => Math.round(ca[k] + (cb[k] - ca[k]) * t)) as RGB
    }
  }
  return stops[stops.length - 1][1]
}

/** Point d along a Hilbert curve of side 2^order. */
export function hilbert(d: number, order: number): [number, number] {
  let x = 0, y = 0, t = d
  for (let s = 1; s < (1 << order); s *= 2) {
    const rx = 1 & (t >> 1)
    const ry = 1 & (t ^ rx)
    if (ry === 0) {
      if (rx === 1) { x = s - 1 - x; y = s - 1 - y }
      ;[x, y] = [y, x]
    }
    x += s * rx
    y += s * ry
    t >>= 2
  }
  return [x, y]
}

const SIGNATURES: [number, string, string][] = [
  [0, 'MZ', 'executable (MZ: DOS or Windows)'], [0, '\x7fELF', 'executable (ELF)'],
  [0, 'PK\x03\x04', 'ZIP archive'], [0, 'Rar!', 'RAR archive'], [0, '7z\xbc\xaf', '7-Zip archive'],
  [0, 'MSCF', 'Microsoft cabinet'], [0, '\x1f\x8b', 'gzip'], [0, 'BZh', 'bzip2'],
  [0, 'IWAD', 'Doom WAD'], [0, 'PWAD', 'Doom WAD'], [0, 'PACK', 'Quake PAK'], [0, 'KenSilverman', 'Build GRP'],
  [0, 'MPQ\x1a', 'Blizzard MPQ'], [0, 'RIFF', 'RIFF container (WAVE, AVI)'], [0, 'FORM', 'IFF container'],
  [0, 'Creative Voice File', 'Creative VOC sound'], [0, 'MThd', 'MIDI'], [0, 'OggS', 'Ogg'],
  [0, 'ID3', 'MP3'], [0, '\x89PNG', 'PNG'], [0, 'GIF8', 'GIF'], [0, '\xff\xd8\xff', 'JPEG'], [0, 'BM', 'BMP'],
  [0, 'DDS ', 'DirectDraw surface'], [0, 'FWS', 'Flash movie (SWF)'], [0, 'CWS', 'Flash movie (SWF, zlib)'],
  [0, 'ZWS', 'Flash movie (SWF, LZMA)'], [0, 'RIFX', 'Shockwave Director'], [0, 'XFIR', 'Shockwave Director'],
  [0, 'SMK2', 'Smacker video'], [0, 'SMK4', 'Smacker video'], [0, 'BIK', 'Bink video'],
  [0, 'SQLite format 3', 'SQLite database'], [0x8001, 'CD001', 'ISO 9660 image'],
  [0, '\x00\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x00', 'raw CD image (BIN)'], [1080, 'M.K.', 'ProTracker MOD'],
]

export function guessKind(bytes: Uint8Array): string {
  for (const [off, sig, name] of SIGNATURES) {
    if (off + sig.length > bytes.length) continue
    let ok = true
    for (let i = 0; i < sig.length && ok; i++) ok = bytes[off + i] === sig.charCodeAt(i)
    if (ok) return name
  }
  const n = Math.min(bytes.length, 4096)
  let text = 0
  for (let i = 0; i < n; i++) if (kindOf(bytes[i]) === 'text') text++
  return n && text / n > 0.97 ? 'text' : 'unknown: look at the byte map'
}

// Palettes, as in tools/formats/rawimage.py.
export function egaPalette(): RGB[] {
  const base = [[0, 0, 0], [0, 0, 42], [0, 42, 0], [0, 42, 42], [42, 0, 0], [42, 0, 42], [42, 21, 0], [42, 42, 42],
    [21, 21, 21], [21, 21, 63], [21, 63, 21], [21, 63, 63], [63, 21, 21], [63, 21, 63], [63, 63, 21], [63, 63, 63]]
  return base.map(c => c.map(v => Math.floor(v * 255 / 63)) as RGB)
}

export function vgaPalette(): RGB[] {
  const pal = egaPalette()
  for (const g of [0, 5, 8, 11, 14, 17, 20, 24, 28, 32, 36, 40, 45, 50, 56, 63]) pal.push([g, g, g].map(v => Math.floor(v * 255 / 63)) as RGB)
  const levels = [[0, 16, 31, 47, 63], [31, 39, 47, 55, 63], [45, 49, 54, 58, 63], [0, 7, 14, 21, 28], [14, 17, 21, 24, 28],
    [20, 22, 24, 26, 28], [0, 4, 8, 12, 16], [8, 10, 12, 14, 16], [11, 12, 13, 15, 16]]
  for (const v of levels) {
    const lo = v[0], hi = v[4]
    const ring: number[][] = []
    for (const i of [0, 1, 2, 3, 4]) ring.push([v[i], lo, hi])
    for (const i of [3, 2, 1, 0]) ring.push([hi, lo, v[i]])
    for (const i of [1, 2, 3, 4]) ring.push([hi, v[i], lo])
    for (const i of [3, 2, 1, 0]) ring.push([v[i], hi, lo])
    for (const i of [1, 2, 3, 4]) ring.push([lo, hi, v[i]])
    for (const i of [3, 2, 1]) ring.push([lo, v[i], hi])
    for (const c of ring) pal.push(c.map(x => Math.floor(x * 255 / 63)) as RGB)
  }
  while (pal.length < 256) pal.push([0, 0, 0])
  return pal.slice(0, 256)
}

export const CGA1: RGB[] = [[0, 0, 0], [85, 255, 255], [255, 85, 255], [255, 255, 255]]
export const GREYS: RGB[] = Array.from({ length: 256 }, (_, i) => [i, i, i] as RGB)

/** A palette from 768 bytes (6-bit VGA values are scaled up when none is above 63). */
export function paletteFrom(bytes: Uint8Array, offset = 0): RGB[] {
  const p = bytes.subarray(offset, offset + 768)
  const six = p.every(v => v <= 63)
  const out: RGB[] = []
  for (let i = 0; i + 2 < p.length; i += 3) out.push([0, 1, 2].map(k => six ? Math.floor(p[i + k] * 255 / 63) : p[i + k]) as RGB)
  while (out.length < 256) out.push([0, 0, 0])
  return out
}

export type Mode = 'linear8' | 'modex' | 'planar4' | 'cga2' | 'packed4' | 'packed2' | 'packed1' | 'rgb565' | 'rgb555'

/** Decode pixels into RGBA, as rawimage.py does. */
export function decode(bytes: Uint8Array, mode: Mode, width: number, height: number, pal: RGB[], start = 0): Uint8ClampedArray {
  const out = new Uint8ClampedArray(width * height * 4)
  const put = (x: number, y: number, c: RGB) => {
    const i = (y * width + x) * 4
    out[i] = c[0]; out[i + 1] = c[1]; out[i + 2] = c[2]; out[i + 3] = 255
  }
  const at = (i: number) => (i >= 0 && i < bytes.length ? bytes[i] : 0)
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      let idx = 0
      if (mode === 'linear8') idx = at(y * width + x)
      else if (mode === 'modex') {
        const plane = Math.floor(bytes.length / 4)
        idx = at((x % 4) * plane + (start + y) * (width >> 2) + (x >> 2))
      } else if (mode === 'planar4') {
        const plane = Math.floor(bytes.length / 4)
        const o = (start + y) * (width >> 3) + (x >> 3), bit = 7 - (x & 7)
        for (let p = 0; p < 4; p++) if ((at(p * plane + o) >> bit) & 1) idx |= 1 << p
      } else if (mode === 'cga2') {
        const o = (y & 1 ? 0x2000 : 0) + (y >> 1) * (width >> 2) + (x >> 2)
        idx = (at(o) >> (6 - 2 * (x & 3))) & 3
      } else if (mode === 'packed4' || mode === 'packed2' || mode === 'packed1') {
        const bits = Number(mode.slice(-1)), per = 8 / bits, stride = Math.ceil(width * bits / 8)
        idx = (at(y * stride + Math.floor(x / per)) >> (8 - bits * ((x % per) + 1))) & ((1 << bits) - 1)
      } else {
        const i = 2 * (y * width + x)
        const v = at(i) | (at(i + 1) << 8)
        const c: RGB = mode === 'rgb565'
          ? [((v >> 11) & 31) * 255 / 31, ((v >> 5) & 63) * 255 / 63, (v & 31) * 255 / 31]
          : [((v >> 10) & 31) * 255 / 31, ((v >> 5) & 31) * 255 / 31, (v & 31) * 255 / 31]
        put(x, y, c.map(Math.round) as RGB)
        continue
      }
      put(x, y, pal[idx] ?? [0, 0, 0])
    }
  }
  return out
}

export function hex(bytes: Uint8Array, offset: number, rows = 16): string[] {
  const lines: string[] = []
  for (let r = 0; r < rows; r++) {
    const at = offset + r * 16
    if (at >= bytes.length) break
    const chunk = Array.from(bytes.subarray(at, at + 16))
    const h = chunk.map(b => b.toString(16).padStart(2, '0')).join(' ')
    const a = chunk.map(b => (b >= 32 && b < 127 ? String.fromCharCode(b) : '.')).join('')
    lines.push(`${at.toString(16).padStart(8, '0')}  ${h.padEnd(47)}  ${a}`)
  }
  return lines
}
