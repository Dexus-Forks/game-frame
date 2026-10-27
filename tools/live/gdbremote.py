"""A small client for the GDB remote protocol: registers, memory, breakpoints, always a detach.

    from gdbremote import Remote
    with Remote.connect('localhost', 54101) as r:      # leaves the program running on exit
        print(r.stop)                                   # the stop that the attach caused
        regs = r.registers()                            # {'rip': 0x140206310, ...}
        data = r.read(0x140000000, 64)
        hit = r.run_to([0x140118340], timeout=10)       # None when no breakpoint was hit
        r.step_over_breakpoint()                        # before the next run_to
        r.run_to([0x1401a0000], 10, kind=2, length=4)   # a write watchpoint
        r.libraries()                                   # [(name, base)] when the stub sends them

Inputs: the host and port of any GDB stub: winedbg --gdb (see winegdb.py), QEMU's -gdb,
VMware Fusion's debugStub, 86Box's GDB stub. Outputs: Python values; no printing.

Leaving the `with` block, normally or on an error or Ctrl-C, stops the program if it is
running, removes every breakpoint this client set and sends a detach. When the stub refuses a
software breakpoint (Z0), the client writes an int3 byte itself and puts the original back. A breakpoint left in
the program after the client goes away stays in its code as an int3, and the program stops or
dies when it reaches it.

The register names and sizes come from the stub's target description (qXfer:features). A stub
without one gets GDB's amd64 order for a 'g' reply of 8-byte registers, else i386's.

What it does not do: writing memory or registers, non-stop mode, more than one process. Standard library only.
"""
import re
import socket
import time
import xml.etree.ElementTree as ET

AMD64 = [(n, 8) for n in ('rax rbx rcx rdx rsi rdi rbp rsp r8 r9 r10 r11 r12 r13 r14 r15 rip'
                          .split())] + [(n, 4) for n in 'eflags cs ss ds es fs gs'.split()]
I386 = [(n, 4) for n in 'eax ecx edx ebx esp ebp esi edi eip eflags cs ss ds es fs gs'.split()]
CHUNK = 0x800
INT3 = -1                       # a breakpoint made by writing 0xcc, not by the stub


class RemoteError(Exception):
    pass


def checksum(data):
    return b'%02x' % (sum(data) & 0xff)


def parse_stop(reply):
    """A stop reply as a dict: kind (S, T, W, X), signal, thread, registers by number."""
    s = reply.decode('latin-1')
    out = {'kind': s[:1], 'signal': int(s[1:3], 16) if len(s) >= 3 else 0, 'regs': {}}
    for field in s[3:].split(';'):
        if ':' not in field:
            continue
        key, value = field.split(':', 1)
        if key == 'thread':
            out['thread'] = value
        elif re.fullmatch(r'[0-9a-fA-F]+', key):
            out['regs'][int(key, 16)] = int.from_bytes(bytes.fromhex(value), 'little')
    return out


class Remote:
    def __init__(self, sock, hw_per_thread=False):
        self.sock, self.buf, self.hw_per_thread = sock, b'', hw_per_thread
        self.breakpoints, self.running, self.layout = {}, False, None
        self.stop = None

    @classmethod
    def connect(cls, host, port, timeout=10.0, attempts=20, hw_per_thread=False):
        """hw_per_thread: the stub sets hardware breakpoints and watchpoints in the selected
        thread only (winedbg), so set and clear them in every thread."""
        for i in range(attempts):
            try:
                sock = socket.create_connection((host, port), timeout=timeout)
                break
            except OSError:
                if i == attempts - 1:
                    raise
                time.sleep(0.25)
        r = cls(sock, hw_per_thread)
        r.ask(b'qSupported:xmlRegisters=i386')
        r.stop = parse_stop(r.ask(b'?'))
        return r

    # --- packets

    def _read(self, timeout):
        self.sock.settimeout(timeout)
        data = self.sock.recv(65536)
        if not data:
            raise RemoteError('the stub closed the connection')
        self.buf += data

    def _packet(self, timeout):
        """The next packet's payload; acks it. Skips '+' and '-' between packets."""
        while True:
            start = self.buf.find(b'$')
            end = self.buf.find(b'#', start + 1) if start >= 0 else -1
            if start >= 0 and end >= 0 and len(self.buf) >= end + 3:
                payload = self.buf[start + 1:end]
                self.buf = self.buf[end + 3:]
                self.sock.sendall(b'+')
                return payload
            self._read(timeout)

    def send(self, payload):
        self.sock.sendall(b'$' + payload + b'#' + checksum(payload))

    def ask(self, payload, timeout=10.0):
        """Send one packet and return the reply, skipping console output ('O...')."""
        self.send(payload)
        while True:
            reply = self._packet(timeout)
            if not (reply.startswith(b'O') and reply != b'OK'):
                return reply

    def _expect_ok(self, payload):
        reply = self.ask(payload)
        if reply != b'OK':
            raise RemoteError(f'{payload.decode()} -> {reply.decode("latin-1") or "(empty: not supported)"}')

    # --- registers

    def _xfer(self, name):
        return self._xfer_object(b'features', name)

    def _xfer_object(self, obj, name):
        data, offset = b'', 0
        while True:
            reply = self.ask(b'qXfer:%s:read:%s:%x,%x' % (obj, name.encode(), offset, 0xfff))
            if not reply or reply[:1] not in (b'm', b'l'):
                return None
            data += reply[1:]
            offset = len(data)
            if reply[:1] == b'l':
                return data.decode('latin-1')

    def _regs_from_xml(self, name, out):
        text = self._xfer(name)
        if text is None:
            return False
        root = ET.fromstring(re.sub(r'<!DOCTYPE[^>]*>', '', text))
        for el in root.iter():
            tag = el.tag.split('}')[-1]
            if tag == 'include':
                self._regs_from_xml(el.get('href'), out)
            elif tag == 'reg':
                out.append((el.get('name'), int(el.get('bitsize')) // 8))
        return True

    def register_layout(self, g_bytes):
        if self.layout is None:
            regs = []
            if self._regs_from_xml('target.xml', regs) and regs:
                self.layout = regs
            else:
                self.layout = AMD64 if g_bytes >= 8 * 17 + 4 * 7 else I386
        return self.layout

    def registers(self, thread=None):
        """{name: value} for the stopped thread, or for `thread` (a stub thread id in hex)."""
        if thread:
            self._expect_ok(b'Hg' + thread.encode())
        raw = self.ask(b'g').decode('latin-1')
        if raw.startswith('E'):
            raise RemoteError(f'g -> {raw}')
        self._raw_g = raw
        layout, regs, pos = self.register_layout(len(raw) // 2), {}, 0
        for name, size in layout:
            field = raw[pos:pos + size * 2]
            if len(field) < size * 2:
                break
            if 'x' not in field:
                regs[name] = int.from_bytes(bytes.fromhex(field), 'little')
            pos += size * 2
        return regs

    def set_pc(self, value):
        """Set the stopped thread's pc: with P, else by writing back every register with G."""
        regs = self.registers(self.stop.get('thread') if self.stop else None)
        pos = 0
        for i, (name, size) in enumerate(self.layout):
            if name in ('rip', 'eip', 'pc'):
                field = value.to_bytes(size, 'little').hex()
                if self.ask(b'P%x=%s' % (i, field.encode())) == b'OK':
                    return
                raw = self._raw_g[:pos] + field + self._raw_g[pos + size * 2:]
                self._expect_ok(b'G' + raw.encode())
                return
            pos += size * 2
        raise RemoteError(f'no pc register in {sorted(regs)}')

    def pc(self, regs):
        for name in ('rip', 'eip', 'pc'):
            if name in regs:
                return regs[name]
        return None

    def threads(self):
        found, reply = [], self.ask(b'qfThreadInfo')
        while reply.startswith(b'm'):
            found += reply[1:].decode().split(',')
            reply = self.ask(b'qsThreadInfo')
        return found

    # --- memory

    def read(self, addr, length):
        out = b''
        while len(out) < length:
            n = min(CHUNK, length - len(out))
            reply = self.ask(b'm%x,%x' % (addr + len(out), n))
            if not reply or reply.startswith(b'E'):
                raise RemoteError(f'cannot read {n} bytes at {addr + len(out):#x}: {reply.decode() or "no reply"}')
            out += bytes.fromhex(reply.decode())
        return out

    def write(self, addr, data):
        self._expect_ok(b'M%x,%x:%s' % (addr, len(data), data.hex().encode()))

    # --- running

    def set_breakpoint(self, addr, kind=None, length=1):
        """kind 0 software, 1 hardware, 2 write, 3 read, 4 access watchpoint, INT3 an int3 byte
        written by this client. Without a kind, a software breakpoint, or an int3 byte when
        the stub refuses it (winedbg does: its hardware breakpoints never fired under Rosetta)."""
        if kind is None:
            if self._z(b'Z', 0, addr, length) == b'OK':
                self.breakpoints[addr] = (0, length)
                return 0
            kind = INT3
        if kind == INT3:
            original = self.read(addr, 1)
            self.write(addr, b'\xcc')
            self.breakpoints[addr] = (INT3, original[0])
            return INT3
        kinds = [kind]
        for k in kinds:
            reply = self._z(b'Z', k, addr, length)
            if reply == b'OK':
                self.breakpoints[addr] = (k, length)
                return k
            if reply and k == kinds[-1]:
                raise RemoteError(f'Z{k} at {addr:#x} -> {reply.decode("latin-1")}')
        raise RemoteError(f'the stub sets no breakpoint of kind {"/".join(map(str, kinds))} at {addr:#x}')

    def clear_breakpoint(self, addr):
        kind, length = self.breakpoints.pop(addr)
        if kind == INT3:
            self.write(addr, bytes([length]))
            return
        reply = self._z(b'z', kind, addr, length)
        if reply != b'OK':
            raise RemoteError(f'z{kind} at {addr:#x} -> {reply.decode("latin-1")}')

    def _z(self, op, kind, addr, length):
        packet = op + b'%d,%x,%x' % (kind, addr, length)
        if kind == 0 or not self.hw_per_thread:
            return self.ask(packet)
        reply = b''
        for t in self.threads():
            self.ask(b'Hg' + t.encode())
            reply = self.ask(packet)
            if reply != b'OK':
                return reply
        return reply

    def libraries(self):
        """[(name, first segment address)] from qXfer:libraries, or [] when not supported."""
        text = self._xfer_object(b'libraries', '')
        if not text:
            return []
        out = []
        for lib in ET.fromstring(text).iter('library'):
            seg = lib.find('segment')
            out.append((lib.get('name'), int(seg.get('address'), 0) if seg is not None else None))
        return out

    def interrupt(self, timeout=10.0):
        self.sock.sendall(b'\x03')
        self.stop = parse_stop(self._packet(timeout))
        self.running = False
        return self.stop

    def _resume(self, cmd, timeout):
        self.send(cmd)
        self.running = True
        deadline = time.monotonic() + timeout
        while True:
            left = deadline - time.monotonic()
            if left <= 0:
                return None
            try:
                reply = self._packet(min(left, 1.0))
            except (socket.timeout, TimeoutError):
                continue
            if reply.startswith(b'O') and reply != b'OK':
                continue
            self.running = False
            self.stop = parse_stop(reply)
            if self.stop['kind'] in ('W', 'X'):
                raise RemoteError(f'the program ended ({reply.decode("latin-1")})')
            return self.stop

    def _vcont(self):
        if not hasattr(self, '_has_vcont'):
            self._has_vcont = self.ask(b'vCont?').startswith(b'vCont')
        return self._has_vcont

    def _continue_all(self):
        """Resume every thread. A bare 'c' can resume only the thread last chosen with Hc or
        Hg: winedbg's did, and the game stayed frozen."""
        if self._vcont():
            return b'vCont;c'
        self.ask(b'Hc-1')
        return b'c'

    def run_to(self, addrs, timeout, kind=None, length=1):
        """Set breakpoints (or watchpoints, with a kind) at `addrs`, continue, and return the
        stop, or None (the program is stopped again first) when nothing is hit within
        `timeout` seconds."""
        for a in addrs:
            if a not in self.breakpoints:
                self.set_breakpoint(a, kind, length)
        stop = self._resume(self._continue_all(), timeout)
        if stop is None:
            self.interrupt()
            return None
        pc = self.pc(self.registers(stop.get('thread')))
        if pc is not None and self.breakpoints.get(pc - 1, (None,))[0] == INT3:
            self.set_pc(pc - 1)                 # back onto the instruction the int3 replaced
        return stop

    def step_over_breakpoint(self):
        """One instruction past a breakpoint the program stopped on, with the breakpoint kept."""
        pc = self.pc(self.registers())
        if pc not in self.breakpoints or self.breakpoints[pc][0] in (2, 3, 4):
            return
        kind, length = self.breakpoints[pc]
        self.clear_breakpoint(pc)
        thread = (self.stop or {}).get('thread')
        step = b'vCont;s:' + thread.encode() if thread and self._vcont() else b's'
        try:
            if self._resume(step, 10.0) is None:
                self.interrupt()
        finally:
            self.set_breakpoint(pc, kind, length)

    def detach(self):
        """Stop if running, remove every breakpoint, detach. Safe to call more than once."""
        if self.sock is None:
            return
        try:
            if self.running:
                self.interrupt()
            for a in sorted(self.breakpoints):
                try:
                    self.clear_breakpoint(a)
                except RemoteError:
                    pass
            self.ask(b'D', timeout=5.0)
        except (OSError, RemoteError):
            pass
        finally:
            self.sock.close()
            self.sock = None

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.detach()
        return False
