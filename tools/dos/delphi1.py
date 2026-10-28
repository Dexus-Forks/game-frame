#!/usr/bin/env python3
"""Delphi 1 (16-bit NE) class tables: every class, its published methods and fields, and the
code behind each form's events.

Usage: delphi1.py BINARY classes | methods | fields | labels | handlers

  classes   one line per class: VMT address, name, instance size, parent
  methods   one line per published method: address, Class.Method (event handlers are published)
  fields    one line per published field: Class, offset, name, the field's class
  labels    the published methods as a labels TSV (address, Class_Method, evidence) for
            labels/names.tsv and `scripts/ghidra-export.sh --labels`, which makes a function at
            each one Ghidra missed
  handlers  each form's event properties (tools/formats/dfm.py) with the code of their methods

Delphi 1 keeps each class's tables in the code segment of its unit, as data between the
procedures. The class reference (`TClass`) is a far pointer to the VMT; the slots before it
are 2-byte near pointers into the same segment and 4-byte far pointers (offset, then a
selector the loader patches through a relocation chain):

  VMT-0x22  word   unknown (small numbers)        VMT-0x16  word  instance size in bytes
  VMT-0x20  near   type information (RTTI)        VMT-0x14  far   parent class (0 for TObject)
  VMT-0x1e  near   published field table          VMT-0x10  far   DefaultHandler
  VMT-0x1c  near   published method table         VMT-0x0c  far   NewInstance
  VMT-0x1a  near   dynamic method table           VMT-0x08  far   FreeInstance
  VMT-0x18  near   class name (a Pascal string)   VMT-0x04  far   Destroy
  VMT+0     far    the virtual methods, 4 bytes each

  method table: word count, then per method a far pointer and its name (a Pascal string)
  field table:  word count, near pointer to a class table (word count, far pointers to VMTs),
                then per field a word offset in the instance, a word class index, its name

The layout is *observed* in one Delphi 1 program (Holiday Island, 1997), not taken from
Borland's documentation; 32-bit Delphi lays its tables out differently. A table is accepted
only when its NewInstance and FreeInstance slots are relocated far pointers and its name is an
identifier, so code bytes do not pass for one. Event handlers are reached only through these
tables, never by a call, so Ghidra's export misses many of them: `labels` names them all.

Standard library only (tools/dos/ne.py, tools/formats/dfm.py).
"""
import os
import re
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path[:0] = [HERE, os.path.join(HERE, '..', 'formats')]
from ne import load  # noqa: E402

NEG = 0x22                     # the VMT's slots before the class reference
IDENT = re.compile(rb'[A-Za-z_][A-Za-z0-9_]*\Z')
CHAIN_LIMIT = 0x10000


def relocated(exe, seg, img):
    """{site of a selector word: target segment number} for one segment's internal relocations."""
    d, at, sites = exe.data, seg.offset + seg.length + 2, {}
    for i in range(seg.relocs):
        src, flags, site, tseg, _, toff = struct.unpack_from('<BBHBBH', d, at + 8 * i)
        if flags & 3:
            continue                                     # imports and OS fixups
        if tseg == 0xff:
            tseg = exe.entries.get(toff, (0, 0))[0]
        if flags & 4:                                    # additive: one site
            sites[site + (2 if src == 3 else 0)] = tseg
            continue
        if src not in (2, 3):
            continue
        for _ in range(CHAIN_LIMIT):                     # each site holds the next, 0xffff ends
            if site == 0xffff or site + 2 > len(img):
                break
            sites[site + (2 if src == 3 else 0)] = tseg
            site = struct.unpack_from('<H', img, site)[0]
    return sites


def pstr_at(img, p):
    if not 0 < p < len(img):
        return None
    n = img[p]
    raw = img[p + 1:p + 1 + n]
    return raw.decode('latin-1') if n and len(raw) == n and IDENT.match(raw) else None


class Program:
    def __init__(self, path):
        self.exe = load(path)
        if self.exe.kind != 'NE':
            raise SystemExit(f'{path}: not an NE program (delphi1.py reads 16-bit Delphi 1 programs)')
        self.images, self.sites = {}, {}
        for s in self.exe.segments:
            if s.is_data or not s.offset:
                continue
            img = self.exe.image(s.selector)
            self.images[s.selector] = img
            self.sites[s.selector] = relocated(self.exe, s, img)
        self.classes = {}                                # (selector, VMT offset) -> class
        for sel in self.images:
            self._scan(sel)
        # Every class but TObject derives from another class of the program (Delphi 1 has no
        # packages): drop candidates whose parent is no class, until none is left.
        while True:
            orphans = [k for k, c in self.classes.items() if c['parent'] and c['parent'] not in self.classes]
            if not orphans:
                break
            for k in orphans:
                del self.classes[k]

    def far(self, sel, at):
        """The far pointer at sel:at as (selector, offset), when the loader patches its selector."""
        img, tseg = self.images[sel], self.sites[sel].get(at + 2)
        if tseg is None or at + 4 > len(img):
            return None
        return self.exe.selector(tseg), struct.unpack_from('<H', img, at)[0]

    def _scan(self, sel):
        img, sites = self.images[sel], self.sites[sel]
        for b in range(0, len(img) - NEG):
            if b + NEG - 0x0a not in sites or b + NEG - 0x06 not in sites:   # NewInstance, FreeInstance
                continue
            w = struct.unpack_from('<6H', img, b + 2)
            typeinfo, fields, methods, dynamic, name_at, size = w
            if any(q and (b <= q < b + NEG or q >= len(img)) for q in (typeinfo, fields, methods, dynamic, name_at)):
                continue                                 # a table pointer into the slots themselves
            name = pstr_at(img, name_at)
            parent = self.far(sel, b + NEG - 0x14)
            if not name or size < 4 or (parent is None and img[b + NEG - 0x14:b + NEG - 0x10] != bytes(4)):
                continue                                 # no class, or a parent the loader does not patch
            self.classes[(sel, b + NEG)] = {
                'name': name, 'vmt': (sel, b + NEG), 'size': size, 'parent': parent,
                'methods': self._methods(sel, methods), 'fields': self._fields(sel, fields)}

    def _methods(self, sel, at):
        img, out = self.images[sel], []
        if not 0 < at < len(img) - 2:
            return out
        n, p = struct.unpack_from('<H', img, at)[0], at + 2
        for _ in range(n):
            target, name = self.far(sel, p), pstr_at(img, p + 4)
            if target is None or name is None:
                return out
            out.append((name, target))
            p += 5 + img[p + 4]
        return out

    def _fields(self, sel, at):
        img, out = self.images[sel], []
        if not 0 < at < len(img) - 4:
            return out
        n, table = struct.unpack_from('<HH', img, at)
        refs = []
        if 0 < table < len(img) - 2:
            k = struct.unpack_from('<H', img, table)[0]
            refs = [self.far(sel, table + 2 + 4 * i) for i in range(k)]
        p = at + 4
        for _ in range(n):
            if p + 5 > len(img):
                break
            off, idx = struct.unpack_from('<HH', img, p)
            name = pstr_at(img, p + 4)
            if name is None:
                break
            out.append((off, name, refs[idx] if idx < len(refs) else None))
            p += 5 + img[p + 4]
        return out

    def class_name(self, ref):
        c = self.classes.get(ref) if ref else None
        return c['name'] if c else ('-' if ref is None else f'{ref[0]:04x}:{ref[1]:04x}')

    def by_name(self):
        return {c['name']: c for c in self.classes.values()}


def addr(ref):
    return f'{ref[0]:04x}:{ref[1]:04x}'


def handlers(path, prog):
    """(form, object, event, method, address or '?') for every event of every form."""
    from dfm import events, forms
    classes = prog.by_name()
    for name, _, root in forms(prog.exe.data):
        cls = classes.get(root['class'])
        # Delphi finds a published method by name without regard to case.
        table = {m.lower(): ref for m, ref in cls['methods']} if cls else {}
        for form, obj, event, method in events(root, str(name)):
            ref = table.get(method.lower())
            yield form, obj, event, f'{root["class"]}.{method}', addr(ref) if ref else '?'


def main(argv):
    if len(argv) != 2 or argv[1] not in ('classes', 'methods', 'fields', 'labels', 'handlers'):
        sys.exit(__doc__)
    prog = Program(argv[0])
    rows = sorted(prog.classes.values(), key=lambda c: c['vmt'])
    cmd = argv[1]
    if cmd == 'classes':
        for c in rows:
            print(f'{addr(c["vmt"])}\t{c["name"]}\t{c["size"]}\t{prog.class_name(c["parent"])}')
    elif cmd == 'methods':
        for c in rows:
            for m, ref in c['methods']:
                print(f'{addr(ref)}\t{c["name"]}.{m}')
    elif cmd == 'fields':
        for c in rows:
            for off, f, ref in c['fields']:
                print(f'{c["name"]}\t+{off:#x}\t{f}\t{prog.class_name(ref)}')
    elif cmd == 'labels':
        print('# address\tlabel\tevidence (from delphi1.py: the program\'s class tables)')
        # Methods only: ApplyLabels makes a function at every label in a code segment, and the
        # class tables are data there.
        for c in rows:
            for m, ref in c['methods']:
                print(f'{addr(ref)}\t{c["name"]}_{m}\tobserved: published method of {c["name"]} '
                      f'(method table of the VMT at {addr(c["vmt"])})')
    else:
        for row in handlers(argv[0], prog):
            print('\t'.join(row))
    sys.stdout.flush()
    print(f'{len(rows)} classes, {sum(len(c["methods"]) for c in rows)} published methods', file=sys.stderr)


if __name__ == '__main__':
    main(sys.argv[1:])
