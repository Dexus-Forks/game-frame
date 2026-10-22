// Records and packets for the records explorer (/lab/records). The same rules as
// tools/formats/fields.py (what each offset holds) and tools/formats/pcap.py (flows in a
// capture). Runs in the reader's browser only.

export type Note = { kind: 'constant' | 'counter' | 'length' | 'type' | 'varies', text: string }

/** Likely record sizes in a file: distances at which bytes repeat most often. */
export function strides(bytes: Uint8Array, max = 512, sample = 65536): { size: number, score: number }[] {
  const n = Math.min(bytes.length, sample)
  const out: { size: number, score: number }[] = []
  for (let k = 2; k <= Math.min(max, n >> 2); k++) {
    let same = 0
    for (let i = 0; i + k < n; i++) if (bytes[i] === bytes[i + k]) same++
    out.push({ size: k, score: same / (n - k) })
  }
  // A record size also scores at each of its multiples. Credit each size with the best score of
  // its multiples, then prefer the smallest size that comes close to the best of all.
  const score = new Map(out.map(o => [o.size, o.score]))
  const best = Math.max(...out.map(o => o.score))
  const ranked = out.map(o => {
    let m = 0
    for (let k = o.size; k <= max; k += o.size) m = Math.max(m, score.get(k) ?? 0)
    return { size: o.size, score: o.score, family: m }
  })
  ranked.sort((a, b) => {
    const ga = a.score >= best * 0.9 ? 1 : 0, gb = b.score >= best * 0.9 ? 1 : 0
    return gb - ga || (ga ? a.size - b.size : b.score - a.score)
  })
  // A size that divides the leader and scores nearly as well is the record; the leader is then
  // a run of records with a repeating field (a type that cycles, say).
  const lead = ranked[0]
  if (lead) {
    const divisor = out.filter(o => lead.size % o.size === 0 && o.size < lead.size && o.score >= lead.score * 0.8)
      .sort((a, b) => a.size - b.size).find(o => o.size >= 4)
    if (divisor) ranked.unshift({ size: divisor.size, score: divisor.score, family: divisor.score })
  }
  const seen = new Set<number>()
  return ranked.filter(r => !seen.has(r.size) && seen.add(r.size)).slice(0, 8).map(({ size, score }) => ({ size, score }))
}

function u(row: Uint8Array, at: number, size: number, le: boolean): number {
  let v = 0
  for (let i = 0; i < size; i++) v += row[at + (le ? i : size - 1 - i)] * 2 ** (8 * i)
  return v
}

/** What each offset that every row has looks like. */
export function classify(rows: Uint8Array[]): Note[][] {
  const width = Math.min(...rows.map(r => r.length))
  const out: Note[][] = []
  for (let at = 0; at < width; at++) {
    const notes: Note[] = []
    const col = rows.map(r => r[at])
    const distinct = new Set(col).size
    if (distinct === 1) notes.push({ kind: 'constant', text: `constant 0x${col[0].toString(16).padStart(2, '0')}` })
    for (const size of [2, 4]) {
      if (at + size > width) continue
      for (const le of [true, false]) {
        const vals = rows.map(r => u(r, at, size, le))
        const high = Array.from({ length: size - 1 }, (_, k) => le ? at + 1 + k : at + k)
        const plausible = high.every(h => new Set(rows.map(r => r[h])).size > 1 || rows[0][h] === 0)
        const step = vals.length > 1 ? vals[1] - vals[0] : 0
        if (plausible && rows.length > 2 && step > 0 && step % 256 && vals.every((v, i) => i === 0 || v - vals[i - 1] === step)) {
          notes.push({ kind: 'counter', text: `counter u${size * 8} ${le ? 'LE' : 'BE'} (+${step})` })
        }
        if (new Set(vals).size > 1 && vals.every((v, i) => v === rows[i].length)) {
          notes.push({ kind: 'length', text: `length u${size * 8} ${le ? 'LE' : 'BE'} (whole)` })
        } else if (new Set(vals).size > 1 && vals.every((v, i) => v === rows[i].length - at - size)) {
          notes.push({ kind: 'length', text: `length u${size * 8} ${le ? 'LE' : 'BE'} (rest)` })
        }
      }
    }
    const step = col.length > 1 ? col[1] - col[0] : 0
    if (rows.length > 2 && step > 0 && col.every((v, i) => i === 0 || v - col[i - 1] === step) && !notes.some(n => n.kind === 'counter')) {
      notes.push({ kind: 'counter', text: `counter u8 (+${step})` })
    }
    if (!notes.length) {
      if (distinct > 1 && distinct <= 8 && rows.length >= 8) {
        notes.push({ kind: 'type', text: `${distinct} values: ${[...new Set(col)].sort((a, b) => a - b).map(v => '0x' + v.toString(16)).join(', ')}` })
      } else notes.push({ kind: 'varies', text: `varies (${distinct} values)` })
    }
    out.push(notes)
  }
  return out
}

/** A colour wheel for byte values: neighbouring values get neighbouring colours. */
export function wheel(v: number): [number, number, number] {
  const h = (v / 256) * 6, k = Math.floor(h), f = h - k
  const q = Math.round(255 * (1 - f)), t = Math.round(255 * f)
  return [[255, q, 0, 0, t, 255][k], [t, 255, 255, q, 0, 0][k], [0, 0, t, 255, 255, q][k]] as [number, number, number]
}

export type Flow = { key: string, label: string, a: Uint8Array[], b: Uint8Array[] }

/** UDP and TCP payloads by flow, from a pcap or pcapng capture. */
export function flowsFrom(bytes: Uint8Array): Flow[] | null {
  const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
  const magic = dv.getUint32(0, false)
  const frames: { link: number, data: Uint8Array }[] = []
  if (magic === 0xd4c3b2a1 || magic === 0xa1b2c3d4 || magic === 0x4d3cb2a1 || magic === 0xa1b23c4d) {
    const le = magic === 0xd4c3b2a1 || magic === 0x4d3cb2a1
    const link = dv.getUint32(20, le)
    for (let at = 24; at + 16 <= bytes.length;) {
      const incl = dv.getUint32(at + 8, le)
      frames.push({ link, data: bytes.subarray(at + 16, at + 16 + incl) })
      at += 16 + incl
    }
  } else if (magic === 0x0a0d0d0a) {
    const le = dv.getUint32(8, true) === 0x1a2b3c4d
    const links: number[] = []
    for (let at = 0; at + 12 <= bytes.length;) {
      const kind = dv.getUint32(at, le), len = dv.getUint32(at + 4, le)
      if (len < 12) break
      if (kind === 1) links.push(dv.getUint16(at + 8, le))
      else if (kind === 6) {
        const iface = dv.getUint32(at + 8, le), cap = dv.getUint32(at + 20, le)
        frames.push({ link: links[iface] ?? 1, data: bytes.subarray(at + 28, at + 28 + cap) })
      }
      at += len
    }
  } else return null
  const flows = new Map<string, Flow>()
  for (const { link, data } of frames) {
    const fv = new DataView(data.buffer, data.byteOffset, data.byteLength)
    let at = 0, kind = 0
    if (link === 1) { kind = fv.getUint16(12); at = 14 } else if (link === 113) { kind = fv.getUint16(14); at = 16 }
    else if (link === 0 || link === 108) { kind = 0x0800; at = 4 } else if (link === 101) { kind = 0x0800; at = 0 } else continue
    if (kind !== 0x0800 || at + 20 > data.length) continue
    const ihl = (data[at] & 15) * 4, total = fv.getUint16(at + 2), proto = data[at + 9]
    const src = Array.from(data.subarray(at + 12, at + 16)).join('.'), dst = Array.from(data.subarray(at + 16, at + 20)).join('.')
    const body = data.subarray(at + ihl, at + total)
    if (body.length < 8) continue
    const bv = new DataView(body.buffer, body.byteOffset, body.byteLength)
    const sport = bv.getUint16(0), dport = bv.getUint16(2)
    const payload = proto === 17 ? body.subarray(8) : proto === 6 ? body.subarray((body[12] >> 4) * 4) : null
    if (!payload || !payload.length) continue
    const a = `${src}:${sport}`, b = `${dst}:${dport}`
    const [lo, hi] = a < b ? [a, b] : [b, a]
    const key = `${proto === 17 ? 'UDP' : 'TCP'} ${lo} <-> ${hi}`
    if (!flows.has(key)) flows.set(key, { key, label: key, a: [], b: [] })
    ;(a === lo ? flows.get(key)!.a : flows.get(key)!.b).push(payload)
  }
  return [...flows.values()]
}
