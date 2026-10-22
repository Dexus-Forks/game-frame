"""Record which instructions read and write chosen memory while an emulated function runs.

This is the first step of taint analysis, done on the emulator instead of a live process with
Pin or DynamoRIO: mark some bytes (a buffer read from the save file, a global, a table), call
the function, and list each instruction that touched them.

    from mem_trace import MemTrace
    trace = MemTrace(emu, [(buf, len(data), 'save')], follow_copies=True)
    emu.call(load, buf, len(data), out)
    trace.write('derived/load.memtrace')   # "label<TAB>r|w<TAB>offset<TAB>size<TAB>module+offset"
    trace.readers()                           # {module+offset: count}, the code that read the bytes

With `follow_copies`, a write made by the same instruction that just read marked bytes (a
`rep movs`, an unrolled copy) marks the destination too, with the same label. A value that
passes through a register between two instructions is not followed: that needs the
instruction's meaning, which Pin-based tools such as libdft model. Start from the readers this
tool lists, and read their code.
"""
from unicorn import UC_HOOK_MEM_READ, UC_HOOK_MEM_WRITE
from unicorn.x86_const import UC_X86_REG_RIP


class MemTrace:
    def __init__(self, emu, regions, follow_copies=False):
        """regions: [(address, size, label)]."""
        self.emu = emu
        self.regions = [(a, a + n, label) for a, n, label in regions]
        self.follow = follow_copies
        self.log = []
        self._last_read = None  # (pc, label) of the latest read of marked bytes
        emu.uc.hook_add(UC_HOOK_MEM_READ, self._on_access, 'r')
        emu.uc.hook_add(UC_HOOK_MEM_WRITE, self._on_access, 'w')

    def _where(self, addr):
        for m in self.emu.modules.values():
            if m.contains(addr):
                return f'{m.name}+{addr - m.base:#x}'
        return f'{addr:#x}'

    def _region(self, addr):
        for start, end, label in self.regions:
            if start <= addr < end:
                return start, label
        return None

    def _on_access(self, uc, access, address, size, value, kind):
        pc = uc.reg_read(UC_X86_REG_RIP)
        hit = self._region(address)
        if hit:
            start, label = hit
            self.log.append((label, kind, address - start, size, self._where(pc)))
            if kind == 'r':
                self._last_read = (pc, label)
        elif kind == 'w' and self.follow and self._last_read and self._last_read[0] == pc:
            label = self._last_read[1]
            self.regions.append((address, address + size, label))
            self.log.append((label, 'copy', 0, size, self._where(pc)))

    def readers(self):
        out = {}
        for label, kind, _, _, where in self.log:
            if kind == 'r':
                out[where] = out.get(where, 0) + 1
        return out

    def write(self, path):
        with open(path, 'w') as f:
            for label, kind, off, size, where in self.log:
                f.write(f'{label}\t{kind}\t{off:#x}\t{size}\t{where}\n')
