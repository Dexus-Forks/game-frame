#!/usr/bin/env python3
"""Write a small capture for tests/run.sh: a made-up game protocol over UDP (a magic, a type
byte, a sequence number, a length and a body) and a short TCP conversation, as classic pcap
with Ethernet frames.

Usage: make_pcap.py OUT.pcap
"""
import random
import struct
import sys


def ipv4(src, dst, proto, body):
    header = struct.pack('>BBHHHBBH4s4s', 0x45, 0, 20 + len(body), 0, 0, 64, proto, 0,
                         bytes(map(int, src.split('.'))), bytes(map(int, dst.split('.'))))
    return header + body


def frame(ip):
    return b'\x02' * 6 + b'\x04' * 6 + b'\x08\x00' + ip


def udp(src, sport, dst, dport, payload):
    return frame(ipv4(src, dst, 17, struct.pack('>HHHH', sport, dport, 8 + len(payload), 0) + payload))


def tcp(src, sport, dst, dport, seq, payload):
    return frame(ipv4(src, dst, 6, struct.pack('>HHIIBBHHH', sport, dport, seq, 0, 0x50, 0x18, 1024, 0, 0) + payload))


def main(out):
    r = random.Random(3)
    pkts = []
    for seq in range(40):
        kind = (1, 2, 3)[seq % 3]
        body = bytes(r.randrange(256) for _ in range({1: 4, 2: 12, 3: 20}[kind]))
        msg = b'GF' + bytes([kind]) + struct.pack('<HH', seq + 100, 7 + len(body)) + body
        pkts.append(udp('10.0.0.2', 50123, '10.0.0.1', 27960, msg))
        pkts.append(udp('10.0.0.1', 27960, '10.0.0.2', 50123, b'ACK' + struct.pack('<H', seq + 100)))
    pkts.append(tcp('10.0.0.2', 50200, '10.0.0.1', 6112, 1000, b'HELLO '))
    pkts.append(tcp('10.0.0.2', 50200, '10.0.0.1', 6112, 1006, b'LOBBY\n'))
    with open(out, 'wb') as f:
        f.write(struct.pack('<IHHiIII', 0xA1B2C3D4, 2, 4, 0, 0, 65535, 1))
        for i, p in enumerate(pkts):
            f.write(struct.pack('<IIII', 1000 + i, 0, len(p), len(p)) + p)


if __name__ == '__main__':
    main(sys.argv[1])
