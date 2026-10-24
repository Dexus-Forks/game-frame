---
id: networking
title: Networking and multiplayer messages
kind: context
status: context
platform: [dos, windows-9x, windows-nt]
runtime: [dosbox, wine, virtual-machine]
hardware: [ports]
api: [dos-ints, win32]
compiler: []
genre: []
symptom: []
tags: [network, multiplayer, ipx, serial, modem, netbios, directplay, winsock, lockstep, packets, messages]
skills: [re-trace, re-observe, re-port, re-verify]
docs: [docs/hardware/other.md, docs/observe/inputs.md, docs/GAME-PATTERNS.md]
---

# Networking and multiplayer messages

## What to know

Identify the transport and the game's message protocol separately. Common transports include
IPX, serial ports, and modems on DOS, plus DirectPlay and Winsock on Windows. Trace the calls
and record the message order, contents, and point in the game loop where each message is handled.

In a lockstep game, another player's commands are inputs to the local simulation. A client-server
game instead uses the server's messages. Replacing IPX or a serial link changes transport; it
does not tell you which protocol data and timing the game must preserve.

The hardware guide lists the common transports. The input guide explains how network messages
become part of a repeatable input record.
