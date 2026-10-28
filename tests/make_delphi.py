"""Build small Delphi-shaped files for the tests of tools/formats/dfm.py and tools/dos/delphi1.py.

Usage: make_delphi.py OUT_DIR

  OUT_DIR/delphi1.exe  a 16-bit NE program laid out as Delphi 1 lays one out: a code segment
                       with two classes' tables (VMT slots, a published method table, a field
                       table with its class table, class names), the selectors of their far
                       pointers in one relocation chain, a byte run that looks like a class
                       table but is not relocated, and the form TFORM1 as an RCDATA resource
  OUT_DIR/form.bin     junk, then a form stream as Delphi 2 and later write one (filer flags,
                       Single, Currency, Date, wide and UTF-8 strings, Int64, Double), then junk

Everything is made here; no compiler, no game. The class table layout is the one delphi1.py
documents (observed in a Delphi 1 program), so this proves the reader, not Borland's layout.
"""
import os
import struct
import sys

NEG = 0x22


def pstr(s):
    b = s.encode('latin-1')
    return bytes([len(b)]) + b


def p(name, value):
    return pstr(name) + value


def form_delphi1():
    """TPF0: Form1 (TForm1) with a value of every Delphi 1 type and a child with an event."""
    item = p('Caption', b'\x06' + pstr('x')) + b'\x00'
    props = (p('Left', b'\x03' + struct.pack('<h', 120))
             + p('Caption', b'\x06' + pstr('Test form'))
             + p('Scaled', b'\x08') + p('Visible', b'\x09')
             + p('Tag', b'\x04' + struct.pack('<i', 70000)) + p('Opacity', b'\x02\xfe')
             + p('Ratio', b'\x05' + struct.pack('<QH', 3 << 62, 16383))          # 1.5
             + p('OnCreate', b'\x07' + pstr('FormCreate'))
             + p('Font.Style', b'\x0b' + pstr('fsBold') + b'\x00')
             + p('Lines.Strings', b'\x01' + b'\x06' + pstr('a') + b'\x06' + pstr('b') + b'\x00')
             + p('Picture.Data', b'\x0a' + struct.pack('<i', 3) + b'\x01\x02\x03')
             + p('Hint', b'\x0c' + struct.pack('<i', 4) + b'long')
             + p('PopupMenu', b'\x0d')
             + p('Items', b'\x0e' + item + b'\x00'))
    child = pstr('TImage') + pstr('Image1') + p('OnClick', b'\x07' + pstr('image1click')) + b'\x00' + b'\x00'
    return b'TPF0' + pstr('TForm1') + pstr('Form1') + props + b'\x00' + child + b'\x00'


def form_later():
    """TPF0 with filer flags (inherited) and the value types Delphi 2 and later added."""
    props = (p('Alpha', b'\x0f' + struct.pack('<f', 0.5))
             + p('Price', b'\x10' + struct.pack('<q', 12345))                    # 1.2345
             + p('When', b'\x11' + struct.pack('<d', 36526.0))
             + p('Title', b'\x12' + struct.pack('<i', 2) + 'Ωx'.encode('utf-16-le'))
             + p('Big', b'\x13' + struct.pack('<q', 1 << 40))
             + p('Name8', b'\x14' + struct.pack('<i', 3) + 'é!'.encode('utf-8'))
             + p('Scale', b'\x15' + struct.pack('<d', 0.25))
             + p('OnShow', b'\x07' + pstr('FormShow')))
    return b'TPF0\xf1' + pstr('TForm2') + pstr('Form2') + props + b'\x00\x00'


class Code:
    """Segment 1: bytes, and the sites of the selectors the loader patches (one chain)."""

    def __init__(self):
        self.b, self.sites = bytearray(), []

    def at(self):
        return len(self.b)

    def word(self, v):
        self.b += struct.pack('<H', v)

    def far(self, off, relocated=True):
        self.word(off)
        if relocated:
            self.sites.append(self.at())
        self.word(0)

    def patch(self, at, v):
        struct.pack_into('<H', self.b, at, v)


def vmt(c, name_at, size, parent, methods=0, fields=0, ret=0):
    """A class's slots; returns the class reference (the VMT's offset). Fills in near pointers later."""
    start = c.at()
    c.word(0)                                    # unknown
    c.word(0)                                    # type information
    c.word(fields)
    c.word(methods)
    c.word(0)                                    # dynamic methods
    c.word(name_at)
    c.word(size)
    if parent is None:
        c.far(0, relocated=False)
    else:
        c.far(parent)
    for _ in range(4):                           # DefaultHandler, NewInstance, FreeInstance, Destroy
        c.far(ret)
    assert c.at() == start + NEG
    c.far(ret)                                   # one virtual method
    return start + NEG


def code_segment():
    c = Code()
    base = vmt(c, 0, 4, None)
    form = vmt(c, 0, 0x20, base)
    # A decoy: a name and slots without relocations, as plain data can look.
    decoy = c.at()
    c.b += bytes(NEG + 4)
    names = {}
    for n in ('TImage', 'TForm1', 'TDecoy'):
        names[n] = c.at()
        c.b += pstr(n)
    c.patch(decoy + 0x0a, names['TDecoy'])
    c.patch(decoy + 0x0c, 8)
    c.patch(base - NEG + 0x0a, names['TImage'])
    c.patch(form - NEG + 0x0a, names['TForm1'])
    table = c.at()
    c.patch(form - NEG + 0x06, table)
    c.word(2)
    fixups = []
    for m in ('FormCreate', 'Image1Click'):
        fixups.append(c.at())
        c.far(0)
        c.b += pstr(m)
    fields = c.at()
    c.patch(form - NEG + 0x04, fields)
    c.word(1)
    c.word(0)                                    # the class table, below
    c.word(0x1c)
    c.word(0)
    c.b += pstr('Image1')
    classes = c.at()
    c.patch(fields + 2, classes)
    c.word(1)
    c.far(base)
    ret = c.at()
    c.b += b'\xcb'                               # retf, the inherited methods' code
    code = {}
    for m, at in zip(('FormCreate', 'Image1Click'), fixups):
        code[m] = c.at()
        c.patch(at, c.at())
        c.b += b'\x55\x89\xe5\x5d\xcb'           # push bp; mov bp, sp; pop bp; retf
    for v in (base, form):                       # the inherited slots point at the retf
        for slot in (0x10, 0x0c, 0x08, 0x04):
            c.patch(v - slot, ret)
        c.patch(v, ret)
    for a, b in zip(c.sites, c.sites[1:] + [0xffff]):   # the chain: each site holds the next
        c.patch(a, b)
    return bytes(c.b), c.sites[0], {'TImage': base, 'TForm1': form, **code}


def ne(code, first_site, form):
    shift = 4
    pad = lambda b: b + bytes(-len(b) % (1 << shift))          # noqa: E731
    reloc = struct.pack('<H', 1) + struct.pack('<BBHBBH', 2, 0, first_site, 1, 0, 0)
    data = bytes(16)
    hdr = 0x40
    segtab = 0x40
    rsrc = segtab + 2 * 8
    rsrc_body = (struct.pack('<H', shift) + struct.pack('<HHI', 0x800a, 1, 0)
                 + struct.pack('<HHHHI', 0, 0, 0x0030, 0, 0) + struct.pack('<H', 0))
    name_at = len(rsrc_body)
    rsrc_body += pstr('TFORM1') + b'\x00'
    restab = rsrc + len(rsrc_body)
    resident = pstr('DELPHI1') + struct.pack('<H', 0) + b'\x00'
    modtab = restab + len(resident)
    imptab = modtab
    entry = imptab + 1
    entries = b'\x00\x00'
    nonres_rel = entry + len(entries)
    nonres = pstr('Delphi 1 shaped test program') + struct.pack('<H', 0) + b'\x00'
    head_len = hdr + nonres_rel + len(nonres)
    body = bytearray(pad(bytes(head_len)))
    code_at = len(body)
    body += pad(code + reloc)
    data_at = len(body)
    body += pad(data)
    form_at = len(body)
    body += pad(form)
    rsrc_body = bytearray(rsrc_body)
    struct.pack_into('<HHHH', rsrc_body, 10, form_at >> shift, len(pad(form)) >> shift, 0x0030, name_at)
    mz = bytearray(0x40)
    mz[0:2] = b'MZ'
    struct.pack_into('<H', mz, 0x18, 0x40)
    struct.pack_into('<I', mz, 0x3c, hdr)
    h = bytearray(0x40)
    h[0:2] = b'NE'
    h[2:4] = bytes([5, 10])
    struct.pack_into('<HH', h, 4, entry, len(entries))
    struct.pack_into('<HHHH', h, 0x0c, 0x0302, 2, 0x400, 0x2000)
    struct.pack_into('<HHHH', h, 0x14, 0, 1, 0, 2)
    struct.pack_into('<HHH', h, 0x1c, 2, 0, len(nonres))
    struct.pack_into('<HHHHH', h, 0x22, segtab, rsrc, restab, modtab, imptab)
    struct.pack_into('<I', h, 0x2c, hdr + nonres_rel)
    struct.pack_into('<H', h, 0x32, shift)
    h[0x36] = 2                                                   # Windows
    seg = (struct.pack('<HHHH', code_at >> shift, len(code), 0x0150, len(code))
           + struct.pack('<HHHH', data_at >> shift, len(data), 0x0c51, len(data)))
    head = bytes(mz) + bytes(h) + seg + bytes(rsrc_body) + resident + b'\x00' + entries + nonres
    body[:len(head)] = head
    return bytes(body)


def main(out):
    os.makedirs(out, exist_ok=True)
    code, first_site, where = code_segment()
    with open(os.path.join(out, 'delphi1.exe'), 'wb') as f:
        f.write(ne(code, first_site, form_delphi1()))
    with open(os.path.join(out, 'form.bin'), 'wb') as f:
        f.write(b'junk TPF\x00' * 3 + form_later() + b'more junk')
    for name, off in where.items():
        print(f'{name}\t1000:{off:04x}')


if __name__ == '__main__':
    main(sys.argv[1] if len(sys.argv) > 1 else '.')
