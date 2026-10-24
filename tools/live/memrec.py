#!/usr/bin/env python3
"""Read a running game's memory from outside: snapshots to diff, and recordings of named values.

Usage: memrec.py snap   PROCESS --out DIR
       memrec.py record PROCESS SPEC [--rate HZ] [--seconds S] [--wait ADDR=VALUE[:TYPE]]
                        [--out FILE]
       memrec.py read   PROCESS ADDR [TYPE] [COUNT]

  PROCESS        --pid PID, or --name TEXT (the first process whose command line holds TEXT,
                 found again on every poll, so a game that restarts is followed)
  --proc DIR     where the process folders are (default /proc); tests use a folder of their own
  snap           copy every writable mapping (rw in /proc/PID/maps) to DIR/0xSTART.bin and list
                 them. Take one, change the value in the game, take another, then compare them
                 with tools/triage/memscan.py DUMP:RULE ... --base 0xSTART
  record         sample the values SPEC names, RATE times a second (default 10), for S seconds
                 (default: until the process ends or Ctrl-C), as JSON Lines in FILE (default
                 stdout): {"t": seconds, "n": sample, "NAME": value or [values], ...}
  --wait         start recording only when ADDR holds VALUE (a phase, a level number)
  read           print COUNT values (default 1) of TYPE (default u32) at ADDR

SPEC is a text file, one value a line: `NAME ADDRESS TYPE [COUNT [STRIDE]]`, with # comments.
TYPE is u8 i8 u16 i16 u32 i32 u64 i64 f32 f64; COUNT values STRIDE bytes apart (default the
type's size) make an array, such as eight cars' x positions: `car_x 0x61aad0 i32 8 0x558`.

Why: a port that must behave like the original needs a reference run, and the simplest
recorder reads the game's memory from outside. Nothing is injected, and the game is not
changed or slowed. Values are kept as raw integers, so 16.16 fixed point is not rounded. A
sample is not atomic: values are read one after another while the game runs, so a sample can
straddle frames, and at 10 Hz an outside reader sees about one frame in eight
(docs/knowhow/record-a-run-from-memory.md).

It reads /proc/PID/mem, so it runs where the game runs on Linux: Wine on Linux, or Wine inside
a Linux VM. Reading another process's memory needs root, or the same user with
kernel.yama.ptrace_scope set to 0. It does not run on macOS or Windows, and never writes to the
process. Standard library only.
"""
import json
import os
import re
import struct
import sys
import time

TYPES = {'u8': 'B', 'i8': 'b', 'u16': 'H', 'i16': 'h', 'u32': 'I', 'i32': 'i',
         'u64': 'Q', 'i64': 'q', 'f32': 'f', 'f64': 'd'}


class Gone(Exception):
    """The process ended, or its memory cannot be read."""


def find(proc, name):
    """The lowest PID whose command line holds `name`, or None."""
    pids = sorted(int(p) for p in os.listdir(proc) if p.isdigit())
    for pid in pids:
        try:
            with open(os.path.join(proc, str(pid), 'cmdline'), 'rb') as f:
                cmd = f.read().replace(b'\0', b' ').decode(errors='replace')
        except OSError:
            continue
        if name in cmd and 'memrec' not in cmd:
            return pid
    return None


class Process:
    def __init__(self, proc, pid=None, name=None):
        self.proc, self.pid, self.name, self.f = proc, pid, name, None

    def open(self):
        if self.name:
            pid = find(self.proc, self.name)
            if pid is None:
                raise Gone(f'no process with {self.name!r} in its command line')
            if pid != self.pid:
                self.close()
                self.pid = pid
        if self.f is None:
            try:
                self.f = open(os.path.join(self.proc, str(self.pid), 'mem'), 'rb', buffering=0)
            except OSError as err:
                raise Gone(f'cannot open the memory of {self.pid}: {err}')
        return self

    def close(self):
        if self.f:
            self.f.close()
        self.f = None

    def read(self, addr, n):
        try:
            self.f.seek(addr)
            data = self.f.read(n)
        except (OSError, ValueError) as err:
            raise Gone(f'cannot read {n} bytes at {addr:#x} in {self.pid}: {err}')
        if len(data) != n:
            raise Gone(f'short read at {addr:#x} in {self.pid}')
        return data

    def value(self, addr, kind, count=1, stride=None):
        fmt = '<' + TYPES[kind]
        size = struct.calcsize(fmt)
        stride = stride or size
        if stride == size:
            vals = list(struct.unpack('<' + TYPES[kind] * count, self.read(addr, size * count)))
        else:
            vals = [struct.unpack(fmt, self.read(addr + i * stride, size))[0] for i in range(count)]
        return vals[0] if count == 1 else vals

    def maps(self):
        """[(start, end, perms, name)] of the process's mappings."""
        out = []
        with open(os.path.join(self.proc, str(self.pid), 'maps'), encoding='utf-8', errors='replace') as f:
            for line in f:
                parts = line.split(None, 5)
                lo, hi = (int(x, 16) for x in parts[0].split('-'))
                out.append((lo, hi, parts[1], parts[5].strip() if len(parts) > 5 else ''))
        return out


def parse_spec(path):
    rows = []
    for n, line in enumerate(open(path, encoding='utf-8'), 1):
        line = line.split('#', 1)[0].strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) < 3 or parts[2] not in TYPES:
            raise SystemExit(f'{path}:{n}: want NAME ADDRESS TYPE [COUNT [STRIDE]], TYPE one of {" ".join(TYPES)}')
        count = int(parts[3], 0) if len(parts) > 3 else 1
        stride = int(parts[4], 0) if len(parts) > 4 else None
        rows.append((parts[0], int(parts[1], 0), parts[2], count, stride))
    return rows


def snap(p, out):
    os.makedirs(out, exist_ok=True)
    total = 0
    for lo, hi, perms, name in p.maps():
        if not perms.startswith('rw'):
            continue
        try:
            data = p.read(lo, hi - lo)
        except Gone:
            continue
        path = os.path.join(out, f'{lo:#x}.bin')
        with open(path, 'wb') as f:
            f.write(data)
        total += len(data)
        print(f'{lo:#010x}-{hi:#010x}  {len(data):>10}  {name}  -> {path}')
    print(f'{total} bytes in {out}')
    return 0


def wait(p, cond, rate):
    m = re.fullmatch(r'(0x[0-9a-fA-F]+|\d+)=(-?[0-9a-fA-Fx.]+)(?::(\w+))?', cond)
    if not m:
        raise SystemExit('--wait wants ADDR=VALUE[:TYPE], such as 0x5920a4=2')
    addr, kind = int(m.group(1), 0), m.group(3) or 'u32'
    want = float(m.group(2)) if kind.startswith('f') else int(m.group(2), 0)
    while True:
        try:
            if p.open().value(addr, kind) == want:
                return
        except Gone:
            p.close()
        time.sleep(1 / rate)


def record(p, spec, rate, seconds, cond, out):
    if cond:
        wait(p, cond, rate)
    start, n = time.monotonic(), 0
    try:
        while seconds is None or time.monotonic() - start < seconds:
            due = start + n / rate
            time.sleep(max(0.0, due - time.monotonic()))
            sample = {'t': round(time.monotonic() - start, 4), 'n': n}
            try:
                p.open()
                for name, addr, kind, count, stride in spec:
                    sample[name] = p.value(addr, kind, count, stride)
            except Gone as err:
                print(f'memrec: {err}; stopping after {n} samples', file=sys.stderr)
                break
            out.write(json.dumps(sample) + '\n')
            out.flush()
            n += 1
    except KeyboardInterrupt:
        pass
    print(f'memrec: {n} samples', file=sys.stderr)
    return 0


def main(argv):
    if not argv or argv[0] in ('-h', '--help') or argv[0] not in ('snap', 'record', 'read'):
        print(__doc__.rstrip())
        return 0 if argv and argv[0] in ('-h', '--help') else 2
    cmd, rest = argv[0], argv[1:]
    opts, args, i = {}, [], 0
    while i < len(rest):
        if rest[i].startswith('--'):
            if i + 1 >= len(rest):
                raise SystemExit(f'memrec: {rest[i]} needs a value')
            opts[rest[i]] = rest[i + 1]
            i += 2
        else:
            args.append(rest[i])
            i += 1
    if ('--pid' in opts) == ('--name' in opts):
        raise SystemExit('memrec: give --pid PID or --name TEXT')
    p = Process(opts.get('--proc', '/proc'), int(opts['--pid']) if '--pid' in opts else None, opts.get('--name'))
    try:
        p.open()
    except Gone as err:
        if not (cmd == 'record' and '--wait' in opts):
            raise SystemExit(f'memrec: {err}')
    if cmd == 'snap':
        return snap(p, opts.get('--out') or sys.exit('memrec: snap needs --out DIR'))
    if cmd == 'read':
        kind = args[1] if len(args) > 1 else 'u32'
        print(p.value(int(args[0], 0), kind, int(args[2], 0) if len(args) > 2 else 1))
        return 0
    spec = parse_spec(args[0])
    seconds = float(opts['--seconds']) if '--seconds' in opts else None
    out = open(opts['--out'], 'w', encoding='utf-8') if '--out' in opts else sys.stdout
    return record(p, spec, float(opts.get('--rate', 10)), seconds, opts.get('--wait'), out)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
