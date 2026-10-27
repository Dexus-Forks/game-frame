#!/usr/bin/env python3
"""REA on a game's binary from a script, with the project's labels and a saved snapshot.

    rea_session.py snapshot BINARY [ADDR ...] [--file F] [--fresh]
    rea_session.py fn BINARY ADDR ... [--file F] [--snapshot-only] [--json]
    rea_session.py labels BINARY [--chunk N]

Common options: --labels FILE (repeatable; the first file wins for an address), --out DIR
(default analysis/rea in the project), --server CMD (default: the `rea` server of the
project's .mcp.json; enable it with ./gameframe mcp --enable rea).

`snapshot` opens BINARY with REA's Ghidra provider (from the old snapshot, unless --fresh), asks
for the dossier (`analyze_function`) of each address, and saves REA's analysis snapshot to
OUT/<binary>.snapshot.json, so the answers add up over runs.

`fn` opens BINARY from that snapshot when there is one, gives REA the project's labels with
`set_addresses_names`, and asks for each address's dossier. It writes each dossier to
OUT/fn/<binary>/<address>.json and prints a short summary with the labels: the function's size,
what refers to it (a vtable slot shows as data), what it calls and its memory reads and writes.
`--snapshot-only` leaves REA's names alone, so warmed functions come from the snapshot.

`labels` writes the labels as `set_addresses_names` input, in chunks, to
OUT/<binary>-names-N.json, for an agent that wants them in its own REA session.

The labels default to labels/names.tsv, labels/names-from-port.tsv and labels/library.tsv in
the project, those that exist, in that order. Only addresses inside BINARY's image (its PE
ImageBase and SizeOfImage) are used, so one labels file can cover several binaries. Ghidra
reads `::` as a namespace, so `::` becomes `__`; a name used twice gets its address appended.

REA 6.7.0, measured on a 2 MB x64 game: the first question of a session waits about 50 s while
Ghidra imports and analyses the binary. A question asked before, in the session or in the
snapshot, takes 0.1 s. A snapshot holds answers to questions, not Ghidra's database: a new
question costs the 50 s again. Each dossier adds 2 to 3 MB to it. Renaming in REA makes the
snapshot's answers stale, and a session with renames cannot save a snapshot. So `snapshot`
renames nothing, and `fn` renames after it opens. Everything this writes describes the game's
code: keep OUT in an ignored folder.
"""
import argparse
import json
import os
import re
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from mcp_client import McpClient, project_root, provider_command  # noqa: E402

LABEL_FILES = ('names.tsv', 'names-from-port.tsv', 'library.tsv')
CHUNK = 400


def address(text):
    """`0x1400be770`, `1400be770` or `FUN_1400be770` as REA's lowercase `0x` form."""
    text = text.strip().lower().removeprefix('fun_').removeprefix('0x')
    if not re.fullmatch(r'[0-9a-f]+', text):
        raise SystemExit(f'not an address: {text}')
    return '0x' + text


def image_range(binary):
    """(first, end) addresses of a PE image, or None for other files."""
    try:
        import pefile
        pe = pefile.PE(binary, fast_load=True)
    except Exception:
        return None
    base = pe.OPTIONAL_HEADER.ImageBase
    return base, base + pe.OPTIONAL_HEADER.SizeOfImage


def ghidra_name(name):
    return re.sub(r'\s+', '_', name.replace('::', '__'))


def label_files(args):
    if args.labels:
        return args.labels
    folder = os.path.join(project_root(), 'labels')
    return [os.path.join(folder, f) for f in LABEL_FILES if os.path.exists(os.path.join(folder, f))]


def load_labels(binary, files):
    """{address: name} for the addresses inside BINARY, the first file winning."""
    span = image_range(binary)
    names = {}
    for path in files:
        for line in open(path, encoding='utf-8'):
            if line.startswith('#') or not line.strip():
                continue
            cols = line.rstrip('\n').split('\t')
            if len(cols) < 2:
                continue
            try:
                addr = address(cols[0])
            except SystemExit:
                continue
            if span and not span[0] <= int(addr, 16) < span[1]:
                continue
            names.setdefault(addr, ghidra_name(cols[1]))
    users = {}
    for addr, name in names.items():
        users.setdefault(name, []).append(addr)
    for name, addrs in users.items():
        if len(addrs) > 1:
            for addr in addrs:
                names[addr] = f'{name}_{addr[2:]}'
    return names


def chunks(names, size):
    items = sorted(names.items())
    return [dict(items[i:i + size]) for i in range(0, len(items), size)]


def apply_labels(rea, names, log):
    applied, refused = 0, []
    for part in chunks(names, CHUNK):
        result = rea.tool('set_addresses_names', names=part)
        # normalized_result maps each address to whether its name stuck.
        for addr, ok in (result.get('normalized_result') or {}).items():
            applied += bool(ok)
            if not ok:
                refused.append(addr)
    log(f'labels: {applied} of {len(names)} applied' +
        (f'; refused {", ".join(refused[:10])}' if refused else ''))


def snapshot_path(args):
    return os.path.join(args.out, os.path.basename(args.binary) + '.snapshot.json')


def open_binary(rea, args, use_snapshot, log):
    params = {'path': args.binary, 'provider_id': 'ghidra'}
    snap = snapshot_path(args)
    if use_snapshot and os.path.exists(snap):
        params['snapshot_path'] = snap
        log(f'opening {os.path.basename(args.binary)} from {snap}')
    else:
        log(f'opening {os.path.basename(args.binary)} (the first question waits for Ghidra)')
    info = rea.tool('open_binary', **params)
    log(f'  sha256 {str(info.get("sha256", "?"))[:16]}')


def addresses(args):
    found = [address(a) for a in getattr(args, 'addresses', None) or []]
    for path in getattr(args, 'files', None) or []:
        for line in open(path, encoding='utf-8'):
            word = line.split('#', 1)[0].split()
            if word:
                found.append(address(word[0]))
    return list(dict.fromkeys(found))


def client(args, log):
    return McpClient(provider_command('rea', override=args.server), log=log)


def cmd_snapshot(args, log):
    wanted = addresses(args)
    os.makedirs(args.out, exist_ok=True)
    with client(args, log) as rea:
        open_binary(rea, args, not args.fresh, log)
        for addr in wanted:
            start = time.monotonic()
            rea.tool('analyze_function', procedure=addr)
            log(f'  {addr} in {time.monotonic() - start:.1f} s')
        rea.tool('close_binary', snapshot_path=snapshot_path(args), overwrite=True)
    log(f'wrote {snapshot_path(args)} ({len(wanted)} new dossiers)')


def cmd_fn(args, log):
    wanted = addresses(args)
    if not wanted:
        raise SystemExit('fn: name at least one address')
    names = load_labels(args.binary, label_files(args))
    outdir = os.path.join(args.out, 'fn', os.path.basename(args.binary))
    os.makedirs(outdir, exist_ok=True)
    with client(args, log) as rea:
        open_binary(rea, args, True, log)
        if not args.snapshot_only and names:
            apply_labels(rea, names, log)
        for addr in wanted:
            dossier = rea.tool('analyze_function', procedure=addr)
            path = os.path.join(outdir, addr[2:] + '.json')
            with open(path, 'w', encoding='utf-8') as f:
                json.dump(dossier, f)
            s = summary(dossier, names)
            print(json.dumps(s, indent=1) if args.json else render(s, path))
        rea.tool('close_binary')


def cmd_labels(args, log):
    os.makedirs(args.out, exist_ok=True)
    names = load_labels(args.binary, label_files(args))
    stem = os.path.join(args.out, os.path.basename(args.binary) + '-names')
    parts = chunks(names, args.chunk)
    for i, part in enumerate(parts, 1):
        with open(f'{stem}-{i}.json', 'w', encoding='utf-8') as f:
            json.dump({'names': part}, f, indent=0)
    log(f'{len(names)} labels in {len(parts)} files: {stem}-N.json')


def _name(ref, names):
    if not ref:
        return '?'
    addr = (ref.get('address') or '').lower()
    return names.get(addr) or ref.get('name') or addr


def summary(dossier, names):
    """The parts of a dossier a porter reads first, with the project's labels."""
    result = dossier.get('normalized_result', dossier)
    proc = result.get('procedure') or {}
    incoming = {}
    for ref in result.get('incoming_references') or []:
        source = ref.get('source_procedure')
        key = _name(source, names) if source else f'data {ref.get("source_address")}'
        incoming[key] = incoming.get(key, 0) + 1
    flow = result.get('native_value_flow') or {}
    effects = {}
    for effect in flow.get('effects') or []:
        effects[effect.get('kind', '?')] = effects.get(effect.get('kind', '?'), 0) + 1
    return {
        'address': proc.get('address'),
        'name': _name(proc, names),
        'bytes': (proc.get('body') or {}).get('total_bytes'),
        'blocks': len(result.get('basic_blocks') or []),
        'referred_from': incoming,
        'calls': sorted({_name(c, names) for c in result.get('callees') or []}),
        'memory_effects': effects,
        'evidence_id': dossier.get('evidence_id'),
    }


def render(s, path):
    refs = ', '.join(f'{k} x{v}' if v > 1 else k for k, v in s['referred_from'].items())
    effects = ', '.join(f'{k} {v}' for k, v in sorted(s['memory_effects'].items()))
    return '\n'.join([
        f"{s['address']}  {s['name']}  ({s['bytes']} bytes, {s['blocks']} blocks)",
        f"  referred from: {refs or 'nothing REA resolved'}",
        f"  calls: {', '.join(s['calls']) or 'nothing'}",
        f"  memory effects: {effects or 'none reported'}",
        f"  evidence {s['evidence_id']}",
        f"  dossier {path}",
    ])


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    sub = parser.add_subparsers(dest='command', required=True)
    for name in ('snapshot', 'fn', 'labels'):
        p = sub.add_parser(name)
        p.add_argument('binary')
        p.add_argument('--labels', action='append', help='a labels TSV (address, name)')
        p.add_argument('--out', default=os.path.join(project_root(), 'analysis', 'rea'))
        p.add_argument('--server', help='the REA server command (default: from .mcp.json)')
        if name == 'labels':
            p.add_argument('--chunk', type=int, default=CHUNK)
            continue
        p.add_argument('addresses', nargs='*')
        p.add_argument('--file', dest='files', action='append', help='one address per line')
        if name == 'snapshot':
            p.add_argument('--fresh', action='store_true', help='leave the old snapshot out')
        else:
            p.add_argument('--snapshot-only', action='store_true',
                           help='do not rename in REA, so answers come from the snapshot')
            p.add_argument('--json', action='store_true', help='print the summaries as JSON')
    args = parser.parse_args(argv)
    args.binary = os.path.realpath(args.binary)
    args.out = os.path.abspath(args.out)

    def log(text):
        print(text, file=sys.stderr, flush=True)

    {'snapshot': cmd_snapshot, 'fn': cmd_fn, 'labels': cmd_labels}[args.command](args, log)


if __name__ == '__main__':
    main()
