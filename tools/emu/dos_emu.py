"""Call functions inside a 16-bit DOS or Windows 3.x program (NE or MZ) under Unicorn.

The 16-bit counterpart of pe_emu.py: run one original routine with inputs you choose and read
its exact result (ground truth for a reimplementation), without DOS, DOSBox or the game.

    from dos_emu import DosEmu, Long, FarPtr
    emu = DosEmu('game.exe')                       # NE: protected mode, Ghidra's selectors
    emu.call('RNG_SEED', Long(1234))               # an exported name, '1000:000f', or (sel, off)
    print(emu.call('1000:0028', 100))              # AX; call_long() returns DX:AX
    buf = emu.bytes_in(b'abc')                     # a FarPtr into emulated memory
    emu.call('CHECKSUM', buf, 3)
    screen = emu.new_segment()                     # a whole 64 KB segment at offset 0

NE programs (Borland Pascal 7 DPMI16, Windows 3.x) run in 16-bit protected mode. Segment N gets
selector 0x1000 + 8 * (N - 1), the same numbers Ghidra's NE loader uses, so an address from the
Ghidra export (`1000:0028`, `FUN_1000_0028`) is the address here. Relocations are applied:
selectors, far pointers and offsets to the program's own segments; imports (by ordinal or name)
become stubs. DS and SS are the auto data segment, as in a Pascal or Windows program.

MZ programs (real mode) load at segment 0x1000, as in Ghidra, with the PSP at 0x0FF0 and DS = ES
= PSP as DOS leaves them; SS:SP come from the header.

Arguments: an int is one word; `Long(v)` is a 32-bit value (high word pushed first); a `FarPtr`
or a (segment, offset) tuple is a far pointer (segment pushed first). The default convention is
`pascal`: arguments pushed left to right and removed by the callee, as Borland Pascal and the
Windows API do. Pass conv='cdecl' for C code (pushed right to left). Routines are called far
(a near routine returns with a plain `ret`: call it through a far caller).

Imports and interrupts: `stubs={'KERNEL.3': fn}` (module.ordinal or module.NAME) and
`interrupts={0x21: fn}` give Python handlers; each gets the emulator, reads and sets registers
with `emu.reg('ax')` / `emu.set(ax=...)`, and an import handler returns the number of argument
bytes to remove (a Pascal callee's `retf N`). An import or interrupt with no handler stops the
run with its name and the caller's address, so you know exactly what to stub next.

What it does not do: run DOS, load overlays or DLLs, emulate DPMI services, timers, VGA or sound
hardware, or the x87 (Borland's floating-point emulator fixups are left as they are). Use DOSBox-X
(scripts/dosbox-run.sh) when you need the whole program running. Needs `unicorn`.
"""
import struct

from unicorn import (
    UC_ARCH_X86, UC_HOOK_CODE, UC_HOOK_INTR, UC_HOOK_MEM_INVALID, UC_MODE_16, UC_MODE_32, Uc, UcError,
)
from unicorn.x86_const import (
    UC_X86_REG_AX, UC_X86_REG_BP, UC_X86_REG_BX, UC_X86_REG_CS, UC_X86_REG_CX, UC_X86_REG_DI,
    UC_X86_REG_DS, UC_X86_REG_DX, UC_X86_REG_EIP, UC_X86_REG_ES, UC_X86_REG_FLAGS, UC_X86_REG_GDTR,
    UC_X86_REG_SI, UC_X86_REG_SP, UC_X86_REG_SS,
)

REGS = {'ax': UC_X86_REG_AX, 'bx': UC_X86_REG_BX, 'cx': UC_X86_REG_CX, 'dx': UC_X86_REG_DX,
        'si': UC_X86_REG_SI, 'di': UC_X86_REG_DI, 'bp': UC_X86_REG_BP, 'sp': UC_X86_REG_SP,
        'ds': UC_X86_REG_DS, 'es': UC_X86_REG_ES, 'ss': UC_X86_REG_SS, 'cs': UC_X86_REG_CS,
        'ip': UC_X86_REG_EIP, 'flags': UC_X86_REG_FLAGS}
FIRST_SELECTOR = 0x1000
GDT_BASE = 0x10000
TRAP = 0xFFF0                      # the return trap's offset in the stub segment
MZ_LOAD = 0x1000                   # where Ghidra and this emulator load an MZ image
MZ_STUBS, MZ_HEAP = 0xF000, 0x8000


class Long(int):
    """A 32-bit argument: pushed as two words, high word first."""


class FarPtr(tuple):
    """A far pointer (segment or selector, offset)."""

    def __new__(cls, seg, off):
        return super().__new__(cls, (seg, off))

    seg = property(lambda self: self[0])
    off = property(lambda self: self[1])

    def __add__(self, n):
        return FarPtr(self[0], (self[1] + n) & 0xFFFF)

    def __repr__(self):
        return f'{self[0]:04x}:{self[1]:04x}'


def descriptor(base, limit, access):
    """An 8-byte GDT entry for a 16-bit segment (byte granular, D = 0)."""
    return struct.pack('<Q', (limit & 0xFFFF) | ((base & 0xFFFFFF) << 16) | (access << 40)
                       | (((limit >> 16) & 0xF) << 48) | ((base >> 24) << 56))


class DosEmu:
    def __init__(self, path, stubs=None, interrupts=None, max_steps=50_000_000, verbose=False):
        self.data = open(path, 'rb').read()
        self.path = path
        self.stubs = dict(stubs or {})
        self.interrupts = dict(interrupts or {})
        self.max_steps = max_steps
        self.verbose = verbose
        self.calls = []          # (name, caller) of every import call, for verbose runs
        self.exports = {}        # NAME (upper case) or '#ordinal' -> FarPtr
        self.imports = {}        # stub offset -> 'MODULE.ordinal' or 'MODULE.NAME'
        self.error = None
        self.pending = None
        self.traces = []
        lfanew, = struct.unpack_from('<I', self.data, 0x3C) if len(self.data) > 0x40 else (0,)
        reloc_at, = struct.unpack_from('<H', self.data, 0x18)
        if reloc_at >= 0x40 and lfanew + 2 <= len(self.data) and self.data[lfanew:lfanew + 2] == b'NE':
            self.kind = 'NE'
            self._load_ne(lfanew)
        elif self.data[:2] in (b'MZ', b'ZM'):
            self.kind = 'MZ'
            self._load_mz()
        else:
            raise ValueError(f'{path}: not an MZ or NE program')
        self.uc.hook_add(UC_HOOK_CODE, self._on_stub, begin=self.linear(self.stub_seg, 0),
                         end=self.linear(self.stub_seg, 0xFFFF))
        self.uc.hook_add(UC_HOOK_INTR, self._on_interrupt)
        self.uc.hook_add(UC_HOOK_MEM_INVALID, self._on_bad_mem)

    # ------------------------------------------------------------------ NE

    def selector(self, n):
        """The selector of NE segment number `n` (1-based), as Ghidra numbers it."""
        return FIRST_SELECTOR + 8 * (n - 1)

    def linear(self, seg, off=0):
        if self.kind == 'NE':
            return ((seg >> 3) << 16) + off
        return (seg << 4) + off

    def _new_selector(self, access, size=0x10000):
        sel = self.next_sel
        self.next_sel += 8
        base = self.linear(sel)
        self.uc.mem_map(base, 0x10000)
        self.uc.mem_write(GDT_BASE + sel, descriptor(base, size - 1, access))
        return sel

    def _load_ne(self, ne):
        d = self.data
        self.uc = Uc(UC_ARCH_X86, UC_MODE_32)
        self.uc.mem_map(GDT_BASE, 0x10000)
        self.uc.reg_write(UC_X86_REG_GDTR, (0, GDT_BASE, 0xFFFF, 0))
        h = lambda fmt, off: struct.unpack_from(fmt, d, ne + off)
        self.autodata, = h('<H', 0x0E)
        heap, stack = h('<HH', 0x10)
        nseg, nmod = h('<HH', 0x1C)
        segtab, _, restab, modtab, imptab = h('<HHHHH', 0x22)
        nonres_file, = h('<I', 0x2C)
        nonres_len, = h('<H', 0x20)
        entry_off, entry_len = h('<HH', 0x04)
        align, = h('<H', 0x32)
        self.next_sel = FIRST_SELECTOR
        self.segments = []   # (file offset, file length, flags, min alloc)
        for i in range(nseg):
            off, length, flags, minalloc = h('<HHHH', segtab + 8 * i)
            self.segments.append((off << align, length or (0x10000 if off else 0), flags, minalloc or 0x10000))
            sel = self._new_selector(0x92 if flags & 1 else 0x9A)
            if off:
                self.uc.mem_write(self.linear(sel), d[(off << align):(off << align) + (length or 0x10000)])
        self.stub_seg = self._new_selector(0x9A)
        self.uc.mem_write(self.linear(self.stub_seg, TRAP), b'\xf4')
        self.next_stub = 0
        self.heap_seg, self.heap_top = self._new_selector(0x92), 0
        self.modules = []
        for i in range(nmod):
            o, = h('<H', modtab + 2 * i)
            self.modules.append(self._pstr(ne + imptab + o))
        # Entry table: ordinal -> (segment number, offset).
        entries, pos, ordinal = {}, ne + entry_off, 1
        while pos < ne + entry_off + entry_len and d[pos]:
            count, kind = d[pos], d[pos + 1]
            pos += 2
            for _ in range(count):
                if kind == 0:
                    pass
                elif kind == 0xFF:      # movable: flags, int 3Fh, segment, offset
                    entries[ordinal] = (d[pos + 3], struct.unpack_from('<H', d, pos + 4)[0])
                    pos += 6
                else:                   # fixed: flags, offset
                    entries[ordinal] = (kind, struct.unpack_from('<H', d, pos + 1)[0])
                    pos += 3
                ordinal += 1
        self.entries = entries
        for o, (seg, off) in entries.items():
            self.exports[f'#{o}'] = FarPtr(self.selector(seg), off)
        for table, end in ((ne + restab, ne + modtab), (nonres_file, nonres_file + nonres_len)):
            pos, first = table, True
            while pos < end and d[pos]:
                name = d[pos + 1:pos + 1 + d[pos]].decode('latin-1')
                o, = struct.unpack_from('<H', d, pos + 1 + d[pos])
                pos += 1 + d[pos] + 2
                if not first and o in entries:
                    self.exports[name.upper()] = self.exports[f'#{o}']
                first = False
        for i, (off, length, flags, _) in enumerate(self.segments, 1):
            if flags & 0x0100 and off:
                self._relocate_ne(i, off + length, ne, imptab)
        ds = self.selector(self.autodata) if self.autodata else self.heap_seg
        size = min(0x10000, self.segments[self.autodata - 1][3] + heap + stack) if self.autodata else 0x10000
        self.regs = {'ds': ds, 'es': ds, 'ss': ds, 'sp': (size - 2) & 0xFFFE}
        ip, cs, sp, ss = h('<HHHH', 0x14)
        self.entry = FarPtr(self.selector(cs), ip) if cs else None

    def _relocate_ne(self, segno, at, ne, imptab):
        d = self.data
        count, = struct.unpack_from('<H', d, at)
        base = self.linear(self.selector(segno))
        for i in range(count):
            src, flags, off = struct.unpack_from('<BBH', d, at + 2 + 8 * i)
            kind = flags & 3
            if kind == 0:     # internal reference
                seg, _, target = struct.unpack_from('<BBH', d, at + 2 + 8 * i + 4)
                if seg == 0xFF:
                    seg, target = self.entries[target]
                sel, target = self.selector(seg), target
            elif kind in (1, 2):   # import by ordinal or by name
                mod, what = struct.unpack_from('<HH', d, at + 2 + 8 * i + 4)
                module = self.modules[mod - 1]
                key = f'{module}.{what}' if kind == 1 else f'{module}.{self._pstr(ne + imptab + what)}'
                sel, target = self.stub_seg, self._stub_for(key)
            else:              # OS fixup (floating-point emulator): left as it is
                continue
            while True:
                old, = struct.unpack('<H', self.uc.mem_read(base + off, 2))
                if src == 2:
                    patch = struct.pack('<H', sel)
                elif src == 3:
                    patch = struct.pack('<HH', target, sel)
                elif src == 5:
                    patch = struct.pack('<H', target)
                else:
                    raise ValueError(f'segment {segno}: relocation type {src} is not supported')
                if flags & 4:   # additive: add to what is there, no chain
                    if src == 5:
                        patch = struct.pack('<H', (old + target) & 0xFFFF)
                    elif src == 3:
                        patch = struct.pack('<HH', (old + target) & 0xFFFF, sel)
                    self.uc.mem_write(base + off, patch)
                    break
                self.uc.mem_write(base + off, patch)
                if old == 0xFFFF or old == off:
                    break
                off = old

    def _stub_for(self, key):
        for off, k in self.imports.items():
            if k == key:
                return off
        off = self.next_stub
        self.next_stub += 4
        self.uc.mem_write(self.linear(self.stub_seg, off), b'\x90\x90\x90\x90')
        self.imports[off] = key
        return off

    def _pstr(self, pos):
        return self.data[pos + 1:pos + 1 + self.data[pos]].decode('latin-1')

    # ------------------------------------------------------------------ MZ

    def _load_mz(self):
        d = self.data
        last, pages, nrel, hdr, minalloc, maxalloc, ss, sp, _, ip, cs, relat = struct.unpack_from('<12H', d, 2)
        size = (pages - 1) * 512 + (last or 512) if pages else len(d)
        image = d[hdr * 16:size]
        self.uc = Uc(UC_ARCH_X86, UC_MODE_16)
        self.uc.mem_map(0, 0x110000)
        self.uc.mem_write(MZ_LOAD * 16, image)
        for i in range(nrel):
            off, seg = struct.unpack_from('<HH', d, relat + 4 * i)
            at = (MZ_LOAD + seg) * 16 + off
            v, = struct.unpack('<H', self.uc.mem_read(at, 2))
            self.uc.mem_write(at, struct.pack('<H', (v + MZ_LOAD) & 0xFFFF))
        psp = MZ_LOAD - 0x10
        self.uc.mem_write(psp * 16, b'\xcd\x20')
        self.stub_seg = MZ_STUBS
        self.uc.mem_write(self.linear(MZ_STUBS, TRAP), b'\xf4')
        self.next_stub = 0
        self.heap_seg, self.heap_top = MZ_HEAP, 0
        self.regs = {'ds': psp, 'es': psp, 'ss': MZ_LOAD + ss, 'sp': sp}
        self.entry = FarPtr(MZ_LOAD + cs, ip)

    # ------------------------------------------------------------------ memory

    def addr(self, where):
        """A FarPtr from an export name, '1000:0028', 'FUN_1000_0028', a tuple or a FarPtr."""
        if isinstance(where, tuple):
            return FarPtr(*where)
        if where.upper() in self.exports:
            return self.exports[where.upper()]
        text = where.upper().removeprefix('FUN_').replace('_', ':')
        if ':' in text:
            seg, off = text.split(':')
            return FarPtr(int(seg, 16), int(off, 16))
        raise KeyError(f'{where}: not an export of {self.path} and not a seg:off address')

    def read(self, ptr, n):
        return bytes(self.uc.mem_read(self.linear(*ptr), n))

    def write(self, ptr, data):
        self.uc.mem_write(self.linear(*ptr), bytes(data))

    def u16(self, ptr):
        return struct.unpack('<H', self.read(ptr, 2))[0]

    def u32(self, ptr):
        return struct.unpack('<I', self.read(ptr, 4))[0]

    def malloc(self, n):
        """Room for `n` bytes (zeroed); returns a FarPtr. Each block stays inside one segment."""
        if n > 0x10000:
            raise ValueError('a 16-bit block is at most 64 KB')
        if self.heap_top + n > 0x10000:
            self.heap_seg = self._new_selector(0x92) if self.kind == 'NE' else self.heap_seg + 0x1000
            self.heap_top = 0
        ptr = FarPtr(self.heap_seg, self.heap_top)
        self.heap_top = (self.heap_top + n + 15) & ~15
        self.write(ptr, b'\0' * n)
        return ptr

    def new_segment(self, data=b''):
        """A fresh 64 KB segment (NE: a new data selector; MZ: 4096 paragraphs from the heap)
        holding `data` at offset 0, as a protected-mode GetMem of a large block returns it.
        Returns the selector or segment; use it as a FarPtr (seg, 0) or store it in a global."""
        if len(data) > 0x10000:
            raise ValueError('a segment holds at most 64 KB')
        if self.kind == 'NE':
            seg = self._new_selector(0x92)
        else:
            self.heap_seg += 0x1000
            seg = self.heap_seg
            self.heap_top = 0x10000   # the next malloc starts a new heap segment
        self.write(FarPtr(seg, 0), b'\0' * 0x10000 if self.kind == 'NE' else b'\0' * 0xFFFF)
        if data:
            self.write(FarPtr(seg, 0), data)
        return seg

    def bytes_in(self, data):
        ptr = self.malloc(len(data))
        self.write(ptr, data)
        return ptr

    def reg(self, name):
        return self.uc.reg_read(REGS[name])

    def set(self, **regs):
        for name, value in regs.items():
            self.uc.reg_write(REGS[name], value)

    def push(self, word):
        sp = (self.reg('sp') - 2) & 0xFFFF
        self.set(sp=sp)
        self.write((self.reg('ss'), sp), struct.pack('<H', word & 0xFFFF))

    def pop(self):
        sp = self.reg('sp')
        v = self.u16((self.reg('ss'), sp))
        self.set(sp=(sp + 2) & 0xFFFF)
        return v

    # ------------------------------------------------------------------ calls

    def call(self, target, *args, conv='pascal', near=False, regs=None):
        """Run the routine at `target` with `args` and return AX. `regs` sets registers first
        (for example ds=... for code that expects another data segment)."""
        ptr = self.addr(target)
        state = dict(self.regs)
        state.update(regs or {})
        for name in ('ss', 'sp', 'ds', 'es'):
            self.set(**{name: state[name]})
        for a in (args if conv == 'pascal' else reversed(args)):
            if isinstance(a, tuple):
                self.push(a[0]); self.push(a[1])
            elif isinstance(a, Long):
                self.push(a >> 16); self.push(a)
            else:
                self.push(a)
        if near:
            raise NotImplementedError('near routines: call the far routine that calls it, or '
                                      'write a far wrapper (push cs; call near; retf) into free memory')
        self.push(self.stub_seg)
        self.push(TRAP)
        self._run(ptr)
        return self.reg('ax')

    def call_long(self, target, *args, **kw):
        """Like call, for a routine that returns a 32-bit value in DX:AX."""
        ax = self.call(target, *args, **kw)
        return (self.reg('dx') << 16) | ax

    def _run(self, ptr):
        self.error = None
        cs, ip = ptr
        while True:
            self.pending = None
            self.set(cs=cs)
            try:
                self.uc.emu_start(ip, 0, count=self.max_steps)
            except UcError as e:
                raise RuntimeError(self.error or f'{e} at {self.reg("cs"):04x}:{self.reg("ip"):04x}') from None
            if self.error:
                raise RuntimeError(self.error)
            if self.pending is None:
                raise RuntimeError(f'stopped after {self.max_steps} instructions at '
                                   f'{self.reg("cs"):04x}:{self.reg("ip"):04x} (an endless loop, '
                                   f'or raise max_steps)')
            if self.pending == 'return':
                return
            key, handler = self.pending
            ret_ip, ret_cs = self.pop(), self.pop()
            self.push(ret_cs); self.push(ret_ip)
            self.calls.append((key, f'{ret_cs:04x}:{ret_ip:04x}'))
            if self.verbose:
                print(f'  import {key} from {ret_cs:04x}:{ret_ip:04x}')
            pop = handler(self) or 0
            ip, cs = self.pop(), self.pop()
            self.set(sp=(self.reg('sp') + pop) & 0xFFFF)

    def _on_stub(self, uc, address, size, _):
        off = address - self.linear(self.stub_seg)
        if off == TRAP:
            self.pending = 'return'
            uc.emu_stop()
            return
        key = self.imports.get(off)
        handler = self.stubs.get(key)
        if handler is None and key:
            mod, _, what = key.partition('.')
            handler = self.stubs.get(f'{mod}.{what.upper()}')
        if handler is None:
            sp = self.reg('sp')
            ret = FarPtr(self.u16((self.reg('ss'), sp + 2)), self.u16((self.reg('ss'), sp)))
            self.error = f'unhandled import {key} called from {ret!r}: add stubs={{"{key}": handler}}'
            uc.emu_stop()
            return
        self.pending = (key, handler)
        uc.emu_stop()

    def _on_interrupt(self, uc, number, _):
        handler = self.interrupts.get(number)
        where = f'{self.reg("cs"):04x}:{self.reg("ip") - 2:04x}'
        if self.verbose:
            print(f'  int {number:02x}h ah={self.reg("ax") >> 8:02x}h at {where}')
        if handler is None:
            self.error = (f'unhandled interrupt {number:02X}h (AH={self.reg("ax") >> 8:02X}h) at {where}: '
                          f'add interrupts={{0x{number:02x}: handler}}')
            uc.emu_stop()
            return
        handler(self)

    def _on_bad_mem(self, uc, access, address, size, value, _):
        self.error = (f'bad memory access at linear {address:#x} ({size} bytes) from '
                      f'{self.reg("cs"):04x}:{self.reg("ip"):04x}')
        return False

    # ------------------------------------------------------------------ call traces

    def trace(self, functions, args=0):
        """Record calls to chosen routines, in order, with their call sites (the random draws).

        `functions` maps an address (as for call) to a label. Only outermost calls are
        recorded, as tools/emu/call_trace.py does for x64. Pass `args=N` to record the first N
        word arguments (Pascal order: the first argument is deepest on the stack). Read
        `emu.traces`, or write them with `write_trace(path)` for tools/verify/seqdiff.py.
        """
        depth = {'until': 0}
        for where, label in functions.items():
            ptr = self.addr(where)
            def on_entry(uc, address, size, _, label=label):
                sp = self.reg('sp')
                if depth['until'] and sp < depth['until']:
                    return
                ss = self.reg('ss')
                ret = FarPtr(self.u16((ss, sp + 2)), self.u16((ss, sp)))
                text = label
                if args:
                    words = [self.u16((ss, sp + 4 + 2 * i)) for i in range(args)]
                    text += '(' + ', '.join(str(w) for w in reversed(words)) + ')'
                self.traces.append(f'{text}\t{ret!r}')
                depth['until'] = sp
                holder = []
                def on_return(uc2, a2, s2, _):
                    depth['until'] = 0
                    uc2.hook_del(holder[0])
                at = self.linear(*ret)
                holder.append(uc.hook_add(UC_HOOK_CODE, on_return, begin=at, end=at))
            at = self.linear(*ptr)
            self.uc.hook_add(UC_HOOK_CODE, on_entry, begin=at, end=at)

    def write_trace(self, path):
        with open(path, 'w') as f:
            f.write('\n'.join(self.traces) + '\n')


if __name__ == '__main__':
    print(__doc__)
