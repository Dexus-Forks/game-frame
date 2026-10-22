"""Diagrams for the media guides (docs/media/): from a disc or disk to the code, the kinds of disc,
ISO 9660, disc images, a booter's load, file signatures, a picture found by its width, structure
from the numbers, two saves compared, constants that name algorithms, Windows resources, XOR
obfuscation, timestamps, the entropy of packed programs, data decoders, flattened control flow,
timing checks, and copy protection by era. site/diagrams/make.py writes them.
"""
import base64
import datetime
import struct
import zlib

from engines import arrow, box, during, group, legend, lines, scratch
from runtime import show
from svg import AMBER, BG, BLUE, DIM, EDGE, GREEN, GREY, MONO, PURPLE, RED, TEAL, TEXT, Svg


def hexrow(s, x, y, data, colours=None, cw=26, size=11):
    """A row of bytes in hex; COLOURS maps an index to a tint."""
    for k, b in enumerate(data):
        c = (colours or {}).get(k)
        if c:
            s.rect(x + k * cw - 2, y - 13, cw - 2, 18, c, rx=2, extra=' fill-opacity="0.35"')
        s.text(x + k * cw + (cw - 4) / 2, y, f'{b:02X}', size, TEXT if c else DIM, 'middle', mono=True)


# ---------------------------------------------------------------- README: from the media to the code
def overview():
    s = Svg(900, 270, 'From what the game came on to the code. An image of the disc or disk gives the files. Each file is identified, and packed code is unpacked before Ghidra')
    s.heading('From the media to the code',
              'Each step has its own tools. Record what each step found before you start the next one.')
    steps = [('disc or disk image', 'discinfo.py, mtools', 'tracks, file systems, dates', AMBER),
             ('the files', 'filetypes.py', 'kind, entropy, signatures', TEAL),
             ('each unknown file', 'binview, xorscan, similar', 'structure, obfuscation', BLUE),
             ('the code', 'peinfo, dosinfo, unpacking', 'readable, ready for Ghidra', PURPLE)]
    for k, (title, tool, out, colour) in enumerate(steps):
        x = 24 + k * 216
        box(s, x, 92, 200, 60, colour, title, tool, op=0.25)
        s.text(x + 100, 176, out, 11, DIM, 'middle')
        if k < 3:
            arrow(s, x + 202, 122, x + 214, 122)
    s.rect(24, 200, 852, 40, GREY, rx=5, extra=' fill-opacity="0.15"')
    s.text(40, 225, 'Record the hash of every file and where the copy came from. Commit nothing from target/, analysis/ or derived/.',
           11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- discs: kinds of disc
def disc_kinds():
    s = Svg(900, 390, 'The layouts of the kinds of CD, which are data only, mixed mode with audio after the data track, Enhanced CD with data in a second session, multi-session, hybrid ISO 9660 and HFS, and bootable discs with El Torito')
    s.heading('The layouts of the kinds of disc',
              'Each bar shows a disc from its start on the left to its end. The text on the right says what the layout means for the port.')
    rows = [('Data only', [('data track 1', 6, BLUE)], 'all the game is in the files'),
            ('Mixed mode', [('data', 3, BLUE), ('audio 2', 1, AMBER), ('audio 3', 1, AMBER), ('audio 4', 1, AMBER)],
             'the port must play the audio tracks'),
            ('Enhanced CD', [('audio, session 1', 4, AMBER), ('|', 0.15, RED), ('data, session 2', 2, BLUE)],
             'data sector numbers start after the audio'),
            ('Multi-session', [('session 1', 2, GREY), ('|', 0.15, RED), ('session 2', 2, GREY), ('|', 0.15, RED),
                               ('session 3', 2, BLUE)], 'read the last session\'s file system'),
            ('Hybrid', [('ISO 9660 + HFS on one data track', 6, PURPLE)], 'the Mac files can differ'),
            ('Bootable', [('El Torito boot record + data', 6, TEAL)], 'starts its own program')]
    unit = 66
    for r, (name, parts, note) in enumerate(rows):
        y = 88 + r * 46
        s.text(24, y + 22, name, 12, TEXT, weight='600')
        x = 150
        for label, n, colour in parts:
            w = n * unit
            if label == '|':
                s.rect(x, y + 2, w, 32, colour, rx=1)
            else:
                box(s, x, y + 2, w - 3, 32, colour, label, op=0.3, size=10.5)
            x += w
        s.text(570, y + 22, note, 11.5, DIM)
    legend(s, 150, 374, [(BLUE, 'data'), (AMBER, 'CD audio'), (RED, 'session boundary')])
    return s.render()


# ---------------------------------------------------------------- discs: ISO 9660
def iso9660():
    s = Svg(900, 360, 'ISO 9660 on a data track. Sectors 0 to 15 are the system area. Sector 16 holds the primary volume descriptor with names and four dates. The path tables, the directories and the files follow')
    s.heading('Where ISO 9660 keeps its structures',
              'Sector numbers count 2048-byte sectors from the start of the data track.')
    parts = [('0 to 15, the system area', 200, GREY), ('16, primary volume descriptor', 190, AMBER), ('17, Joliet and terminator', 140, PURPLE),
             ('path tables', 90, TEAL), ('directories', 90, TEAL), ('files ...', 142, BLUE)]
    x = 24
    for label, w, colour in parts:
        box(s, x, 88, w - 3, 36, colour, label, op=0.3, size=10.5)
        x += w
    s.rect(224, 140, 300, 130, AMBER, rx=4, extra=' fill-opacity="0.12"')
    s.line(318, 124, 318, 140, AMBER, 1.5)
    s.text(236, 160, 'the primary volume descriptor', 11.5, TEXT, weight='600')
    lines(s, 236, 180, ['system, volume, publisher,', 'data preparer, application', 'the dates created, modified,',
                        'expires, effective', 'volume size, root directory'], 11, DIM, gap=17)
    s.text(24, 160, 'empty on a PC disc,', 11, DIM)
    s.text(24, 176, 'licence text and boot', 11, DIM)
    s.text(24, 192, 'code on many consoles', 11, DIM)
    s.text(560, 160, 'Numbers are stored twice', 11.5, TEXT, weight='600')
    s.text(560, 180, '2048 as a 32-bit field in 8 bytes', 11, DIM)
    hexrow(s, 560, 204, [0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00], {0: TEAL, 1: TEAL, 2: TEAL, 3: TEAL, 4: AMBER,
                                                                          5: AMBER, 6: AMBER, 7: AMBER}, cw=36)
    s.text(560, 226, 'little-endian, then big-endian', 10.5, DIM)
    s.text(24, 296, 'Four checks a file list misses', 12, TEXT, weight='600')
    lines(s, 24, 316, ['Sectors that no file uses (--map), the system area, files in only one of the ISO and Joliet trees,',
                       'and files dated after the volume.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- discs: images
def disc_images():
    s = Svg(900, 330, 'What each disc image format keeps of the user data, sector headers, audio tracks, Form 2 sectors and subchannels, for .iso, .cue and .bin, CloneCD, Alcohol 120% and CHD images')
    s.heading('What each disc image format keeps',
              'Work from a raw image when the disc has audio or Mode 2 data.')
    cols = ['2048 data bytes', 'sector headers', 'audio tracks', 'Form 2 sectors', 'subchannels']
    rows = [('.iso', [1, 0, 0, 0, 0]), ('.cue + .bin', [1, 1, 1, 1, 0]), ('.ccd .img .sub', [1, 1, 1, 1, 1]),
            ('.mds + .mdf', [1, 1, 1, 1, 2]), ('.chd', [1, 1, 1, 1, 2])]
    for k, c in enumerate(cols):
        s.text(250 + k * 126 + 60, 96, c, 11, DIM, 'middle')
    for r, (name, keep) in enumerate(rows):
        y = 108 + r * 38
        s.rect(24, y, 852, 32, GREY, rx=4, extra=' fill-opacity="0.08"')
        s.text(40, y + 21, name, 12, TEXT, weight='600', mono=True)
        for k, v in enumerate(keep):
            cx = 250 + k * 126 + 60
            if v == 1:
                s.rect(cx - 40, y + 6, 80, 20, TEAL, rx=3, extra=' fill-opacity="0.45"')
                s.text(cx, y + 20, 'kept', 10.5, TEXT, 'middle')
            elif v == 2:
                s.rect(cx - 40, y + 6, 80, 20, TEAL, rx=3, extra=' fill-opacity="0.2"')
                s.text(cx, y + 20, 'often', 10.5, TEXT, 'middle')
            else:
                s.rect(cx - 40, y + 6, 80, 20, RED, rx=3, extra=' fill-opacity="0.2"')
                s.text(cx, y + 20, 'lost', 10.5, DIM, 'middle')
    s.text(24, 316, 'Some protections read the subchannel (LibCrypt on PAL PlayStation discs). Keep the .sub file or Redump\'s SBI data.',
           11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- floppies: a booter (animated)
def booter():
    s = Svg(900, 360, 'A PC booter game. The BIOS reads the boot sector to 0000:7C00 and jumps there. The boot code reads more sectors with interrupt 13h to addresses it chooses, which make up the load map')
    s.heading('How a booter game loads itself',
              'The BIOS loads one sector. The code in that sector loads the rest of the game, sector by sector, to addresses it chooses.')
    dur = 7.0
    # The disk.
    s.text(24, 92, 'the disk', 11.5, TEXT, weight='600')
    for k in range(12):
        colour = AMBER if k == 0 else (BLUE if k < 9 else GREY)
        s.rect(24 + k * 30, 100, 28, 30, colour, rx=2, extra=f' fill-opacity="{0.45 if k < 9 else 0.15}"')
    s.text(60, 148, 'sector 1 of track 0 is the boot sector', 10.5, DIM)
    # Memory.
    mx = 520
    s.text(mx, 92, 'memory', 11.5, TEXT, weight='600')
    regions = [('0000:7C00  boot code, 512 bytes', AMBER, 0.0), ('1000:0000  loader', BLUE, 0.3),
               ('2000:0000  game code', BLUE, 0.5), ('3000:0000  graphics', BLUE, 0.7)]
    for k, (label, colour, t0) in enumerate(regions):
        y = 100 + k * 40
        s.rect(mx, y, 356, 34, GREY, rx=3, extra=' fill-opacity="0.12"')
        body = scratch(lambda t, y=y, label=label, colour=colour: box(t, mx, y, 356, 34, colour, label, op=0.35, size=11, mono=True))
        s.add(group(body, t0 + 0.05, 0.97, dur))
    s.add(f'<path d="M38 132C 38 220 430 230 516 118" fill="none" stroke="{AMBER}" stroke-width="1.5" '
          f'marker-end="url(#arrow)"/>')
    s.text(245, 224, 'the BIOS reads it and jumps there', 10.5, AMBER, 'middle')
    # The load map.
    s.text(24, 258, 'the load map, built from each INT 13h read', 11.5, TEXT, weight='600')
    maps = [('track 0, side 0, sectors 2 to 5', '1000:0000', 0.3), ('track 1, side 0, sectors 1 to 9', '2000:0000', 0.5),
            ('track 2, side 1, sectors 1 to 9', '3000:0000', 0.7)]
    for k, (src, dst, t0) in enumerate(maps):
        y = 270 + k * 24
        body = scratch(lambda t, y=y, src=src, dst=dst: (t.text(24, y + 14, src, 11, TEXT, mono=True),
                                                          t.text(330, y + 14, '->  ' + dst, 11, TEAL, mono=True)))
        s.add(group(body, t0 + 0.05, 0.97, dur))
    s.text(520, 300, 'The addresses here are illustrative.', 10.5, DIM)
    s.text(520, 318, 'Log each INT 13h call in an emulator', 10.5, DIM)
    s.text(520, 336, 'for measured evidence.', 10.5, DIM)
    return s.render()


# ---------------------------------------------------------------- files: signatures
def magic():
    s = Svg(900, 360, 'The first bytes of common formats in hex and as text, for MZ executables, ZIP, RIFF, PNG, GIF, Doom WAD, Quake PAK, Build GRP, Blizzard MPQ and Flash SWF')
    s.heading('File signatures in the first bytes',
              'Most formats start with a fixed signature. These are some of the signatures that filetypes.py knows.')
    rows = [('MZ', [0x4D, 0x5A], 'DOS and Windows executables'), ('PK', [0x50, 0x4B, 0x03, 0x04], 'ZIP, and .pk3 files'),
            ('RIFF', [0x52, 0x49, 0x46, 0x46], 'WAVE, AVI, and Director (RIFX, XFIR)'),
            ('.PNG', [0x89, 0x50, 0x4E, 0x47], 'PNG pictures'), ('GIF8', [0x47, 0x49, 0x46, 0x38], 'GIF pictures'),
            ('IWAD', [0x49, 0x57, 0x41, 0x44], 'Doom WAD (or PWAD)'), ('PACK', [0x50, 0x41, 0x43, 0x4B], 'Quake PAK'),
            ('KenS', [0x4B, 0x65, 0x6E, 0x53], 'Build engine GRP, "KenSilverman"'),
            ('MPQ.', [0x4D, 0x50, 0x51, 0x1A], 'Blizzard MPQ'), ('CWS', [0x43, 0x57, 0x53], 'Flash, compressed (FWS, ZWS)')]
    for k, (txt, data, what) in enumerate(rows):
        col, row = divmod(k, 5)
        x, y = 24 + col * 440, 100 + row * 48
        s.rect(x, y - 18, 420, 38, GREY, rx=4, extra=' fill-opacity="0.1"')
        hexrow(s, x + 10, y + 5, data, {i: AMBER for i in range(len(data))}, cw=30)
        s.text(x + 140, y + 5, txt, 12, TEAL, mono=True)
        s.text(x + 190, y + 5, what, 11, DIM)
    s.text(24, 346, 'A file with no known signature is often the game\'s own format. Its name in the strings and the code that opens it say more.',
           11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- files: a picture by its width
def _png(w, h, pixels, palette):
    raw = b''.join(b'\x00' + bytes(pixels[y * w:(y + 1) * w]) for y in range(h))

    def chunk(tag, data):
        return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', zlib.crc32(tag + data) & 0xffffffff)
    plte = b''.join(bytes(c) for c in palette)
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 3, 0, 0, 0)) +
            chunk(b'PLTE', plte) + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b''))


def raw_width():
    s = Svg(900, 330, 'The same bytes drawn as a picture at three widths. A width that is too narrow or too wide gives slanted stripes. The right width makes the picture stand straight')
    s.heading('Finding a picture in the data by its width',
              'The data has one byte per pixel and no header. Change the width until the picture stands straight.')
    W, H = 64, 40
    data = []
    for y in range(H):
        for x in range(W):
            v = 0
            if (x - 20) ** 2 + (y - 20) ** 2 < 150:
                v = 1
            if 38 <= x <= 58 and 8 <= y <= 32 and (x == 38 or x == 58 or y == 8 or y == 32 or x - 38 == y - 8):
                v = 2
            if y == 37 and x % 4 < 2:
                v = 3
            data.append(v)
    palette = [(13, 27, 30), (45, 212, 191), (245, 158, 11), (96, 165, 250)]
    for k, (width, label, ok) in enumerate(((60, 'width 60', False), (64, 'width 64, the right one', True), (68, 'width 68', False))):
        h = len(data) // width
        png = _png(width, h, data[:width * h], palette)
        uri = base64.b64encode(png).decode()
        x = 24 + k * 292
        s.add(f'<image x="{x}" y="88" width="{width * 4}" height="{h * 4}" href="data:image/png;base64,{uri}" '
              f'style="image-rendering:pixelated" preserveAspectRatio="none"/>')
        s.text(x, 88 + h * 4 + 20, label, 12, TEAL if ok else RED, weight='600')
    s.text(24, 316, 'rawimage.py tries 8-bit, Mode X planes, EGA planes, CGA and 16-bit colour, with a palette from a file.', 11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- files: structure from the numbers
def structure():
    s = Svg(900, 330, 'Reading a file\'s layout from its numbers. The file holds a value equal to its size, a count followed by that many offsets, offsets that rise and stay inside the file, and a length-prefixed string')
    s.heading('Reading the structure of a file from its numbers',
              'This example file is 96 bytes long. Before you read any code, its values already show the layout.')
    data = [0x4C, 0x56, 0x4C, 0x31, 0x60, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00,
            0x30, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00, 0x05, 0x46, 0x4F, 0x52, 0x45, 0x53, 0x54, 0x00]
    tints = {}
    for i in range(4):
        tints[i] = GREY
    for i in range(4, 8):
        tints[i] = AMBER
    for i in range(8, 12):
        tints[i] = TEAL
    for i in range(12, 24):
        tints[i] = BLUE
    for i in range(24, 31):
        tints[i] = PURPLE
    for row in range(2):
        y = 104 + row * 30
        s.text(24, y, f'{row * 16:04X}', 11, DIM, mono=True)
        hexrow(s, 80, y, data[row * 16:row * 16 + 16], {k: tints.get(row * 16 + k) for k in range(16)}, cw=30)
    notes = [(AMBER, '60 00 00 00 = 96, the file\'s own size'), (TEAL, '03 00 00 00, a count of 3'),
             (BLUE, 'three rising offsets, 0x18, 0x30 and 0x48, 24 bytes apart'), (PURPLE, '05 then FOREST, a length-prefixed string'),
             (GREY, 'LVL1, a signature')]
    for k, (colour, text) in enumerate(notes):
        y = 190 + k * 24
        s.rect(80, y - 11, 14, 14, colour, rx=2, extra=' fill-opacity="0.5"')
        s.text(102, y, text, 11.5, TEXT)
    lines(s, 600, 186, ['Count x 24 bytes fits', 'the bytes after the', 'header, so there are', 'three records of 24 bytes.'], 11.5, DIM, gap=18)
    s.text(24, 316, 'Also look for byte order (00 00 01 2C is 300 big-endian), floats (00 00 80 3F is 1.0) and UTF-16 text.', 11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- files: two saves compared
def save_diff():
    s = Svg(900, 270, 'Two saves made before and after one change in the game. The bytes that differ hold the gold, which goes from 100 to 150 as a 16-bit little-endian number')
    s.heading('Comparing two saves',
              'Save, change one thing in the game and save again. The bytes that changed hold the value of that thing.')
    a = [0x53, 0x41, 0x56, 0x45, 0x02, 0x00, 0x0A, 0x00, 0x64, 0x00, 0x03, 0x00, 0x1F, 0x00, 0x00, 0x00]
    b = list(a)
    b[8] = 0x96
    for k, (name, data) in enumerate((('before, with 100 gold', a), ('after, with 150 gold', b))):
        y = 110 + k * 36
        s.text(24, y, name, 11.5, TEXT, weight='600')
        hexrow(s, 200, y, data, {8: AMBER, 9: AMBER} if True else None, cw=36)
    s.text(200 + 8 * 36, 186, 'at offset 8, 64 00 = 100 becomes 96 00 = 150', 11.5, AMBER)
    lines(s, 24, 222, ['This is an example. cmp -l lists the bytes that differ. Make a third save with another value to confirm the field.',
                       'similar.py finds how much two files share, even when blocks were inserted or moved.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- files: constants that name algorithms
def constants():
    s = Svg(900, 300, 'Numbers that name an algorithm, such as the CRC-32 table, the MD5 start values, the Microsoft C rand multiplier and increment, the FNV prime and the LZSS window start of 4078')
    s.heading('Constants that identify algorithms',
              'These numbers appear in few other places. constants.py searches for them.')
    rows = [('0x77073096', 'CRC-32', 'the second entry of its table'), ('0x67452301', 'MD5, SHA-1', 'the first start value'),
            ('214013, 2531011', 'Microsoft C rand()', 'multiplier and increment'),
            ('0x01000193', 'FNV-1 and FNV-1a, 32-bit', 'the prime'), ('4078 (0xFEE)', 'LZSS, the common decoder', 'where the window starts')]
    for k, (val, name, note) in enumerate(rows):
        y = 88 + k * 38
        s.rect(24, y, 852, 32, GREY, rx=4, extra=' fill-opacity="0.08"')
        s.text(40, y + 21, val, 12, AMBER, mono=True)
        s.text(250, y + 21, name, 12, TEXT, weight='600')
        s.text(520, y + 21, note, 11.5, DIM)
    s.text(24, 286, 'A hit names a function to confirm, often the random generator, a checksum of the saves or the decompressor.', 11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- files: Windows resources
def pe_resources():
    s = Svg(900, 344, 'Windows resources form a tree of type, id and language. String tables hold 16 strings per block. The code passes a string\'s id to LoadString')
    s.heading('Resources inside an executable',
              'The resources form a tree of type, id and language, with the data at the end. peres.py lists and extracts them.')
    types = [('RT_VERSION', 'version information'), ('RT_STRING', 'string tables'), ('RT_DIALOG', 'dialogs'),
             ('RT_ICON', 'icons'), ('RT_RCDATA', 'the developer\'s own data')]
    box(s, 24, 160, 100, 36, GREY, 'resources', op=0.3, size=11)
    for k, (t, d) in enumerate(types):
        y = 88 + k * 44
        s.add(f'<path d="M126 178C 150 178 150 {y + 18} 176 {y + 18}" fill="none" stroke="{DIM}" stroke-width="1.2"/>')
        box(s, 180, y, 220, 36, TEAL if t == 'RT_STRING' else BLUE, t, d, op=0.25, size=11, mono=True)
    s.add(f'<path d="M402 150L452 150" stroke="{DIM}" stroke-width="1.2" marker-end="url(#arrow)"/>')
    box(s, 456, 130, 190, 40, TEAL, 'block 7', 'ids 96 to 111', op=0.25, size=11)
    arrow(s, 648, 150, 686, 150)
    box(s, 690, 130, 186, 40, TEAL, 'language 0x409', 'the strings', op=0.25, size=11)
    lines(s, 456, 206, ['A string table block holds 16 strings.', 'Block n holds ids (n - 1) x 16 to (n - 1) x 16 + 15.',
                        'Search the code for an id to find where the', 'text is shown, as in LoadString(hInstance, 100, ...)'],
          11.5, DIM, gap=18)
    s.text(24, 330, 'Dialogs and menus here are the game\'s own windows. Their layouts and ids are evidence too.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- files: XOR obfuscation (animated)
def xor_scheme():
    s = Svg(900, 300, 'With XOR by one byte, text and zero padding become scrambled bytes. Every zero byte becomes the key itself, so a byte repeated many times often reveals the key')
    s.heading('XOR with one byte',
              'A zero byte XOR the key gives the key. A value that repeats many times in a file is often the key.')
    dur = 5.0
    plain = list(b'LEVEL 1\x00\x00\x00\x00\x00\x00')
    key = 0x5A
    enc = [b ^ key for b in plain]
    s.text(24, 104, 'plain', 11.5, TEXT, weight='600')
    s.text(24, 160, 'stored', 11.5, TEXT, weight='600')
    hexrow(s, 110, 104, plain, {k: GREY for k in range(7)} | {k: TEAL for k in range(7, 13)}, cw=34)
    for k, ch in enumerate('LEVEL 1'):
        s.text(110 + k * 34 + 15, 126, ch, 10.5, DIM, 'middle', mono=True)
    hexrow(s, 110, 160, enc, {k: GREY for k in range(7)} | {k: AMBER for k in range(7, 13)}, cw=34)
    s.text(560, 104, 'XOR 0x5A', 12, PURPLE, weight='600', mono=True)
    s.add(f'<rect x="106" y="92" width="34" height="80" rx="4" fill="none" stroke="{PURPLE}" stroke-width="2">'
          f'<animate attributeName="x" values="{";".join(str(106 + k * 34) for k in range(13))}" '
          f'keyTimes="{";".join(f"{k / 13:.4f}" for k in range(13))}" calcMode="discrete" dur="{dur}s" repeatCount="indefinite"/></rect>')
    s.text(110 + 7 * 34, 190, 'six zeros became 5A, the key', 11, AMBER)
    lines(s, 24, 236, ['Other weak schemes are a short repeating key, adding a constant, a key that changes by a fixed step, each byte',
                       'XORed with the one before. xorscan.py scores each one by how much of the result reads as text or zeros.'],
          11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- files: timestamps
def timestamps():
    value = 0x2E6B7A40
    unix = datetime.datetime(1970, 1, 1) + datetime.timedelta(seconds=value)
    mac = datetime.datetime(1904, 1, 1) + datetime.timedelta(seconds=value)
    date, time = value >> 16, value & 0xFFFF
    fat = (1980 + (date >> 9), (date >> 5) & 15, date & 31, time >> 11, (time >> 5) & 63, (time & 31) * 2)
    s = Svg(900, 300, f'One 32-bit value, 0x{value:08X}, read as a Unix time, a Mac time and a FAT date and time, '
                      'which gives three different dates')
    s.heading('Reading one value as several kinds of timestamp',
              f'timestamps.py 0x{value:08X} prints every reading. Only the kind that the game uses gives the right date.')
    rows = [('Unix seconds since 1970', unix.strftime('%Y-%m-%d %H:%M:%S'), TEAL),
            ('Mac seconds since 1904', mac.strftime('%Y-%m-%d %H:%M:%S'), PURPLE),
            ('FAT date (high word) and time', f'{fat[0]}-{fat[1]:02d}-{fat[2]:02d} {fat[3]:02d}:{fat[4]:02d}:{fat[5]:02d}', AMBER)]
    s.text(24, 104, f'0x{value:08X}', 16, TEXT, weight='600', mono=True)
    for k, (kind, when, colour) in enumerate(rows):
        y = 92 + k * 44
        s.add(f'<path d="M150 99C 190 99 190 {y + 16} 230 {y + 16}" fill="none" stroke="{DIM}" stroke-width="1.2" '
              f'marker-end="url(#arrow)"/>')
        s.rect(234, y, 642, 34, colour, rx=4, extra=' fill-opacity="0.16"')
        s.text(248, y + 22, kind, 12, TEXT)
        s.text(560, y + 22, when, 12, TEXT, weight='600', mono=True)
    lines(s, 24, 246, ['Windows FILETIME (100 ns steps since 1601) and OLE dates (days since 1899-12-30, a double) need 64 bits.',
                       'A field is real when a later save has a later value at the same offset. FAT and Mac times are local time.'],
          11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- protection: entropy of a packed program
def packed_entropy():
    s = Svg(900, 330, 'The entropy of each block of an executable. An ordinary program varies between code, data and resources. A packed program is above 7.5 almost everywhere after a small stub')
    s.heading('The entropy of a packed program',
              'filetypes.py --blocks 4096 gives the entropy of each block, from 0 to 8 bits per byte.')
    import random
    rnd = random.Random(11)
    normal = [5.8 + rnd.random() * 0.6 for _ in range(18)] + [3.0 + rnd.random() for _ in range(8)] + \
             [1.0 + rnd.random() * 2 for _ in range(6)] + [6.5 + rnd.random() * 0.8 for _ in range(8)]
    packed = [5.9, 6.1] + [7.8 + rnd.random() * 0.18 for _ in range(36)] + [3.1, 2.4]
    for r, (name, vals, colour) in enumerate((('an ordinary program', normal, TEAL), ('a packed program', packed, RED))):
        y0 = 100 + r * 110
        s.text(24, y0 + 10, name, 12, TEXT, weight='600')
        s.line(200, y0 + 80, 860, y0 + 80, EDGE, 1)
        s.line(200, y0 + 80 - 7.5 * 10, 860, y0 + 80 - 7.5 * 10, AMBER, 1, ' stroke-dasharray="4 3"')
        s.text(194, y0 + 9, '7.5', 10, AMBER, 'end')
        for k, v in enumerate(vals):
            s.rect(200 + k * 16.5, y0 + 80 - v * 10, 14, v * 10, colour, rx=1, extra=' fill-opacity="0.55"')
    s.text(24, 140, 'code, data, resources', 10.5, DIM)
    s.text(24, 250, 'a stub, then compressed', 10.5, DIM)
    s.text(24, 316, 'The bars are illustrative shapes. Other signs are very few imports, sections named UPX0 and UPX1, and almost no strings.',
           11.5, DIM)
    return s.render()


# ---------------------------------------------------------------- protection: a data decoder (animated)
def decoder_loop():
    s = Svg(900, 300, 'A data decoder in the code. The file is read into a buffer. A tight loop then changes each byte with constants as keys. Stopping just after the loop gives the plain data')
    s.heading('Letting the game decode its data',
              'Right after the file is read, a tight loop changes every byte. Stop the program after the loop and save the buffer.')
    dur = 5.0
    box(s, 24, 96, 150, 50, GREY, 'ReadFile', 'the scrambled bytes', op=0.3)
    arrow(s, 176, 121, 206, 121)
    box(s, 210, 90, 250, 62, PURPLE, None, op=0.2)
    lines(s, 222, 110, ['for each byte of the buffer:', '  b = rotate(b ^ key, 3) + k'], 11, TEXT, mono=True, gap=18)
    s.text(222, 146, 'illustrative', 9.5, DIM)
    arrow(s, 462, 121, 492, 121)
    box(s, 496, 96, 160, 50, AMBER, 'stop here', 'save the buffer', op=0.3)
    n = 24
    for k in range(n):
        x = 24 + k * 35
        s.rect(x, 190, 32, 26, RED, rx=2, extra=' fill-opacity="0.3"')
        s.add(f'<rect x="{x}" y="190" width="32" height="26" rx="2" fill="{TEAL}" opacity="0">{during(0.8 * k / n, 0.97, dur, on="0.7")}</rect>')
    s.text(24, 236, 'the buffer, which turns from scrambled (red) to plain (teal) as the loop runs', 10.5, DIM)
    s.text(24, 278, 'Port the decoder like any function, with its constants and order of operations, and check it by emulation.', 11.5, TEXT)
    return s.render()


# ---------------------------------------------------------------- protection: flattened control flow (animated)
def flattening():
    s = Svg(900, 340, 'With control flow flattening, the blocks A, B, C and D of an ordinary function become cases of one switch inside a loop. A state variable chooses the next block')
    s.heading('A function with flattened control flow',
              'Every block becomes a case of one switch inside a loop. A state variable picks the next block.')
    dur = 6.0
    s.text(24, 92, 'as written', 11.5, TEXT, weight='600')
    pos = {'A': (110, 120), 'B': (60, 190), 'C': (160, 190), 'D': (110, 260)}
    for a, b in (('A', 'B'), ('A', 'C'), ('B', 'D'), ('C', 'D')):
        arrow(s, pos[a][0], pos[a][1] + 14, pos[b][0], pos[b][1] - 16)
    for k, (x, y) in pos.items():
        box(s, x - 30, y - 14, 60, 28, BLUE, k, op=0.3, size=12)
    s.text(320, 92, 'flattened', 11.5, TEXT, weight='600')
    box(s, 320, 104, 220, 40, PURPLE, 'loop: switch (state)', op=0.3, size=11.5, mono=True)
    cases = [('A', 'state = x ? 2 : 3'), ('B', 'state = 4'), ('C', 'state = 4'), ('D', 'return')]
    order = ['A', 'C', 'D']
    s.line(316, 144, 316, 170 + 3 * 38 + 15, DIM, 1.2)
    s.line(330, 124, 316, 124, DIM, 1.2)
    for k, (name, nxt) in enumerate(cases):
        y = 170 + k * 38
        s.line(316, y + 15, 328, y + 15, DIM, 1.2)
        box(s, 330, y, 90, 30, BLUE, f'case {k + 1}: {name}', op=0.3, size=10.5)
        s.text(430, y + 20, nxt, 10.5, DIM, mono=True)
        if name in order:
            j = order.index(name)
            s.add(f'<rect x="327" y="{y - 3}" width="96" height="36" rx="5" fill="none" stroke="{AMBER}" stroke-width="2.5" '
                  f'opacity="0">{during(j / 3, (j + 1) / 3, dur)}</rect>')
    lines(s, 620, 110, ['Ghidra shows one loop with', 'many cases and no visible', 'structure. Follow the state', 'values to rebuild the order.',
                        '', 'In games this code is usually', 'the protection around the', 'game. Find where it ends.'],
          11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- protection: a timing check (animated)
def timing_check():
    s = Svg(900, 300, 'In a timing check, the program reads a clock before and after a short piece of code. When the program runs normally, the gap is small. Under a debugger, the gap is long and the check fails')
    s.heading('A timing check against debuggers',
              'The program reads a clock, runs a little code and reads the clock again. Under a debugger, the gap is long.')
    dur = 6.0
    for r, (name, w, colour, verdict) in enumerate((('run normally', 60, TEAL, 'the gap is small, so the program carries on'),
                                                     ('under a debugger, stepping', 440, RED, 'the gap is long, so the check fails'))):
        y = 100 + r * 80
        s.text(24, y + 20, name, 12, TEXT, weight='600')
        s.rect(240, y, 6, 32, AMBER, rx=1)
        s.text(243, y - 6, 'read clock', 10, AMBER, 'middle')
        s.add(f'<rect x="250" y="{y + 4}" height="24" rx="3" fill="{colour}" fill-opacity="0.45" width="0">'
              f'<animate attributeName="width" values="0;{w};{w}" keyTimes="0;0.6;1" dur="{dur}s" repeatCount="indefinite"/></rect>')
        s.rect(254 + w, y, 6, 32, AMBER, rx=1)
        s.text(257 + w, y - 6, 'read clock', 10, AMBER, 'middle')
        s.text(270 + w, y + 21, verdict, 11, colour) if w < 200 else s.text(250, y + 50, verdict, 11, colour)
    lines(s, 24, 262, ['re-emulate calls one function with chosen inputs, so start-up checks never run. Note any check made during play,',
                       'with its address, so the port can leave it out.'], 11.5, DIM, gap=18)
    return s.render()


# ---------------------------------------------------------------- protection: copy protection by era
def protection_eras():
    s = Svg(900, 300, 'Copy protection by era. Floppies used manual lookups, code wheels and key disks. CDs used disc checks, SafeDisc, SecuROM and StarForce. Online games use activation and Denuvo. The port needs none of it')
    s.heading('Copy protection in each era',
              'The port leaves out every protection check. Mark each check that it leaves out as not ported.')
    eras = [('Floppy era', 'manual lookups, code wheels,', 'key disks with odd sectors', AMBER),
            ('CD era', 'disc label, file or TOC checks,', 'SafeDisc, SecuROM, StarForce', BLUE),
            ('Online era', 'activation, account checks,', 'Denuvo, store wrappers', PURPLE)]
    for k, (name, a, b, colour) in enumerate(eras):
        x = 24 + k * 288
        s.rect(x, 88, 276, 92, colour, rx=5, extra=' fill-opacity="0.18"')
        s.text(x + 14, 112, name, 13, TEXT, weight='600')
        s.text(x + 14, 136, a, 11.5, DIM)
        s.text(x + 14, 154, b, 11.5, DIM)
        if k < 2:
            arrow(s, x + 278, 134, x + 286, 134)
    s.rect(24, 200, 852, 44, TEAL, rx=5, extra=' fill-opacity="0.12"')
    s.text(40, 227, 'The port needs nothing from any of them. A manual lookup is part of the game\'s text. The project owner decides whether to keep it.',
           11.5, TEXT)
    s.text(24, 278, 'Prefer a release without protection. Many digital re-releases ship without the disc checks.', 11.5, DIM)
    return s.render()


DIAGRAMS = {
    'media-overview.svg': overview, 'media-disc-kinds.svg': disc_kinds, 'media-iso9660.svg': iso9660,
    'media-disc-images.svg': disc_images, 'media-booter.svg': booter, 'media-magic.svg': magic,
    'media-raw-width.svg': raw_width, 'media-structure.svg': structure, 'media-save-diff.svg': save_diff,
    'media-constants.svg': constants, 'media-pe-resources.svg': pe_resources, 'media-xor.svg': xor_scheme,
    'media-timestamps.svg': timestamps, 'media-packed-entropy.svg': packed_entropy,
    'media-decoder-loop.svg': decoder_loop, 'media-flattening.svg': flattening, 'media-timing-check.svg': timing_check,
    'media-protection-eras.svg': protection_eras,
}
