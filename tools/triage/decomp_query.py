#!/usr/bin/env python3
"""Answer questions about a Ghidra export (scripts/ghidra-export.sh) without reading it whole.

Usage: decomp_query.py [--binary NAME] [--export DIR] COMMAND ARGS

Commands:
  fn ADDR|NAME          one function's pseudocode, with your labels applied
  callers ADDR|NAME     the functions that call or reference it
  callees ADDR|NAME     the functions it calls (with sizes, labelled ones first)
  find TEXT             the functions whose pseudocode contains TEXT (a string, a constant...)
  strings TEXT          defined strings containing TEXT, and the functions that use each
  writers OFFSET        the functions that store to `+ OFFSET` of a pointer (who sets a field?)
  biggest [N]           the N largest functions not yet labelled (where the work is)
  stats                 functions, labelled share by count and by size

ADDR is an address as Ghidra prints it (1400be770, 0x1400be770 or FUN_1400be770). NAME is a
Ghidra name or one of your labels. Labels come from labels/names.tsv (address<TAB>label<TAB>
evidence): add a row every time you identify a function, and every later answer is easier to
read. `--binary` picks a binary when the export holds several (default: the only one, or the
first .exe).
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
HEADER = re.compile(r'^// ==== ([0-9a-fA-F]+) (.*)$')


class Export:
    def __init__(self, export_dir, binary=None):
        names = sorted(os.listdir(export_dir)) if os.path.isdir(export_dir) else []
        if not names:
            sys.exit(f'no export in {export_dir}: run scripts/ghidra-export.sh first')
        if binary is None:
            exes = [n for n in names if n.lower().endswith('.exe')]
            binary = names[0] if len(names) == 1 else (exes or names)[0]
        self.dir = os.path.join(export_dir, binary)
        self.binary = binary
        self.labels = load_labels()
        self.functions = {}   # addr -> (name, size)
        for line in open(os.path.join(self.dir, 'functions.tsv'), encoding='utf-8', errors='replace'):
            cols = line.rstrip('\n').split('\t')
            self.functions[int(cols[0], 16)] = (cols[2], int(cols[1]))
        self.bodies = {}      # addr -> pseudocode
        current, buf = None, []
        for line in open(os.path.join(self.dir, 'decomp.c'), encoding='utf-8', errors='replace'):
            m = HEADER.match(line)
            if m:
                if current is not None:
                    self.bodies[current] = ''.join(buf)
                current, buf = int(m.group(1), 16), []
            else:
                buf.append(line)
        if current is not None:
            self.bodies[current] = ''.join(buf)

    def name(self, addr):
        return self.labels.get(addr) or self.functions.get(addr, (f'FUN_{addr:x}', 0))[0]

    def resolve(self, text):
        t = text.lower().removeprefix('fun_').removeprefix('0x')
        if re.fullmatch(r'[0-9a-f]+', t):
            addr = int(t, 16)
            if addr in self.functions:
                return addr
            starts = [a for a in self.functions if a <= addr]
            if starts:
                return max(starts)  # an address inside a function
        for addr, label in self.labels.items():
            if label == text:
                return addr
        for addr, (name, _) in self.functions.items():
            if name == text or name.endswith('::' + text):
                return addr
        sys.exit(f'{text}: no such function')

    def refs(self, addr):
        """The spellings by which pseudocode refers to a function."""
        name = self.functions.get(addr, ('', 0))[0]
        out = {f'FUN_{addr:x}', f'FUN_{addr:08x}'}
        if name and not name.startswith('FUN_'):
            out.add(name.split('::')[-1])
        return out

    def label(self, text):
        """Pseudocode with FUN_ names replaced by labels (kept in a comment)."""
        def sub(m):
            addr = int(m.group(1), 16)
            label = self.labels.get(addr)
            return f'{label}/*{m.group(0)}*/' if label else m.group(0)
        return re.sub(r'FUN_([0-9a-fA-F]+)', sub, text)


def load_labels():
    labels = {}
    path = os.path.join(ROOT, 'labels', 'names.tsv')
    if os.path.exists(path):
        for line in open(path, encoding='utf-8'):
            if line.startswith('#') or not line.strip():
                continue
            cols = line.rstrip('\n').split('\t')
            try:
                labels[int(cols[0].lower().removeprefix('0x'), 16)] = cols[1]
            except (ValueError, IndexError):
                pass
    return labels


def main(argv):
    export_dir = os.path.join(ROOT, 'analysis', 'export')
    binary = None
    while argv and argv[0].startswith('--'):
        if argv[0] == '--binary':
            binary, argv = argv[1], argv[2:]
        elif argv[0] == '--export':
            export_dir, argv = argv[1], argv[2:]
        else:
            sys.exit(f'unknown option {argv[0]}')
    if not argv:
        sys.exit(__doc__)
    cmd, args = argv[0], argv[1:]
    ex = Export(export_dir, binary)

    if cmd == 'fn':
        addr = ex.resolve(args[0])
        print(f'// {addr:#x} {ex.name(addr)} ({ex.functions[addr][1]} bytes) in {ex.binary}')
        print(ex.label(ex.bodies.get(addr, '// not decompiled (a thunk, or missing: '
                                     'scripts/ghidra-run.sh BINARY decompile OUT ADDR)\n')))
    elif cmd == 'callers':
        addr = ex.resolve(args[0])
        refs = ex.refs(addr)
        hits = [a for a, body in ex.bodies.items() if a != addr and any(r in body for r in refs)]
        for a in sorted(hits):
            print(f'{a:#x}  {ex.name(a)}')
        print(f'{len(hits)} functions refer to {ex.name(addr)}', file=sys.stderr)
    elif cmd == 'callees':
        addr = ex.resolve(args[0])
        seen = []
        for m in re.finditer(r'FUN_([0-9a-fA-F]+)', ex.bodies.get(addr, '')):
            a = int(m.group(1), 16)
            if a not in seen and a != addr:
                seen.append(a)
        for a in sorted(seen, key=lambda a: (a not in ex.labels, -ex.functions.get(a, ('', 0))[1])):
            print(f'{a:#x}  {ex.functions.get(a, ("", 0))[1]:6d} bytes  {ex.name(a)}')
    elif cmd == 'find':
        needle = ' '.join(args)
        for a, body in sorted(ex.bodies.items()):
            if needle in body:
                line = next(l.strip() for l in body.splitlines() if needle in l)
                print(f'{a:#x}  {ex.name(a):<40}  {line[:100]}')
    elif cmd == 'strings':
        needle = ' '.join(args).lower()
        for line in open(os.path.join(ex.dir, 'strings.txt'), encoding='utf-8', errors='replace'):
            addr, _, value = line.rstrip('\n').partition('\t')
            if needle in value.lower():
                users = [a for a, body in ex.bodies.items() if value[:60] in body]
                where = ', '.join(ex.name(a) for a in users[:6]) or '(no direct use in pseudocode)'
                print(f'{addr}  {value[:80]!r}  <- {where}')
    elif cmd == 'writers':
        off = int(args[0], 16) if args[0].lower().startswith('0x') else int(args[0])
        pat = re.compile(r'\+ (0x%x|%d)\) = ' % (off, off))
        for a, body in sorted(ex.bodies.items()):
            if pat.search(body):
                print(f'{a:#x}  {ex.name(a)}')
    elif cmd == 'biggest':
        n = int(args[0]) if args else 30
        todo = sorted(((size, a) for a, (_, size) in ex.functions.items()
                       if a not in ex.labels and a in ex.bodies), reverse=True)
        for size, a in todo[:n]:
            print(f'{a:#x}  {size:7d} bytes  {ex.name(a)}')
    elif cmd == 'stats':
        total = sum(size for _, size in ex.functions.values())
        done = sum(ex.functions[a][1] for a in ex.labels if a in ex.functions)
        print(f'{ex.binary}: {len(ex.functions)} functions, {len(ex.bodies)} decompiled, '
              f'{len(ex.labels)} labelled ({100 * done / max(total, 1):.1f} % of code by size)')
    else:
        sys.exit(__doc__)


if __name__ == '__main__':
    main(sys.argv[1:])
