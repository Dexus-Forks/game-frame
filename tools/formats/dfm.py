#!/usr/bin/env python3
"""Read Delphi and C++Builder forms (binary DFM, the `TPF0` stream) from a program or a file, as text.

Usage: dfm.py FILE [OUT_DIR] [--list | --events]

  FILE     a 16-bit NE program (Delphi 1: forms are RCDATA resources named after the form's
           class), a 32-bit program, a .dfm file or any blob: outside an NE resource table every
           `TPF0` in the file is read
  OUT_DIR  write one NAME.txt per form (default: print them all). Forms are the game's own
           material: write them under derived/, never into git
  --list   one line per form: its name, size, root class and object count
  --events one line per event property: form, object, event, handler method (TAB-separated)

A form stream is `TPF0`, then one object: its class and name as Pascal strings, its properties
until a zero byte, its child objects until a zero byte. From Delphi 2 on an object may start
with a filer-flags byte (0xF0 + flags; with ffChildPos an integer follows). A property is a
Pascal-string name and a typed value (TValueType): 1 list, 2/3/4 integers of 1/2/4 bytes, 5 an
80-bit Extended, 6 string, 7 identifier, 8/9 False/True, 10 binary (32-bit length), 11 set
(names until an empty one), 12 long string, 13 nil, 14 collection; Delphi 1 stops there, later
versions add 15 Single, 16 Currency, 17 Date, 18 wide string, 19 Int64, 20 UTF-8 string,
21 Double. An event property (`OnClick = Button1Click`) holds an identifier: the name of a
published method of the form's class. Find its code with `tools/dos/delphi1.py handlers`
(Delphi 1) or the class's method table.

    from dfm import forms, parse, events
    for name, size, root in forms(open('GAME.EXE', 'rb').read()):
        for form, obj, event, method in events(root, name): ...

Standard library only.
"""
import os
import struct
import sys

RT_RCDATA = 10
MAGIC = b'TPF0'


class Stream:
    def __init__(self, data, pos=0):
        self.d, self.p = data, pos

    def peek(self):
        if self.p >= len(self.d):
            raise ValueError(f'the form ends at {self.p:#x} inside an object')
        return self.d[self.p]

    def u8(self):
        self.peek()
        self.p += 1
        return self.d[self.p - 1]

    def take(self, n):
        if self.p + n > len(self.d):
            raise ValueError(f'a value at {self.p:#x} runs past the end')
        self.p += n
        return self.d[self.p - n:self.p]

    def pstr(self):
        return self.take(self.u8()).decode('cp1252')

    def num(self, fmt):
        return struct.unpack(fmt, self.take(struct.calcsize(fmt)))[0]


def extended(b):
    """An 80-bit Extended's value as a float (rounded); exact enough to show."""
    mant, se = struct.unpack('<QH', b)
    if se & 0x7fff == 0 and mant == 0:
        return 0.0
    v = mant * 2.0 ** ((se & 0x7fff) - 16383 - 63)
    return -v if se & 0x8000 else v


def value(s):
    kind = s.u8()
    simple = {2: '<b', 3: '<h', 4: '<i', 15: '<f', 17: '<d', 19: '<q', 21: '<d'}
    if kind in simple:
        return s.num(simple[kind])
    if kind == 1:                                        # vaList
        items = []
        while s.peek() != 0:
            items.append(value(s))
        s.u8()
        return items
    if kind == 5:
        return extended(s.take(10))
    if kind == 6:
        return s.pstr()
    if kind == 7:                                        # vaIdent: a name, such as a method's
        return ('ident', s.pstr())
    if kind in (8, 9):
        return kind == 9
    if kind == 10:                                       # vaBinary: shown by its length
        return ('binary', len(s.take(s.num('<i'))))
    if kind == 11:                                       # vaSet
        items = []
        while True:
            n = s.pstr()
            if not n:
                return ('set', items)
            items.append(n)
    if kind == 12:
        return s.take(s.num('<i')).decode('cp1252')
    if kind == 13:
        return None
    if kind == 14:                                       # vaCollection: items, each an optional index then properties
        items = []
        while s.peek() != 0:
            if s.peek() in (2, 3, 4):
                value(s)
            items.append(properties(s))
        s.u8()
        return ('collection', items)
    if kind == 16:                                       # Currency: an Int64 of ten-thousandths
        return s.num('<q') / 10000
    if kind == 18:                                       # wide string: a length in characters
        return s.take(2 * s.num('<i')).decode('utf-16-le')
    if kind == 20:
        return s.take(s.num('<i')).decode('utf-8')
    raise ValueError(f'value type {kind} at {s.p - 1:#x}')


def properties(s):
    props = []
    while s.peek() != 0:
        props.append((s.pstr(), value(s)))
    s.u8()
    return props


def obj(s):
    flags = 0
    if s.peek() & 0xf0 == 0xf0:                          # Delphi 2 on: filer flags
        flags = s.u8() & 0x0f
        if flags & 2:                                    # ffChildPos: the child's position
            value(s)
    cls, name = s.pstr(), s.pstr()
    props = properties(s)
    children = []
    while s.peek() != 0:
        children.append(obj(s))
    s.u8()
    out = {'class': cls, 'name': name, 'props': props, 'children': children}
    if flags & 1:
        out['inherited'] = True
    return out


def parse(body, at=0):
    """The root object of the form stream starting at `at` (with `TPF0`), and where it ends."""
    if body[at:at + 4] != MAGIC:
        raise ValueError(f'no {MAGIC!r} at {at:#x}')
    s = Stream(body, at + 4)
    return obj(s), s.p


def ne_resources(data):
    """(type, name, bytes) of every resource of an NE program; type and name are an int or a str."""
    ne = struct.unpack_from('<H', data, 0x3c)[0]
    table = ne + struct.unpack_from('<H', data, ne + 0x24)[0]
    if table == ne + struct.unpack_from('<H', data, ne + 0x26)[0]:
        return []                                        # no resource table
    shift = struct.unpack_from('<H', data, table)[0]

    def name(v):
        if v & 0x8000:
            return v & 0x7fff
        n = data[table + v]
        return data[table + v + 1:table + v + 1 + n].decode('latin-1')

    out, p = [], table + 2
    while True:
        rtype = struct.unpack_from('<H', data, p)[0]
        if rtype == 0:
            return out
        count = struct.unpack_from('<H', data, p + 2)[0]
        p += 8
        for _ in range(count):
            off, length, _, rid = struct.unpack_from('<HHHH', data, p)
            p += 12
            out.append((name(rtype), name(rid), data[off << shift:(off << shift) + (length << shift)]))


def is_ne(data):
    if data[:2] != b'MZ' or len(data) < 0x40:
        return False
    ne = struct.unpack_from('<I', data, 0x3c)[0]
    return data[ne:ne + 2] == b'NE'


def forms(data):
    """(name, size, root object) of every form: an NE program's RCDATA forms by resource name,
    otherwise each `TPF0` in the bytes by its offset (`@0x...`). A form that does not read is
    reported on stderr and skipped."""
    if is_ne(data):
        found = [(rid, body, 0) for rtype, rid, body in ne_resources(data)
                 if rtype == RT_RCDATA and body[:4] == MAGIC]
    else:
        found, at = [], data.find(MAGIC)
        while at >= 0:
            found.append((f'@{at:#x}', data, at))
            at = data.find(MAGIC, at + 4)
    for name, body, at in found:
        try:
            root, end = parse(body, at)
        except (ValueError, IndexError, struct.error, UnicodeDecodeError) as e:
            print(f'{name}: cannot read the form: {e}', file=sys.stderr)
            continue
        yield name, end - at, root


def walk(o):
    yield o
    for c in o['children']:
        yield from walk(c)


def events(root, form=None):
    """(form, object, event, method) for every property whose value is an identifier named On*."""
    form = form if form is not None else root['class']
    for o in walk(root):
        for k, v in o['props']:
            if k.startswith('On') and isinstance(v, tuple) and v[0] == 'ident':
                yield form, o['name'], k, v[1]


def shown(v):
    if isinstance(v, tuple):
        kind, inner = v
        if kind == 'ident':
            return inner
        if kind == 'set':
            return '[' + ', '.join(inner) + ']'
        if kind == 'binary':
            return f'{{{inner} bytes}}'
        return f'<{len(inner)} items>'
    if isinstance(v, list):
        return '(' + ' '.join(shown(i) for i in v) + ')'
    return 'nil' if v is None else repr(v)


def text(o, depth=0):
    pad = '  ' * depth
    word = 'inherited' if o.get('inherited') else 'object'
    lines = [f'{pad}{word} {o["name"]}: {o["class"]}']
    lines += [f'{pad}  {k} = {shown(v)}' for k, v in o['props']]
    for c in o['children']:
        lines += text(c, depth + 1)
    return lines + [f'{pad}end']


def main(argv):
    if not argv or argv[0] in ('-h', '--help'):
        sys.exit(__doc__)
    data = open(argv[0], 'rb').read()
    out = next((a for a in argv[1:] if not a.startswith('--')), None)
    if out and os.path.realpath(out).startswith(os.path.realpath('target') + os.sep):
        sys.exit('refusing to write into target/: it is the game; use derived/')
    count = 0
    for name, size, root in forms(data):
        count += 1
        if '--list' in argv:
            print(f'{name}\t{size}\t{root["class"]}\t{sum(1 for _ in walk(root))} objects')
        elif '--events' in argv:
            for row in events(root, str(name)):
                print('\t'.join(row))
        elif out:
            os.makedirs(out, exist_ok=True)
            safe = os.path.basename(str(name)).replace('..', '_') or 'unnamed'
            with open(os.path.join(out, f'{safe}.txt'), 'w') as f:
                f.write('\n'.join(text(root)) + '\n')
        else:
            print('\n'.join(text(root)))
    if not count:
        print(f'{argv[0]}: no Delphi form (TPF0) found', file=sys.stderr)
        sys.exit(1)


if __name__ == '__main__':
    main(sys.argv[1:])
