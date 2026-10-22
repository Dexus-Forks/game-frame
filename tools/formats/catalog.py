#!/usr/bin/env python3
"""A first resource viewer: every file of a game, what it is, and what the project's decoders
make of it, as a static HTML page for people and a JSON manifest for agents.

Usage: catalog.py GAME_DIR OUT_DIR [--decoder GLOB=DECODER.py ...] [--palette FILE]
                  [--palette-offset N] [--hex N]

  GAME_DIR              the game's files (walked recursively)
  OUT_DIR               where index.html, catalog.json and the pictures go. It holds game
                        material: keep it in an ignored folder (derived/catalog), never in git
  --decoder GLOB=FILE   run a decoder (the validate.py kind: `decode(reader)`) on every file
                        whose path relative to GAME_DIR matches GLOB (fnmatch, case-blind).
                        Repeat it for each format. The first matching decoder is used
  --palette FILE        256 RGB entries for palette-indexed pictures (6-bit VGA values are
                        scaled up when none is above 63); --palette-offset N reads it from
                        inside a bigger file. Without it, the default VGA palette
  --hex N               bytes of each file shown in hex (default 256)

A decoder may also define `images(result)`, returning a list of pictures to show:
`(label, width, height, pixels)` with `pixels` one palette index per byte, row by row, or
`(label, width, height, pixels, 'rgb')` with three bytes per pixel. Each becomes a PNG on the
page. Use it to look at what a decoder found: a wrong width, offset or palette is obvious in a
picture and hidden in a table.

Prints one summary line, also in catalog.json: how many files and bytes decode completely,
how many fail, and how many have no decoder yet. That line is the format coverage of the
project; re-run the catalogue after each decoder to see it grow (docs/working/viewer.md).

It does not play sound, draw 3D or run the game: that is the project's own viewer
(re-browse skill). Standard library only, plus the toolkit's reader, filetypes and rawimage.
"""
import fnmatch
import html
import importlib.util
import json
import os
import pprint
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, '..', 'triage'))
from rawimage import default_vga, png  # noqa: E402
from reader import FormatError, Reader  # noqa: E402
import filetypes  # noqa: E402

MAX_PICTURES = 64
MAX_PREVIEW = 4000


def load_decoder(path):
    spec = importlib.util.spec_from_file_location(os.path.splitext(os.path.basename(path))[0], path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def load_palette(path, offset):
    data = open(path, 'rb').read()[offset:offset + 768]
    if len(data) < 768:
        sys.exit(f'{path}: fewer than 768 palette bytes at {offset}')
    scale = 255 / 63 if max(data) <= 63 else 1
    return [tuple(int(v * scale) for v in data[i * 3:i * 3 + 3]) for i in range(256)]


def hexdump(data):
    lines = []
    for at in range(0, len(data), 16):
        row = data[at:at + 16]
        text = ''.join(chr(b) if 32 <= b < 127 else '.' for b in row)
        lines.append(f'{at:06x}  {row.hex(" "):<48}  {text}')
    return '\n'.join(lines)


def run_decoder(mod, data, name):
    r = Reader(data, name=name)
    try:
        result = mod.decode(r)
        r.done()
        return 'complete', result, ''
    except (FormatError, ValueError, IndexError, UnicodeDecodeError, KeyError) as e:
        return 'failed', None, f'{e} (stopped at {r.pos:#x} of {len(data):#x})'


def write_pictures(mod, result, palette, out_dir, stem):
    """PNG files for the decoder's `images(result)`, as (label, file name) pairs."""
    if not hasattr(mod, 'images'):
        return [], ''
    try:
        pics = list(mod.images(result))
    except Exception as e:  # a viewer must show the error, not stop
        return [], f'images(): {e}'
    out = []
    for k, pic in enumerate(pics[:MAX_PICTURES]):
        label, w, h, pixels = pic[:4]
        rgb = len(pic) > 4 and pic[4] == 'rgb'
        if w <= 0 or h <= 0 or len(pixels) < w * h * (3 if rgb else 1):
            out.append((f'{label}: {w}x{h} does not fit {len(pixels)} bytes', None))
            continue
        if rgb:
            rows = [bytes(pixels[y * w * 3:(y + 1) * w * 3]) for y in range(h)]
        else:
            rows = [b''.join(bytes(palette[p]) for p in pixels[y * w:(y + 1) * w]) for y in range(h)]
        name = f'{stem}-{k}.png'
        png(os.path.join(out_dir, name), w, h, rows)
        out.append((f'{label} ({w}x{h})', name))
    note = f'{len(pics) - MAX_PICTURES} more not shown' if len(pics) > MAX_PICTURES else ''
    return out, note


PAGE = """<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1"><title>Game catalogue</title>
<style>
:root {{ color-scheme: light dark; --bg:#fff; --fg:#1d1f22; --muted:#666; --ok:#1a7f37; --bad:#b42318; --line:#ddd }}
@media (prefers-color-scheme: dark) {{ :root {{ --bg:#15171a; --fg:#e6e1d6; --muted:#9a958a; --ok:#5fd17a; --bad:#ff8a80; --line:#333 }} }}
body {{ background:var(--bg); color:var(--fg); font:14px/1.45 system-ui,sans-serif; margin:16px; }}
table {{ border-collapse:collapse; width:100%; }} td,th {{ border-bottom:1px solid var(--line); padding:3px 6px; text-align:left; }}
.complete {{ color:var(--ok) }} .failed {{ color:var(--bad) }} .muted {{ color:var(--muted) }}
pre {{ overflow:auto; font-size:12px; }} img {{ image-rendering:pixelated; min-width:64px; margin:4px; border:1px solid var(--line) }}
figure {{ display:inline-block; margin:4px; }} figcaption {{ font-size:11px; color:var(--muted) }}
</style></head><body>
<h1>Game catalogue</h1>
<p>{summary}</p>
<table><tr><th>File</th><th>Bytes</th><th>Kind (from bytes)</th><th>Entropy</th><th>Decoder</th></tr>
{rows}</table>
{sections}
</body></html>
"""


def main(argv):
    if len(argv) < 2 or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    opts, pos, i = {'--decoder': []}, [], 0
    while i < len(argv):
        a = argv[i]
        if a == '--decoder':
            opts['--decoder'].append(argv[i + 1])
            i += 2
        elif a in ('--palette', '--palette-offset', '--hex'):
            opts[a] = argv[i + 1]
            i += 2
        else:
            pos.append(a)
            i += 1
    game, out_dir = pos[0], pos[1]
    os.makedirs(out_dir, exist_ok=True)
    decoders = []
    for spec in opts['--decoder']:
        glob, _, path = spec.partition('=')
        decoders.append((glob.lower(), path, load_decoder(path)))
    palette = (load_palette(opts['--palette'], int(opts.get('--palette-offset', '0'), 0))
               if '--palette' in opts else default_vga())
    hex_bytes = int(opts.get('--hex', '256'))

    files, rows, sections = [], [], []
    for n, (path, root) in enumerate(filetypes.walk([game])):
        rel = os.path.relpath(path, root)
        data = open(path, 'rb').read()
        kind = filetypes.kind_of(data[:filetypes.HEAD], len(data))
        ent = filetypes.entropy(filetypes.sample(path, len(data)))
        entry = {'path': rel, 'size': len(data), 'kind': kind, 'entropy': round(ent, 2),
                 'note': filetypes.verdict(kind, ent), 'decoder': None, 'status': 'no decoder'}
        match = next(((g, p, m) for g, p, m in decoders if fnmatch.fnmatch(rel.lower(), g)), None)
        body = [f'<pre>{html.escape(hexdump(data[:hex_bytes]))}</pre>']
        if match:
            status, result, err = run_decoder(match[2], data, rel)
            entry.update(decoder=match[1], status=status, error=err)
            if status == 'complete':
                text = pprint.pformat(result, width=110)
                more = f'\n… ({len(text) - MAX_PREVIEW} more characters)' if len(text) > MAX_PREVIEW else ''
                body.insert(0, f'<pre>{html.escape(text[:MAX_PREVIEW] + more)}</pre>')
                pics, note = write_pictures(match[2], result, palette, out_dir, f'f{n}')
                entry['pictures'] = len(pics)
                figs = ''.join(f'<figure><img src="{html.escape(f)}" alt="{html.escape(l)}"><figcaption>{html.escape(l)}</figcaption></figure>'
                               if f else f'<p class="failed">{html.escape(l)}</p>' for l, f in pics)
                if figs or note:
                    body.insert(0, figs + (f'<p class="muted">{html.escape(note)}</p>' if note else ''))
            else:
                body.insert(0, f'<p class="failed">{html.escape(err)}</p>')
        files.append(entry)
        anchor = f'f{n}'
        cls = entry['status'].replace(' ', '-')
        rows.append(f'<tr><td><a href="#{anchor}">{html.escape(rel)}</a></td><td>{len(data):,}</td>'
                    f'<td>{html.escape(kind)}{" · " + html.escape(entry["note"]) if entry["note"] else ""}</td>'
                    f'<td>{ent:.2f}</td><td class="{cls}">{html.escape(entry["status"])}</td></tr>')
        sections.append(f'<h2 id="{anchor}">{html.escape(rel)}</h2>' + ''.join(body))

    done = [f for f in files if f['status'] == 'complete']
    failed = [f for f in files if f['status'] == 'failed']
    total = sum(f['size'] for f in files) or 1
    covered = sum(f['size'] for f in done)
    summary = (f'{len(done)} of {len(files)} files decode completely '
               f'({covered * 100 // total}% of {total:,} bytes); {len(failed)} fail; '
               f'{len(files) - len(done) - len(failed)} have no decoder')
    with open(os.path.join(out_dir, 'catalog.json'), 'w') as f:
        json.dump({'summary': summary, 'files': files}, f, indent=2)
    with open(os.path.join(out_dir, 'index.html'), 'w') as f:
        f.write(PAGE.format(summary=html.escape(summary), rows='\n'.join(rows), sections='\n'.join(sections)))
    print(summary)
    for f in failed:
        print(f'FAIL {f["error"]}')
    print(f'wrote {os.path.join(out_dir, "index.html")} and catalog.json')
    return 1 if failed else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
