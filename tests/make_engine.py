#!/usr/bin/env python3
"""Write two test files: a 'game' whose code compares FNV-1a hashes of lower-case names, and a
model file with a 16-bit parent array, joint names and a bind pose of quaternions."""
import math
import random
import struct
import sys

sys.path.insert(0, 'tools/triage')
import hashnames  # noqa: E402

out = sys.argv[1]
rng = random.Random(7)
noise = lambda n: bytes(rng.randrange(256) for _ in range(n))
names = ['PlayerController', 'EnemySpawner', 'door_open', 'WeaponPickup', 'CameraShake', 'HealthBar', 'LevelExit']
game = bytearray(noise(4096)) + b'\0'.join(n.encode() for n in names) + b'\0'
for n in names:
    game += b'\x3d' + struct.pack('<I', hashnames.fnv1a(n.lower().encode())) + b'\x90'
game += noise(2048)
open(f'{out}/game.bin', 'wb').write(game)

parents = [-1, 0, 1, 2, 3, 2, 5, 6, 2, 8, 9, 0, 11, 12, 0, 14, 15]
model = bytearray(noise(300)) + b'Bip01\0Bip01 Pelvis\0Bip01 Spine\0Bip01 Head\0L_Thigh\0'
model += struct.pack(f'<{len(parents)}h', *parents) + noise(37)
for i in range(len(parents)):
    a = 0.1 * i
    model += struct.pack('<4f', 0.0, math.sin(a / 2), 0.0, math.cos(a / 2)) + struct.pack('<4f', i, 0, 1, 1.0)
model += noise(200)
open(f'{out}/rig.mdl', 'wb').write(model)

# Shaders: a Direct3D 9 ps_2_0 with a constant table, a DXBC ps_4_0, SPIR-V and GLSL source.
u32 = lambda *v: struct.pack(f'<{len(v)}I', *v)
ctab = b'CTAB' + b'\0' * 12 + b'FogColor\0\0\0\0'
ps = u32(0xFFFF0200, 0xFFFE | ((len(ctab) // 4) << 16)) + ctab
ps += u32(0x05000051, 0xA00F0000) + struct.pack('<4f', 0.5, 0.6, 0.7, 1.0)  # def c0
ps += u32(0x02000001, 0x800F0800, 0xA0E40000, 0x0000FFFF)                   # mov oC0, c0; end
rdef = b'RDEF' + u32(12) + b'LightDir\0\0\0\0'
shdr = b'SHDR' + u32(8) + u32(0x00000040, 2)
size = 32 + 8 + len(rdef) + len(shdr)
dx = b'DXBC' + b'\0' * 16 + u32(1, size, 2, 40, 40 + len(rdef)) + rdef + shdr
spv = u32(0x07230203, 0x00010000, 0, 10, 0) + u32((5 << 16) | 15, 4, 2) + b'main\0\0\0\0'
spv += u32((4 << 16) | 5, 3) + b'Albedo\0\0' + u32((1 << 16) | 56, 0)
glsl = b'#version 120\nuniform sampler2D waterTex;\nvoid main() { gl_FragColor = texture2D(waterTex, gl_TexCoord[0].xy); }\n'
blob = noise(64) + ps + noise(20) + dx + noise(16) + spv + noise(12) + glsl + noise(32)
open(f'{out}/shaders.bin', 'wb').write(blob)

# Memory snapshots: 2 KB per frame for 10 frames. The port's copy goes wrong at frame 6, in
# player_y (0x0076), and the error spreads to the score at frame 8. The frame counter at 0x0009
# differs in every frame and is ignored by the test.
orig, port = bytearray(), bytearray()
for frame in range(10):
    ram = bytearray(2048)
    ram[0x75], ram[0x76], ram[0x09] = 10 + frame, 50 + 2 * frame, frame
    ram[0x100] = 5 * frame
    mine = bytearray(ram)
    mine[0x09] = 200
    if frame >= 6:
        mine[0x76] += 1
    if frame >= 8:
        mine[0x100] += 5
    orig += ram
    port += mine
open(f'{out}/orig.ram', 'wb').write(orig)
open(f'{out}/port.ram', 'wb').write(port)
open(f'{out}/ram.map', 'w').write('# address size name\n0x0075 1 player_x\n0x0076 1 player_y\n0x0100 2 score\n')

# Hashes computed at start-up: the names are in a 'program', the hashes only in data files, as
# type ids at the start of each record (CRC-32 of the UTF-16 name, as Banished does).
import os  # noqa: E402
import zlib  # noqa: E402
classes = ['TerrainDescription', 'GraphicsMesh', 'ImageBuffer', 'SoundBuffer', 'TutorialImage']
crc16 = lambda n: zlib.crc32(n.encode('utf-16-le')) & 0xFFFFFFFF
prog = bytearray(noise(2048)) + b''.join(n.encode('utf-16-le') + b'\0\0' for n in classes) + noise(512)
open(f'{out}/program.bin', 'wb').write(prog)
os.makedirs(f'{out}/data', exist_ok=True)
for k in range(40):
    name = classes[k % 4]
    open(f'{out}/data/res{k:02}.dat', 'wb').write(b'RSRC' + struct.pack('<II', 1, crc16(name)) + noise(60 + k))

# A skeleton stored as records: a name, the parent index, then the rotation and translation.
bones = [('root', -1), ('hips', 0), ('spine', 1), ('chest', 2), ('neck', 3), ('head', 4), ('leftArm', 3), ('leftHand', 6),
         ('rightArm', 3), ('rightHand', 8), ('leftLeg', 1), ('leftFoot', 10), ('rightLeg', 1), ('rightFoot', 12)]
rec = bytearray(noise(64)) + struct.pack('<I', len(bones))
for k, (name, parent) in enumerate(bones):
    a = 0.07 * k
    rec += struct.pack('<I', len(name)) + name.encode() + struct.pack('<i', parent)
    rec += struct.pack('<4f', 0.0, 0.0, math.sin(a / 2), math.cos(a / 2)) + struct.pack('<3f', 0, k, 0)
open(f'{out}/bones.mdl', 'wb').write(rec + noise(64))
