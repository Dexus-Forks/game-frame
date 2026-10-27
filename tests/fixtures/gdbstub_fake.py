#!/usr/bin/env python3
"""A fake GDB stub for testing tools/live/gdbremote.py and winegdb.py without a debugger.

Usage: gdbstub_fake.py LOG [--hit] [--no-z0]

Listens on a free port on 127.0.0.1 and prints the port. Serves one client: an amd64 machine
with no target description, "MZ" at 0x140000000, and two threads. `c` stops at the first
breakpoint after a short wait with --hit, and never stops without it (only an interrupt,
0x03, stops it). With --no-z0, Z0 gets an empty reply (not supported), as winedbg gives; an
int3 byte written with M is then a breakpoint, which stops with rip one byte past it, as on
x86. P gets an empty reply; G sets rip. Every packet received is written to LOG, one per line, with "^C" for an interrupt.
"""
import socket
import sys
import time

log_path = sys.argv[1]
hit = '--hit' in sys.argv
no_z0 = '--no-z0' in sys.argv
memory = {0x140000000 + i: b for i, b in enumerate(b'MZ\x90\x00')}
memory[0x140001000] = 0x48
breakpoints, rip = set(), 0x7ffff000


def regs():
    vals = [0] * 16 + [rip]
    return ''.join(v.to_bytes(8, 'little').hex() for v in vals) + '00000000' * 7


def reply_to(p):
    global rip
    if p.startswith('qSupported'):
        return 'PacketSize=4000'
    if p == '?':
        return 'T05thread:01;'
    if p == 'g':
        return regs()
    if p.startswith('H'):
        return 'OK'
    if p == 'qfThreadInfo':
        return 'm01,02'
    if p == 'qsThreadInfo':
        return 'l'
    if p.startswith('m'):
        addr, n = (int(x, 16) for x in p[1:].split(','))
        return ''.join(f'{memory.get(addr + i, 0):02x}' for i in range(n))
    if p[:1] in 'Zz':
        kind, addr, _ = p[1:].split(',')
        if kind == '0' and no_z0:
            return ''
        (breakpoints.add if p[0] == 'Z' else breakpoints.discard)(int(addr, 16))
        return 'OK'
    if p.startswith('M'):
        where, data = p[1:].split(':')
        addr = int(where.split(',')[0], 16)
        for i in range(0, len(data), 2):
            memory[addr + i // 2] = int(data[i:i + 2], 16)
        return 'OK'
    if p.startswith('G'):
        rip = int.from_bytes(bytes.fromhex(p[1 + 16 * 16:1 + 17 * 16]), 'little')
        return 'OK'
    if p == 'c':
        int3 = [a for a, b in memory.items() if b == 0xcc]
        if hit and (breakpoints or int3):
            time.sleep(0.2)
            rip = min(breakpoints) if breakpoints else min(int3) + 1
            return 'T05thread:01;'
        return None                     # running until interrupted
    if p == 's':
        rip += 1
        return 'T05thread:01;'
    if p == 'D':
        return 'OK'
    return ''


def main():
    server = socket.socket()
    server.bind(('127.0.0.1', 0))
    server.listen(1)
    print(server.getsockname()[1], flush=True)
    conn, _ = server.accept()
    log, buf = open(log_path, 'w'), b''
    while True:
        data = conn.recv(4096)
        if not data:
            break
        buf += data
        while True:
            buf = buf.lstrip(b'+-')
            if buf.startswith(b'\x03'):
                buf = buf[1:]
                log.write('^C\n')
                send(conn, 'T02thread:01;')
                continue
            end = buf.find(b'#')
            if not buf.startswith(b'$') or end < 0 or len(buf) < end + 3:
                break
            packet, buf = buf[1:end].decode(), buf[end + 3:]
            conn.sendall(b'+')
            log.write(packet + '\n')
            log.flush()
            answer = reply_to(packet)
            if answer is not None:
                send(conn, answer)
            if packet == 'D':
                conn.close()
                return


def send(conn, text):
    data = text.encode()
    conn.sendall(b'$' + data + b'#' + b'%02x' % (sum(data) & 0xff))


if __name__ == '__main__':
    main()
