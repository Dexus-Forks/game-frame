"""Call routines inside a 32-bit Windows (x86) game under Unicorn, without running the game.

The x64 emulator `pe_emu.py` covers 64-bit games. Games of the 1990s are 32-bit, and much of
their fastest code is hand-written assembly that takes its inputs in registers, not by a C
calling convention: a span filler with the destination in EDI, a mixer with the channel in
ESI. This module runs such routines exactly as they are, MMX and x87 included.

    from pe32_emu import Emu32
    emu = Emu32('GAME.EXE')                        # mapped at its own image base
    spans = emu.alloc(0x1000)
    emu.write(spans, data)
    out = emu.call(0x4dde10, esi=spans, ebx=mask, edx=rows)       # registers in
    out = emu.call(0x4d1460, args=[channel, buffer, 64])          # or cdecl/stdcall args
    print(out['eax'], emu.read(mask, 0x50))

    emu = Emu32.raw(code_bytes, base=0x401000)     # or bare machine code, for tests

What it does:

- maps the image at its preferred base, as the loader would; Watcom images whose section
  headers give a virtual size of 0 are mapped from their raw sizes (pefile does this);
- points every import at a stub that stops the run with the import's name, so a routine that
  calls Windows fails loudly instead of running off into nowhere (`stubs=` gives a Python
  handler for an import: it receives the emulator and returns EAX; the stub then returns with
  the callee popping `pop_bytes` of arguments, as stdcall does);
- gives the code a stack and a bump allocator (`alloc`); `reset()` puts the image's data back
  and clears the allocator, so each trial starts from the same state;
- `call(addr, args=[...], **registers)` pushes `args` right to left (cdecl and stdcall), then a
  return address that stops the run, sets the named registers and runs until the routine
  returns. It returns every general register afterwards.

What it does not do: run DllMain or the C runtime start-up, create threads, or emulate
Windows. Routines that need the game's globals set up need them written first (`write`), from
the original's own initialisation or from a captured memory dump.

Usage as a program (prints the registers after the call):

    pe32_emu.py BINARY ADDRESS [REG=VALUE ...] [--args V,V,...] [--dump ADDRESS:LENGTH]

Needs `unicorn` and `pefile` (scripts/setup.sh puts them in .venv).
"""
import struct
import sys

import pefile
from unicorn import UC_ARCH_X86, UC_HOOK_CODE, UC_HOOK_MEM_INVALID, UC_MODE_32, Uc, UcError
from unicorn.x86_const import (
    UC_X86_REG_EAX, UC_X86_REG_EBP, UC_X86_REG_EBX, UC_X86_REG_ECX, UC_X86_REG_EDI,
    UC_X86_REG_EDX, UC_X86_REG_EFLAGS, UC_X86_REG_EIP, UC_X86_REG_ESI, UC_X86_REG_ESP,
)

REGS = {'eax': UC_X86_REG_EAX, 'ebx': UC_X86_REG_EBX, 'ecx': UC_X86_REG_ECX,
        'edx': UC_X86_REG_EDX, 'esi': UC_X86_REG_ESI, 'edi': UC_X86_REG_EDI,
        'ebp': UC_X86_REG_EBP, 'esp': UC_X86_REG_ESP, 'eflags': UC_X86_REG_EFLAGS}
PAGE = 0x1000
HEAP_BASE, HEAP_SIZE = 0x60000000, 0x04000000
STACK_TOP, STACK_SIZE = 0x6f000000, 0x00100000
STUB_BASE, STUB_SIZE = 0x7f000000, 0x00010000
RETURN_TRAP = STUB_BASE + STUB_SIZE - 0x10


def align(v, a=PAGE):
    return (v + a - 1) & ~(a - 1)


class EmuError(RuntimeError):
    pass


class Emu32:
    def __init__(self, path=None, stubs=None, *, image=None, base=None):
        """Load a 32-bit PE (`path`), or bare code (`image` bytes at `base`; see `raw`)."""
        self.uc = Uc(UC_ARCH_X86, UC_MODE_32)
        self.stubs = {}        # stub address -> (import name, handler, pop_bytes)
        self.handlers = dict(stubs or {})
        if path is not None:
            pe = pefile.PE(path)
            if pe.FILE_HEADER.Machine != 0x14c:
                raise ValueError(f'{path}: not a 32-bit x86 image (machine {pe.FILE_HEADER.Machine:#x})')
            self.base = pe.OPTIONAL_HEADER.ImageBase
            image = pe.get_memory_mapped_image()
            self.size = align(max(pe.OPTIONAL_HEADER.SizeOfImage, len(image)))
            image = image[:self.size]
        else:
            self.base, self.size, pe = base, align(len(image)), None
        self.uc.mem_map(self.base, self.size)
        self.uc.mem_write(self.base, bytes(image))
        self.uc.mem_map(HEAP_BASE, HEAP_SIZE)
        self.uc.mem_map(STACK_TOP - STACK_SIZE, STACK_SIZE)
        self.uc.mem_map(STUB_BASE, STUB_SIZE)
        self.uc.mem_write(STUB_BASE, b'\xf4' * STUB_SIZE)  # hlt everywhere: every stub stops
        if pe is not None:
            self._link(pe)
        self.image = bytes(self.uc.mem_read(self.base, self.size))
        self.heap_top = HEAP_BASE
        self.uc.hook_add(UC_HOOK_CODE, self._on_stub, begin=STUB_BASE, end=STUB_BASE + STUB_SIZE - 1)
        self.uc.hook_add(UC_HOOK_MEM_INVALID, self._on_bad_mem)
        self._stopped_in = None

    @classmethod
    def raw(cls, code, base=0x401000, stubs=None):
        """Bare machine code at `base`, with the same stack, heap and calls as an image."""
        return cls(None, stubs, image=code, base=base)

    # ------------------------------------------------------------------ loading

    def _link(self, pe):
        nxt = STUB_BASE
        for d in getattr(pe, 'DIRECTORY_ENTRY_IMPORT', []):
            for imp in d.imports:
                name = imp.name.decode() if imp.name else f'#{imp.ordinal}'
                full = f'{d.dll.decode()}!{name}'
                handler = self.handlers.get(name) or self.handlers.get(full)
                pop = 0
                if isinstance(handler, tuple):
                    handler, pop = handler
                self.stubs[nxt] = (full, handler, pop)
                self.uc.mem_write(imp.address, struct.pack('<I', nxt))
                nxt += 0x10

    # ------------------------------------------------------------------ memory

    def reset(self):
        """Put the image back as loaded and empty the heap."""
        self.uc.mem_write(self.base, self.image)
        self.uc.mem_write(HEAP_BASE, bytes(self.heap_top - HEAP_BASE))
        self.heap_top = HEAP_BASE

    def alloc(self, n, fill=0):
        a = self.heap_top
        self.heap_top = align(a + n, 16)
        if self.heap_top > HEAP_BASE + HEAP_SIZE:
            raise EmuError('the emulated heap is full')
        self.uc.mem_write(a, bytes([fill]) * n)
        return a

    def write(self, addr, data):
        self.uc.mem_write(addr, bytes(data))

    def read(self, addr, n):
        return bytes(self.uc.mem_read(addr, n))

    def u32(self, addr):
        return struct.unpack('<I', self.read(addr, 4))[0]

    # ------------------------------------------------------------------ calls

    def call(self, addr, args=(), count=50_000_000, **regs):
        """Run the routine at `addr` until it returns; return the registers afterwards."""
        sp = STACK_TOP - 0x100 - 4 * len(args)
        self.write(sp + 4, struct.pack(f'<{len(args)}I', *[a & 0xffffffff for a in args]))
        self.write(sp, struct.pack('<I', RETURN_TRAP))
        self.uc.reg_write(UC_X86_REG_ESP, sp)
        for k, v in regs.items():
            if k not in REGS:
                raise ValueError(f'unknown register {k}')
            self.uc.reg_write(REGS[k], v & 0xffffffff)
        self._stopped_in = None
        start = addr
        while True:
            try:
                self.uc.emu_start(start, RETURN_TRAP, count=count)
            except UcError as e:
                eip = self.uc.reg_read(UC_X86_REG_EIP)
                raise EmuError(f'{e} at {eip:#x}') from None
            if self._stopped_in is None:
                break
            name, handler, pop = self.stubs[self._stopped_in]
            if handler is None:
                raise EmuError(f'the routine called {name}, which has no handler (pass stubs={{"{name.split("!")[1]}": fn}})')
            self._stopped_in = None
            self.uc.reg_write(UC_X86_REG_EAX, handler(self) & 0xffffffff)
            esp = self.uc.reg_read(UC_X86_REG_ESP)
            ret = self.u32(esp)
            self.uc.reg_write(UC_X86_REG_ESP, esp + 4 + pop)
            start = ret
            if ret == RETURN_TRAP:
                break
        return {k: self.uc.reg_read(r) for k, r in REGS.items()}

    def arg(self, i):
        """Inside a stub handler: the i-th stack argument of the import being called."""
        return self.u32(self.uc.reg_read(UC_X86_REG_ESP) + 4 + 4 * i)

    def _on_stub(self, uc, address, size, user):
        if address in self.stubs:
            self._stopped_in = address
            uc.emu_stop()

    def _on_bad_mem(self, uc, access, address, size, value, user):
        eip = uc.reg_read(UC_X86_REG_EIP)
        print(f'pe32_emu: bad memory access at {address:#x} (size {size}) from {eip:#x}', file=sys.stderr)
        return False


def main(argv):
    if len(argv) < 2 or argv[0] in ('-h', '--help'):
        print(__doc__)
        return 0 if argv[:1] in (['-h'], ['--help']) else 1
    emu = Emu32(argv[0])
    addr = int(argv[1], 16)
    regs, args, dumps = {}, [], []
    rest = argv[2:]
    while rest:
        a = rest.pop(0)
        if a == '--args':
            args = [int(v, 0) for v in rest.pop(0).split(',')]
        elif a == '--dump':
            where, n = rest.pop(0).split(':')
            dumps.append((int(where, 16), int(n, 0)))
        elif '=' in a:
            k, v = a.split('=', 1)
            regs[k.lower()] = int(v, 0)
        else:
            raise SystemExit(f'pe32_emu: unexpected argument {a}')
    out = emu.call(addr, args=args, **regs)
    print(' '.join(f'{k}={v:#010x}' for k, v in out.items()))
    for where, n in dumps:
        print(f'{where:#x}: {emu.read(where, n).hex()}')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
