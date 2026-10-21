"""A small DOS, BIOS and PC for tools/emu/dos_emu.py, so a whole 16-bit program can run headless.

    from dos_emu import DosEmu, ProgramStopped
    from dos_env import DosEnv
    emu = DosEmu('target/game/GAME.EXE')
    env = DosEnv(emu, root='target/game', overlay='derived/run')   # files, KERNEL, ports, BIOS
    try:
        emu.run_range(emu.entry, '1000:ca59')      # from the program's entry to a chosen point
    except ProgramStopped as stop:
        print('exit code', stop.args[0])
    print(env.log[-20:])                           # what the program asked of DOS, in order

What it provides:
  - DOS (INT 21h): files under `root` (names matched without case). A file the program writes
    is saved to `overlay` when closed (and kept in `env.written`); root is never written, and
    a file only opened read/write (Pascal's default) is not saved at all. Later opens read the
    overlay's copy first. handles, seek, find first/next with the
    DTA, current drive and directory, date and time (fixed: `env.clock`, so runs repeat
    exactly), version 5.0, interrupt vectors (kept in a table; nothing is ever called through
    them), device information, and exit (ends the run with ProgramStopped(exit code)).
  - DPMI16 KERNEL imports (Borland Pascal 7, Windows 3.x): GlobalAlloc / GlobalLock (64 KB
    selectors in a row, 8 apart, for blocks over 64 KB), GlobalFree, GetFreeSpace, InitTask.
  - Ports: VGA status (0x3DA) alternates so retrace waits end; the DAC (0x3C8/0x3C9) is kept
    in `env.dac` (256 x RGB, 6-bit); PIT, PIC, keyboard, joystick, Sound Blaster and AdLib
    ports answer with safe constant values. Every port write is counted in `env.ports`.
  - BIOS: INT 10h (video modes, recorded in `env.video_mode`), INT 16h (no key waiting),
    INT 33h (no mouse), INT 1Ah and the tick count at 0040:006C (`env.ticks`, one tick per
    128 reads of it, so wait-for-the-next-tick loops end).
  - DPMI (INT 31h): descriptors, DOS memory, interrupt and exception vectors, version, memory
    information; calls into real mode are skipped. INT 2Fh answers "nothing installed".
  - The timer: IRQ 0 goes to INT 8's handler (the program's, or the BIOS's, which counts the
    tick) at the rate the program sets on the PIT, counted in instructions: `ips` per second
    (default 25 million, a fast 486). `env.irq0_count` counts deliveries.
Anything else stops the run with its name (the emulator's own message), so you know what to
add. It does not emulate sound output, a real keyboard (use env.keys), or DPMI services
beyond what is listed.
"""
import os
import struct

from unicorn import UC_HOOK_INSN, UC_HOOK_MEM_READ
from unicorn.x86_const import UC_X86_INS_IN, UC_X86_INS_OUT, UC_X86_REG_FLAGS

from dos_emu import FarPtr

CF = 1


class DosEnv:
    def __init__(self, emu, root, overlay=None, clock=(1999, 8, 31, 12, 0, 0, 0), args='', ips=25_000_000,
                 program='C:\\GAME.EXE', environment=('PATH=C:\\', 'COMSPEC=C:\\COMMAND.COM')):
        self.emu, self.root = emu, root
        self.overlay = overlay
        self.clock = clock              # year, month, day, hour, minute, second, hundredths
        self.ticks = 0
        self.files = {}                 # handle -> [path, data(bytearray), pos, writable]
        self.next_handle = 5
        self.vectors = {}
        self.dta = None
        self.find = None
        self.log = []
        self.stdout = bytearray()
        self.dac = bytearray(768)
        self.dac_index = self.dac_read = 0
        self.video_mode = None
        self.ports = {}
        self.retrace = 0
        self.pit, self.pit_high = 0xFFFF, False
        emu.interrupts.update({0x21: self.int21, 0x10: self.int10, 0x16: self.int16,
                               0x33: self.int33, 0x1A: self.int1a, 0x31: self.int31,
                               0x2F: self.int2f})
        self.dos_mem = 0x2000                 # next free real-mode paragraph for DOS memory
        self.pm_vectors, self.rm_vectors, self.exc_vectors = {}, {}, {}
        self.callbacks = []
        self.keys = []                        # BIOS keyboard queue: scan << 8 | ascii
        self.written = {}                     # files the program wrote: NAME -> bytes
        emu.stubs.update({'KERNEL.15': self.global_alloc, 'KERNEL.17': self.global_free,
                          'KERNEL.18': self.global_lock, 'KERNEL.169': self.get_free_space,
                          'KERNEL.91': self.init_task,
                          'RTM.5': self.rtm_alloc, 'RTM.6': self.rtm_free, 'RTM.21': self.rtm_free_mem})
        self.init_bios_data()
        self.psp = self.make_psp(args, program, environment)
        # The BIOS's own interrupt handlers: an IRET for every vector (INT 8 also counts the
        # BIOS tick), so a program that chains to the old handler returns cleanly.
        self.bios = emu.new_selector(access=0x9A)
        emu.write(FarPtr(self.bios, 0), b'\xCF' * 0x400)
        from unicorn import UC_HOOK_CODE
        at = emu.linear(self.bios, 8 * 4)
        emu.uc.hook_add(UC_HOOK_CODE, self.bios_int8, begin=at, end=at)
        # The timer: IRQ 0 every (instructions per second / tick rate) instructions.
        self.ips = ips
        self.divisor = 0x10000
        self.pit_write_high = False
        self.irq0_count = 0
        emu.on_slice = self.on_slice
        self.set_timer_slice()
        emu.uc.hook_add(UC_HOOK_MEM_READ, self.bios_tick_read, begin=0x46C, end=0x46F)
        self.tick_reads = 0
        emu.uc.hook_add(UC_HOOK_INSN, self.port_in, None, 1, 0, UC_X86_INS_IN)
        emu.uc.hook_add(UC_HOOK_INSN, self.port_out, None, 1, 0, UC_X86_INS_OUT)

    # ------------------------------------------------------------------ helpers

    def carry(self, on):
        f = self.emu.reg('flags')
        self.emu.set(flags=(f | CF) if on else (f & ~CF))

    def fail(self, code):
        self.emu.set(ax=code)
        self.carry(True)

    def asciiz(self, seg, off):
        out = bytearray()
        while len(out) < 260:
            b = self.emu.read((seg, (off + len(out)) & 0xFFFF), 1)[0]
            if b == 0:
                break
            out.append(b)
        return out.decode('latin-1')

    def host_path(self, dos_name, for_write=False):
        """Find a DOS name under root (or overlay), ignoring case; None if missing."""
        name = dos_name.replace('\\', '/').split(':')[-1].lstrip('/')
        for base in ([self.overlay] if self.overlay else []) + [self.root]:
            if base is None:
                continue
            path = base
            for part in [p for p in name.split('/') if p and p != '.']:
                if not os.path.isdir(path):
                    path = None
                    break
                match = [e for e in os.listdir(path) if e.lower() == part.lower()]
                path = os.path.join(path, match[0]) if match else None
                if path is None:
                    break
            if path and os.path.exists(path):
                return path
        if for_write and self.overlay:
            os.makedirs(self.overlay, exist_ok=True)
            return os.path.join(self.overlay, os.path.basename(name).upper())
        return None

    # ------------------------------------------------------------------ INT 21h

    def int21(self, emu):
        ah, al = emu.reg('ax') >> 8, emu.reg('ax') & 0xFF
        ds, dx, bx, cx = emu.reg('ds'), emu.reg('dx'), emu.reg('bx'), emu.reg('cx')
        self.carry(False)
        if ah in (0x02, 0x06):
            self.stdout.append(dx & 0xFF)
        elif ah == 0x09:
            s = bytearray()
            while len(s) < 4096:
                b = emu.read((ds, (dx + len(s)) & 0xFFFF), 1)[0]
                if b == ord('$'):
                    break
                s.append(b)
            self.stdout += s
        elif ah == 0x0B:
            emu.set(ax=(emu.reg('ax') & 0xFF00) | (0xFF if self.keys else 0))
        elif ah in (0x01, 0x07, 0x08, 0x0C):
            if ah == 0x0C:                                  # flush, then the function in AL
                if al not in (0x01, 0x07, 0x08):
                    return
            if not self.keys:
                emu.error = 'the program waits for a key (INT 21h): put keys in env.keys'
                emu.uc.emu_stop()
                return
            emu.set(ax=(emu.reg('ax') & 0xFF00) | (self.keys.pop(0) & 0xFF))
        elif ah == 0x0E:
            emu.set(ax=(emu.reg('ax') & 0xFF00) | 26)
        elif ah == 0x19:
            emu.set(ax=(emu.reg('ax') & 0xFF00) | 2)             # drive C:
        elif ah == 0x1A:
            self.dta = FarPtr(ds, dx)
        elif ah == 0x25:
            self.vectors[al] = FarPtr(ds, dx)
        elif ah == 0x35:
            v = self.vector(al)
            emu.set(es=v.seg, bx=v.off)
        elif ah == 0x2A:
            y, m, d = self.clock[:3]
            emu.set(cx=y, dx=(m << 8) | d, ax=(emu.reg('ax') & 0xFF00) | 2)
        elif ah == 0x2C:
            h, mi, sec, hs = self.clock[3:]
            emu.set(cx=(h << 8) | mi, dx=(sec << 8) | hs)
        elif ah == 0x30:
            emu.set(ax=0x0005, bx=0, cx=0)
        elif ah == 0x3B:
            pass                                                # chdir: one folder only
        elif ah in (0x3C, 0x3D, 0x5B):
            name = self.asciiz(ds, dx)
            if ah == 0x3D:
                path = self.host_path(name)
                if path is None or not os.path.isfile(path):
                    self.log.append(f'open {name}: not found')
                    return self.fail(2)
                data = bytearray(open(path, 'rb').read())
            else:
                data = bytearray()                      # create: an empty file
            # Writes never reach root: a file is saved (to overlay, or nowhere) only when the
            # program writes to it. Pascal's Reset opens files read/write (FileMode 2), so the
            # open mode alone says nothing.
            h = self.next_handle
            self.next_handle += 1
            self.files[h] = [name, data, 0, ah != 0x3D]   # name, bytes, position, dirty
            self.log.append(f'{"create" if ah != 0x3D else "open"} {name} -> {h}')
            emu.set(ax=h)
        elif ah == 0x3E:
            f = self.files.pop(bx, None)
            if f and f[3]:
                self.save(f[0], f[1])
        elif ah == 0x3F:
            if bx == 0:
                return emu.set(ax=0)
            f = self.files.get(bx)
            if f is None:
                return self.fail(6)
            chunk = bytes(f[1][f[2]:f[2] + cx])
            emu.write((ds, dx), chunk)
            f[2] += len(chunk)
            emu.set(ax=len(chunk))
        elif ah == 0x40:
            data = emu.read((ds, dx), cx)
            if bx in (1, 2):
                self.stdout += data
            else:
                f = self.files.get(bx)
                if f is None:
                    return self.fail(6)
                f[1][f[2]:f[2] + cx] = data
                f[2] += cx
                f[3] = True                                 # dirty: saved on close
            emu.set(ax=cx)
        elif ah == 0x41:
            pass
        elif ah == 0x42:
            f = self.files.get(bx)
            if f is None:
                return self.fail(6)
            off = struct.unpack('<i', struct.pack('<HH', dx, cx))[0]
            base = (0, f[2], len(f[1]))[al] if al < 3 else 0
            f[2] = max(0, base + off)
            emu.set(ax=f[2] & 0xFFFF, dx=f[2] >> 16)
        elif ah == 0x43:
            emu.set(cx=0x20)
        elif ah == 0x44:
            if al == 0:
                emu.set(dx=0x80 | 0x03 if bx < 5 else 0x02)
            else:
                emu.set(ax=0)
        elif ah == 0x47:
            emu.write((ds, emu.reg('si')), b'\0')
        elif ah == 0x4C:
            self.log.append(f'exit {al}')
            emu.stop(al)
        elif ah == 0x4E:
            self.find_first(self.asciiz(ds, dx), cx)
        elif ah == 0x4F:
            self.find_next()
        elif ah in (0x51, 0x62):
            emu.set(bx=self.psp)
        else:
            emu.error = f'INT 21h AH={ah:02X}h (AL={al:02X}h) is not provided by dos_env'
            emu.uc.emu_stop()

    def save(self, name, data):
        """A written file goes to overlay (never to root); without an overlay it is kept only
        in memory (env.written)."""
        self.written[os.path.basename(name.replace('\\', '/')).upper()] = bytes(data)
        if self.overlay:
            os.makedirs(self.overlay, exist_ok=True)
            with open(os.path.join(self.overlay, os.path.basename(name.replace('\\', '/')).upper()), 'wb') as out:
                out.write(data)
        self.log.append(f'save {name} ({len(data)} bytes)')

    def find_first(self, pattern, attrs):
        import fnmatch
        pat = pattern.replace('\\', '/').split('/')[-1].lower().replace('*.*', '*')
        names = sorted(set(os.listdir(self.root)) | (set(os.listdir(self.overlay)) if self.overlay and os.path.isdir(self.overlay) else set()))
        self.find = [n for n in names if fnmatch.fnmatch(n.lower(), pat) and os.path.isfile(self.host_path(n) or '')]
        self.log.append(f'find {pattern}: {len(self.find)}')
        self.find_next()

    def find_next(self):
        if not self.find:
            return self.fail(18)
        name = self.find.pop(0)
        size = os.path.getsize(self.host_path(name))
        dta = self.dta or FarPtr(self.emu.regs['ds'], 0x80)
        rec = bytearray(43)
        rec[21] = 0x20
        struct.pack_into('<I', rec, 26, size)
        rec[30:30 + 13] = name.upper().encode('latin-1')[:12].ljust(13, b'\0')
        self.emu.write(dta, rec)
        self.emu.set(ax=0)

    # ------------------------------------------------------------------ DPMI (INT 31h)

    def int31(self, emu):
        ax, bx, cx, dx = emu.reg('ax'), emu.reg('bx'), emu.reg('cx'), emu.reg('dx')
        self.carry(False)
        if ax == 0x0000:                                  # allocate CX descriptors
            first = emu.new_selector()
            for _ in range(max(cx, 1) - 1):
                emu.new_selector()
            emu.set(ax=first)
        elif ax in (0x0001, 0x0101, 0x0502, 0x0600, 0x0601, 0x0009, 0x0008, 0x0603, 0x0604):
            pass                                          # free, unlock, limits, access rights
        elif ax == 0x0002:                                # real-mode segment -> selector
            emu.set(ax=emu.new_selector(base=bx << 4))
        elif ax == 0x0003:
            emu.set(ax=8)                                 # selector increment
        elif ax == 0x0006:
            base = emu.linear(bx, 0)
            emu.set(cx=base >> 16, dx=base & 0xFFFF)
        elif ax == 0x0007:
            emu.set_descriptor(bx, (cx << 16) | dx)
        elif ax == 0x000A:                                # alias: a data selector, same base
            emu.set(ax=emu.new_selector(base=emu.linear(bx, 0)))
        elif ax == 0x0100:                                # DOS memory: BX paragraphs
            seg = self.dos_mem
            self.dos_mem += bx
            emu.set(ax=seg, dx=emu.new_selector(base=seg << 4))
        elif ax == 0x0200:
            v = self.rm_vectors.get(bx & 0xFF, FarPtr(0, 0))
            emu.set(cx=v.seg, dx=v.off)
        elif ax == 0x0201:
            self.rm_vectors[bx & 0xFF] = FarPtr(cx, dx)
        elif ax == 0x0204:
            v = self.vector(bx & 0xFF)
            emu.set(cx=v.seg, dx=v.off)
        elif ax == 0x0205:
            self.pm_vectors[bx & 0xFF] = FarPtr(cx, dx)
        elif ax == 0x0202:
            v = self.exc_vectors.get(bx & 0xFF, FarPtr(0, 0))
            emu.set(cx=v.seg, dx=v.off)
        elif ax == 0x0203:
            self.exc_vectors[bx & 0xFF] = FarPtr(cx, dx)
        elif ax in (0x0300, 0x0301, 0x0302):              # call real mode: nothing to call
            self.log.append(f'dpmi {ax:04x}: real-mode call skipped')
        elif ax == 0x0303:                                # real-mode callback address
            emu.set(cx=0xF000, dx=0x0100 + 4 * len(self.callbacks))
            self.callbacks.append(FarPtr(emu.reg('ds'), emu.reg('si')))
        elif ax in (0x0304, 0x0E01, 0x0900, 0x0901):
            if ax in (0x0900, 0x0901):
                emu.set(ax=(ax & 0xFF00) | 1)
        elif ax == 0x0902:
            emu.set(ax=1)
        elif ax in (0x0305, 0x0306):
            emu.set(ax=0, bx=0xF000, cx=0x0200, si=self.stub_selector(), di=0x0200)
        elif ax == 0x0507:
            pass
        elif ax == 0x0800:                                # physical address: identity map
            emu.set(bx=bx, cx=cx)
        elif ax == 0x0E00:
            emu.set(ax=0x0045)                            # an FPU is present (387)
        elif ax == 0x0A00:
            return self.fail(0x8001)                      # no vendor extensions
        elif ax == 0x0400:
            emu.set(ax=0x005A, bx=0x0005, cx=0x0003, dx=0x0870)   # DPMI 0.90, 286 host, 386
        elif ax == 0x0500:
            info = struct.pack('<12I', 0x00F00000, 0xF00, 0xF00, 0xF00, 0xF00, 0xF00, 0xF00, 0xF00, 0, 0, 0, 0)
            emu.write((emu.reg('es'), emu.reg('di')), info[:48])
        else:
            emu.error = f'INT 31h (DPMI) AX={ax:04X}h is not provided by dos_env'
            emu.uc.emu_stop()
        self.log.append(f'dpmi {ax:04x}')

    def stub_selector(self):
        return self.emu.stub_seg

    # ------------------------------------------------------------------ BIOS

    def int10(self, emu):
        """Video BIOS: modes, the text cursor and text output on the 0xB8000 screen."""
        ax, bx, cx, dx = emu.reg('ax'), emu.reg('bx'), emu.reg('cx'), emu.reg('dx')
        ah, al = ax >> 8, ax & 0xFF
        uc = emu.uc
        if ah == 0x00:
            self.video_mode = al & 0x7F
            uc.mem_write(0x449, bytes([self.video_mode]))
            if self.video_mode == 3 and not al & 0x80:
                uc.mem_write(0xB8000, b'\x20\x07' * 2000)
            self.cursor = (0, 0)
        elif ah == 0x01:
            self.cursor_shape = cx
        elif ah == 0x02:
            self.cursor = (dx >> 8, dx & 0xFF)
        elif ah == 0x03:
            row, col = getattr(self, 'cursor', (0, 0))
            emu.set(dx=(row << 8) | col, cx=getattr(self, 'cursor_shape', 0x0607))
        elif ah in (0x09, 0x0A):
            row, col = getattr(self, 'cursor', (0, 0))
            at = 0xB8000 + (row * 80 + col) * 2
            for i in range(cx):
                uc.mem_write(at + 2 * i, bytes([al]) if ah == 0x0A else bytes([al, bx & 0xFF]))
        elif ah == 0x0E:
            row, col = getattr(self, 'cursor', (0, 0))
            if al == 13:
                col = 0
            elif al == 10:
                row += 1
            elif al == 8:
                col = max(0, col - 1)
            else:
                uc.mem_write(0xB8000 + (row * 80 + col) * 2, bytes([al]))
                col += 1
            if col >= 80:
                row, col = row + 1, 0
            if row >= 25:
                screen = bytes(uc.mem_read(0xB8000 + 160, 3840))
                uc.mem_write(0xB8000, screen + b'\x20\x07' * 80)
                row = 24
            self.cursor = (row, col)
        elif ah == 0x08:
            row, col = getattr(self, 'cursor', (0, 0))
            emu.set(ax=int.from_bytes(uc.mem_read(0xB8000 + (row * 80 + col) * 2, 2), 'little'))
        elif ah == 0x0F:
            emu.set(ax=(80 << 8) | (self.video_mode if self.video_mode is not None else 3), bx=bx & 0xFF)
        elif ah == 0x1A:
            emu.set(ax=0x001A, bx=0x0008)                 # VGA with colour display
        elif ah == 0x12 and (bx & 0xFF) == 0x10:
            emu.set(bx=0x0003, cx=0x0009)                 # EGA/VGA, 256 KB, colour
        elif ah == 0x11 and al == 0x30:
            emu.set(cx=16, dx=24)                         # character height, rows - 1
        self.log.append(f'int 10h ah={ah:02x}h al={al:02x} dx={dx:04x} cx={cx:04x} bx={bx:04x}')

    def save_screen(self, path, seg=None):
        """Write the 320 x 200 mode 13h screen (VGA memory at 0xA0000, or a 64 KB segment the
        program draws into) as a PNG, coloured with the DAC palette the program set."""
        import zlib
        pix = bytes(self.emu.uc.mem_read(0xA0000, 64000)) if seg is None else self.emu.read((seg, 0), 64000)
        rgb = bytes(c << 2 | c >> 4 for c in self.dac)
        raw = b''.join(b'\0' + b''.join(rgb[p * 3:p * 3 + 3] for p in pix[y * 320:(y + 1) * 320]) for y in range(200))
        def chunk(kind, data):
            return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
        png = b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', 320, 200, 8, 2, 0, 0, 0))
        png += chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b'')
        with open(path, 'wb') as f:
            f.write(png)

    def screen_text(self):
        """The 80 x 25 text screen as lines (what a text-mode message says)."""
        raw = bytes(self.emu.uc.mem_read(0xB8000, 4000))[::2].decode('cp437', 'replace')
        return [raw[i:i + 80].rstrip() for i in range(0, 2000, 80)]

    def int16(self, emu):
        """Keyboard BIOS with a queue: env.keys holds (scan << 8 | ascii) words. Reading a key
        when the queue is empty stops the run (a real BIOS would wait), so a script decides what
        is typed; AX = 0 would be Ctrl-Break."""
        ah = emu.reg('ax') >> 8
        flags = emu.reg('flags')
        if ah in (0x01, 0x11):
            if self.keys:
                emu.set(ax=self.keys[0], flags=flags & ~0x40)
            else:
                emu.set(flags=flags | 0x40)            # ZF: no key waiting
        elif ah in (0x00, 0x10):
            if not self.keys:
                emu.error = 'the program waits for a key (INT 16h): put keys in env.keys'
                emu.uc.emu_stop()
                return
            emu.set(ax=self.keys.pop(0))
        elif ah in (0x02, 0x12):
            emu.set(ax=0)                              # shift states: none

    def int2f(self, emu):
        """Multiplex: nothing installed (registers left as they are, which callers read as
        "not present": no Windows, no RTM extensions, no XMS driver)."""
        self.log.append(f'int 2fh ax={emu.reg("ax"):04x}h: not present')

    def int33(self, emu):
        emu.set(ax=0)                                 # no mouse driver

    def int1a(self, emu):
        if emu.reg('ax') >> 8 == 0:
            emu.set(cx=self.ticks >> 16, dx=self.ticks & 0xFFFF, ax=0)

    def make_psp(self, args, program, environment):
        """A program segment prefix: INT 20h at 0, the environment's selector at 0x2C, the
        command line at 0x80. The environment block ends with the program's path (ParamStr(0))."""
        emu = self.emu
        env_block = b''.join(e.encode('latin-1') + b'\0' for e in environment) + b'\0'
        env_block += b'\x01\x00' + program.encode('latin-1') + b'\0'
        env_sel = emu.new_segment(env_block)
        tail = args.encode('latin-1')[:126]
        psp = bytearray(256)
        psp[0:2] = b'\xCD\x20'
        struct.pack_into('<H', psp, 2, 0xA000)            # top of memory, as a paragraph
        struct.pack_into('<H', psp, 0x2C, env_sel)
        psp[0x80] = len(tail)
        psp[0x81:0x81 + len(tail) + 1] = tail + b'\r'
        return emu.new_segment(bytes(psp))

    def init_bios_data(self):
        """The BIOS data area (0040:0000) as a PC with colour VGA in text mode 3 has it."""
        uc = self.emu.uc
        uc.mem_write(0x410, struct.pack('<H', 0x0027))   # equipment: colour 80x25, FPU, floppy
        uc.mem_write(0x413, struct.pack('<H', 640))      # KB of conventional memory
        uc.mem_write(0x449, bytes([3]))                  # video mode
        uc.mem_write(0x44A, struct.pack('<H', 80))       # columns
        uc.mem_write(0x44C, struct.pack('<H', 4096))     # bytes per text page
        uc.mem_write(0x463, struct.pack('<H', 0x3D4))    # CRT controller port (status at +6)
        uc.mem_write(0x484, bytes([24, 16, 0]))          # rows - 1, character height
        uc.mem_write(0x487, bytes([0x60, 0x09]))         # EGA/VGA flags

    def vector(self, n):
        """Where interrupt n goes: the program's protected-mode handler, else one set with
        INT 21h AH=25h, else the BIOS's IRET."""
        return self.pm_vectors.get(n) or self.vectors.get(n) or FarPtr(self.bios, 4 * n)

    def set_timer_slice(self):
        hz = 1_193_182 / self.divisor
        self.emu.slice = max(1000, int(self.ips / hz))

    def on_slice(self, emu):
        """Between slices: deliver IRQ 0 (the timer) through INT 8's handler."""
        v = self.vector(8)
        if emu.interrupt(v.seg, v.off):
            self.irq0_count += 1

    def bios_int8(self, uc, address, size, _):
        self.ticks = (self.ticks + 1) & 0xFFFFFFFF
        uc.mem_write(0x46C, struct.pack('<I', self.ticks))

    def bios_tick_read(self, uc, access, address, size, value, _):
        """The BIOS tick count (0040:006C) advances one tick every 128 reads of it, so loops
        that wait for the next tick (Crt's Delay calibration) end."""
        self.tick_reads += 1
        if self.tick_reads % 128 == 0:
            self.ticks = (self.ticks + 1) & 0xFFFFFFFF
            uc.mem_write(0x46C, struct.pack('<I', self.ticks))

    # ------------------------------------------------------------------ ports

    def port_in(self, uc, port, size, _):
        if port == 0x40:
            # PIT channel 0: counts down a little at every read, low byte then high byte, so
            # delay loops that watch it (OPL write delays) end.
            if not self.pit_high:
                self.pit = (self.pit - 7) & 0xFFFF
            v = (self.pit >> 8) if self.pit_high else (self.pit & 0xFF)
            self.pit_high = not self.pit_high
            return v
        if port == 0x3DA:
            self.retrace ^= 0x09
            return self.retrace
        if port == 0x3C9:
            v = self.dac[self.dac_read % 768]
            self.dac_read += 1
            return v
        if port == 0x201:
            return 0xFF
        if port in (0x21, 0xA1):
            return self.ports.get(('mask', port), 0)
        if port == 0x388:
            return 0x06
        return 0xFF if 0x220 <= port <= 0x22F else 0

    def port_out(self, uc, port, size, value, _):
        self.ports[port] = self.ports.get(port, 0) + 1
        if port == 0x3C8:
            self.dac_index = (value & 0xFF) * 3
        elif port == 0x3C7:
            self.dac_read = (value & 0xFF) * 3
        elif port == 0x3C9:
            self.dac[self.dac_index % 768] = value & 0x3F
            self.dac_index += 1
        elif port in (0x21, 0xA1):
            self.ports[('mask', port)] = value & 0xFF
        elif port == 0x43:
            self.pit_high = False                     # a latch or mode command restarts the read
            self.pit_write_high = False
        elif port == 0x40:
            if not self.pit_write_high:
                self.pit_low = value & 0xFF
            else:
                self.divisor = (self.pit_low | (value & 0xFF) << 8) or 0x10000
                self.set_timer_slice()
                self.log.append(f'timer {1_193_182 / self.divisor:.2f} Hz')
            self.pit_write_high = not self.pit_write_high

    # ------------------------------------------------------------------ KERNEL

    def global_alloc(self, emu):
        """GlobalAlloc(flags: Word; size: Longint): THandle. The handle is the selector."""
        sp, ss = emu.reg('sp'), emu.reg('ss')
        size = emu.u32((ss, sp + 4))
        count = max(1, (size + 0xFFFF) // 0x10000)
        first = emu.new_segment()
        for _ in range(count - 1):
            emu.new_segment()
        self.log.append(f'GlobalAlloc {size} -> {first:04x}')
        emu.set(ax=first, dx=0)
        return 6

    def global_free(self, emu):
        emu.set(ax=0)
        return 2

    def global_lock(self, emu):
        sel = emu.u16((emu.reg('ss'), emu.reg('sp') + 4))
        emu.set(ax=0, dx=sel)
        return 2

    def get_free_space(self, emu):
        emu.set(ax=0, dx=0x00F0)                      # about 15 MB
        return 2

    # Borland RTM (Borland Pascal 7 / C++ DPMI16 runtime) memory functions. Their meaning is
    # inferred from how System's heap manager calls them (Tyrian 2000, 1040:0433, 055d, 0381).

    def rtm_alloc(self, emu):
        """RTM.5(var Selector: Word; Owner: Longint; Flags: Word; Size: Longint): 0 = done."""
        sp, ss = emu.reg('sp'), emu.reg('ss')
        result = FarPtr(emu.u16((ss, sp + 6)), emu.u16((ss, sp + 4)))
        size = emu.u32((ss, sp + 14))
        sel = emu.new_segment()
        for _ in range(max(1, (size + 0xFFFF) // 0x10000) - 1):
            emu.new_segment()
        emu.write(result, struct.pack('<H', sel))
        self.log.append(f'RTM alloc {size} -> {sel:04x}')
        emu.set(ax=0)
        return 14

    def rtm_free(self, emu):
        """RTM.6(Selector: Word): free a block."""
        emu.set(ax=0)
        return 2

    def rtm_free_mem(self, emu):
        """RTM.21(Flags: Word; Owner: Longint?): free memory in bytes, DX:AX."""
        emu.set(ax=0, dx=0x00F0)
        return 6

    def init_task(self, emu):
        """InitTask: register-based. AX=1 (success), CX = stack size, DI = instance (DS)."""
        emu.set(ax=1, cx=0x2000, di=emu.reg('ds'), si=0, dx=1, bx=0x81, es=self.psp)
        return 0
