#!/usr/bin/env python3
"""Attach to a program running under Wine or CrossOver through winedbg's GDB server, read its
registers and memory, wait at breakpoints, and always detach so the program runs on.

Usage:
  winegdb.py procs                          WHERE
  winegdb.py regs   TARGET [--thread ID] [--all-threads]
  winegdb.py read   TARGET ADDR LEN [--out FILE]
  winegdb.py break  TARGET ADDR [ADDR ...] [--hits N] [--timeout S] [--dump WHAT:LEN ...]
  winegdb.py watch  TARGET ADDR [--len N] [--hits N] [--timeout S] [--dump WHAT:LEN ...]

  WHERE     --bottle NAME (a CrossOver bottle) or --prefix DIR (a WINEPREFIX), and
            --wine PATH when wine is not CrossOver's or not on PATH
  TARGET    --pid PID or --name EXE (a Windows process id in hex as `procs` prints it, or the
            executable's name), with WHERE; or --connect HOST:PORT for a GDB server that is
            already listening (winedbg --gdb, QEMU -gdb, VMware's debugStub, 86Box)
  ADDR      a number: 0x140118340. The game's own addresses from Ghidra work as they are when
            Wine loads it at its preferred base; `read TARGET 0x140000000 2` shows "MZ" then
  break     waits at code addresses. winedbg refuses software breakpoints (Z0), so the tool
            writes an int3 byte itself, moves rip back onto the address at a hit, and puts the
            original byte back before it detaches
  watch     waits for a write to ADDR (--len 1, 2, 4 or 8 bytes, default 4), with the CPU's
            debug registers, in every thread that exists at the attach. The registers are
            those after the writing instruction. Under Rosetta it never fires: see Pitfalls
  --hits    stop after N hits (default 1); each hit prints the registers and the dumps
  --timeout seconds to wait for each hit (default 30); then the program is stopped, the
            breakpoints are removed and the tool detaches
  --dump    memory to print at each hit: ADDR:LEN, or REG:LEN for the memory a register points
            at, or REG+OFF:LEN (rcx+0x40:16)
  --json    one JSON object per line instead of text
  --all-regs  text output with the x87, SSE and control registers too (JSON always has them)

Output: text, or JSON lines with --json. `regs` prints the stopped thread's registers. `read`
prints a hex dump or writes the bytes to --out.

How it works: `wine winedbg --gdb --no-start PID` attaches winedbg to the process and starts a
GDB server on a free port; this tool reads the port from winedbg's output and talks to it with
gdbremote.py. Attaching stops the program until the tool detaches.

Pitfalls, from Banished under CrossOver 26.3 on Apple silicon (docs/knowhow/winedbg-gdb-server.md):
- Ending winedbg while it is attached ends the program. This tool never kills it; it detaches,
  and winedbg then exits by itself. A winedbg that stays is reported, with its process id.
- A breakpoint left set when the client goes away stays in the code as an int3. The tool
  removes its breakpoints on every exit path, including Ctrl-C and a timeout.
- A bare `c` resumes only one thread there, and the game stays frozen. gdbremote.py sends
  `vCont;c`, which resumes them all.
- winedbg answers Z0 (a software breakpoint) with an empty reply: not supported. It accepts
  hardware breakpoints (Z1) and watchpoints (Z2 to Z4), but under Rosetta they never fired,
  and the game ended soon after each try. `watch` is for winedbg on an x86 host, or for a
  stub given with --connect.
- winedbg's module list (qXfer:libraries) has empty names and the Mac's own library
  addresses, so neither this tool nor lldb lists modules. Use the addresses from Ghidra.
- Started from Steam, Banished shows its launcher first, in the same executable. The launcher
  runs none of the game's frame loop, so a breakpoint there waits for nothing.

What it does not do: write memory or registers, start a program under the debugger, or 32-bit programs' addresses in a 64-bit Wine beyond what winedbg itself gives.
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gdbremote import Remote, RemoteError  # noqa: E402

CROSSOVER = '/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/bin/wine'


def wine_command(args, *rest):
    wine = args.wine or (CROSSOVER if args.bottle and os.path.exists(CROSSOVER) else shutil.which('wine'))
    if not wine:
        sys.exit('no wine found: give --wine PATH')
    env = dict(os.environ)
    if args.bottle:
        return [wine, '--bottle', args.bottle, '--cx-app', 'winedbg.exe', *rest], env
    if args.prefix:
        env['WINEPREFIX'] = os.path.abspath(args.prefix)
    return [wine, 'winedbg', *rest], env


def processes(args):
    """[(pid hex, threads, exe)] from winedbg's `info proc`."""
    cmd, env = wine_command(args, '--command', 'info proc')
    out = subprocess.run(cmd, env=env, capture_output=True, text=True, timeout=120).stdout
    found = []
    for line in out.splitlines():
        m = re.match(r"\s*=?\s*([0-9a-fA-F]{8})\s+(\d+)\s+[\\_ ]*'([^']+)'", line)
        if m:
            found.append((m.group(1).lower(), int(m.group(2)), m.group(3)))
    return found


class Winedbg:
    """winedbg --gdb attached to one process, its GDB port, and its output."""

    def __init__(self, args, pid):
        # winedbg takes a process id as 0x3b8; a bare 000003b8 is a program name to start.
        cmd, env = wine_command(args, '--gdb', '--no-start', hex(int(pid, 16)))
        self.log = []
        self.proc = subprocess.Popen(cmd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                     text=True, bufsize=1)
        deadline = time.monotonic() + 60
        self.port = None
        while self.port is None and time.monotonic() < deadline:
            line = self.proc.stdout.readline()
            if not line:
                break
            self.log.append(line.rstrip())
            m = re.search(r'target remote [^:\s]+:(\d+)', line)
            if m:
                self.port = int(m.group(1))
        if self.port is None:
            sys.exit('winedbg gave no GDB port:\n' + '\n'.join(self.log[-20:]))

    def finish(self):
        """After the detach: wait for winedbg to exit, and keep the rest of its output."""
        try:
            out, _ = self.proc.communicate(timeout=15)
            self.log += out.splitlines()
        except subprocess.TimeoutExpired:
            print(f'winedbg (pid {self.proc.pid}) is still running after the detach. Do not kill it:'
                  ' that ends the program it is attached to.', file=sys.stderr)


def resolve_pid(args):
    if args.pid:
        return args.pid.lower().removeprefix('0x').rjust(8, '0')
    rows = [p for p in processes(args) if p[2].lower() == args.name.lower()]
    if not rows:
        sys.exit(f'no process named {args.name} (winegdb.py procs lists them)')
    return rows[0][0]


def dump_spec(spec, regs):
    """'rcx+0x40:16', '0x1400000:64' or 'rsp:32' -> (address, length)."""
    where, length = spec.rsplit(':', 1)
    base, _, off = where.partition('+')
    addr = regs[base] if base in regs else int(base, 0)
    return addr + (int(off, 0) if off else 0), int(length, 0)


def hexdump(addr, data):
    lines = []
    for i in range(0, len(data), 16):
        row = data[i:i + 16]
        text = ''.join(chr(b) if 32 <= b < 127 else '.' for b in row)
        lines.append(f'{addr + i:#014x}  {row.hex(" "):<47}  {text}')
    return '\n'.join(lines)


def show_regs(regs, as_json, extra=None, all_regs=False):
    if as_json:
        print(json.dumps({**(extra or {}), 'regs': {k: hex(v) for k, v in regs.items()}}))
        return
    if extra:
        print(' '.join(f'{k}={v}' for k, v in extra.items()))
    names = [n for n in regs if all_regs or not n.startswith(('st', 'fctrl', 'fstat', 'ftag', 'fi', 'fo', 'fop', 'xmm', 'ymm', 'mxcsr'))]
    for i in range(0, len(names), 4):
        print('  '.join(f'{n:>6} {regs[n]:#018x}' for n in names[i:i + 4]))


def run(args, r):
    if args.cmd == 'regs':
        threads = r.threads() if args.all_threads else [args.thread]
        for t in threads:
            show_regs(r.registers(t), args.json, {'thread': t} if t else None, args.all_regs)
    elif args.cmd == 'read':
        data = r.read(int(args.addr, 0), int(args.len, 0))
        if args.out:
            open(args.out, 'wb').write(data)
            print(f'{len(data)} bytes to {args.out}')
        elif args.json:
            print(json.dumps({'addr': args.addr, 'hex': data.hex()}))
        else:
            print(hexdump(int(args.addr, 0), data))
    elif args.cmd in ('break', 'watch'):
        addrs = [int(a, 0) for a in args.addrs]
        kind, length = (2, args.len) if args.cmd == 'watch' else (None, 1)
        for n in range(args.hits):
            stop = r.run_to(addrs, args.timeout, kind, length)
            if stop is None:
                print(f'no hit within {args.timeout} s ({n} hits)', file=sys.stderr)
                return 1 if n == 0 else 0
            regs = r.registers(stop.get('thread'))
            show_regs(regs, args.json, {'hit': n + 1, 'pc': hex(r.pc(regs) or 0),
                                        'thread': stop.get('thread', '?')}, args.all_regs)
            for spec in args.dump:
                addr, length = dump_spec(spec, regs)
                data = r.read(addr, length)
                print(json.dumps({'dump': spec, 'addr': hex(addr), 'hex': data.hex()}) if args.json
                      else f'{spec} at {addr:#x}:\n{hexdump(addr, data)}')
            r.step_over_breakpoint()
    return 0


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    p.add_argument('cmd', choices=['procs', 'regs', 'read', 'break', 'watch'])
    p.add_argument('rest', nargs='*')
    p.add_argument('--bottle')
    p.add_argument('--prefix')
    p.add_argument('--wine')
    p.add_argument('--pid')
    p.add_argument('--name')
    p.add_argument('--connect')
    p.add_argument('--thread')
    p.add_argument('--all-threads', action='store_true')
    p.add_argument('--all-regs', action='store_true')
    p.add_argument('--out')
    p.add_argument('--len', type=int, default=4)
    p.add_argument('--hits', type=int, default=1)
    p.add_argument('--timeout', type=float, default=30.0)
    p.add_argument('--dump', action='append', default=[])
    p.add_argument('--json', action='store_true')
    p.add_argument('--log', action='store_true', help="print winedbg's own output at the end")
    if argv is None and len(sys.argv) == 1:
        print(__doc__)
        return 0
    args = p.parse_args(argv)
    if args.cmd == 'read':
        if len(args.rest) != 2:
            p.error('read needs ADDR LEN')
        args.addr, args.len = args.rest
    if args.cmd in ('break', 'watch'):
        if not args.rest:
            p.error(f'{args.cmd} needs at least one ADDR')
        args.addrs = args.rest
    if args.cmd == 'procs':
        if not (args.bottle or args.prefix or args.wine):
            p.error('procs needs --bottle, --prefix or --wine')
        for pid, threads, exe in processes(args):
            print(f'{pid}  {threads:3}  {exe}')
        return 0
    if args.connect:
        host, _, port = args.connect.rpartition(':')
        dbg, host, port = None, host or 'localhost', int(port)
    elif args.pid or args.name:
        dbg = Winedbg(args, resolve_pid(args))
        host, port = 'localhost', dbg.port
    else:
        p.error('give --pid, --name or --connect')
    try:
        with Remote.connect(host, port, hw_per_thread=dbg is not None) as r:
            rc = run(args, r)
    except RemoteError as e:
        print(f'error: {e}', file=sys.stderr)
        rc = 1
    except KeyboardInterrupt:
        rc = 130
    if dbg:
        dbg.finish()
        if args.log:
            print('\n'.join(dbg.log), file=sys.stderr)
    return rc


if __name__ == '__main__':
    sys.exit(main())
