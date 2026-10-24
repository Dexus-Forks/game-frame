"""The keyboard: scan codes on port 0x60, delivered as IRQ 1 through INT 9's handler.
Part of DosEnv (see __init__.py); the methods use its state.

    env.press(0x39, at=10, hold=3)        # Space down at tick 10, up at tick 13
    env.press(0xE048, at=12)              # an extended key (Up): E0 48, then E0 C8
    env.key_script('10:39:3 12:e048')     # the same as text, for a capture's script
    env.key_clock = lambda: frame         # count `at` and `hold` in the script's frames instead

A due key puts its make code (and later its break code, scan | 0x80) in the keyboard
controller's queue. One byte at a time sits on port 0x60; the PIC raises IRQ 1 through the
INT 9 vector (the program's handler, or the BIOS's) while IRQ 1 is unmasked (port 0x21 bit 1)
and the last one was acknowledged (EOI: 0x20 or 0x61 written to port 0x20). Port 0x64 bit 0
says a byte is waiting. The BIOS's INT 9 reads the byte, puts make codes in env.keys with
their US-layout character (so INT 16h sees them), and sends the EOI. It does not emulate key
repeat while held, shift states, the keyboard LEDs or commands sent to the keyboard.
"""
import re

# US layout, unshifted: scan code -> ASCII, for the BIOS's INT 9.
ASCII = {0x01: 0x1B, 0x0E: 0x08, 0x0F: 0x09, 0x1C: 0x0D, 0x39: 0x20}
for row, first in (('1234567890-=', 0x02), ('qwertyuiop[]', 0x10), ("asdfghjkl;'`", 0x1E),
                   ('\\zxcvbnm,./', 0x2B)):
    ASCII.update({first + i: ord(c) for i, c in enumerate(row)})

# How many instructions to wait after a slice before trying IRQ 1 again: short, so a key
# arrives soon after the timer's handler has returned.
KEY_SLICE = 2000


class Keyboard:
    def init_keyboard(self):
        self.key_events = []          # (time, byte): bytes still to enter the controller
        self.key_queue = []           # bytes in the controller, waiting for port 0x60
        self.key_data = 0             # the byte on port 0x60
        self.key_in_service = False   # IRQ 1 delivered, no EOI yet
        self.key_clock = lambda: self.irq0_count
        self.irq1_count = 0
        self.bios_e0 = False

    def press(self, scan, at=None, hold=1):
        """Press `scan` at time `at` (now when None) and release it `hold` units later. Time is
        env.key_clock(): IRQ 0 deliveries unless the script sets its own clock. A scan code
        above 0xFF (0xE048) is an extended key, sent with its E0 prefix."""
        at = self.key_clock() if at is None else at
        prefix = [0xE0] if scan > 0xFF else []
        code = scan & 0x7F
        self.key_events += [(at, b) for b in prefix + [code]]
        self.key_events += [(at + hold, b) for b in prefix + [code | 0x80]]
        self.key_events.sort(key=lambda e: e[0])

    def key_script(self, text):
        """Presses from text: `at:scan[:hold]` items (scan in hex), separated by spaces or commas."""
        for item in re.split(r'[\s,]+', text.strip()):
            if item:
                at, scan, *hold = item.split(':')
                self.press(int(scan, 16), int(at), int(hold[0]) if hold else 1)

    def keyboard_slice(self, delivered):
        """Between slices: move due bytes into the controller and raise IRQ 1 when it can.
        `delivered` says IRQ 0 went in this slice (one interrupt per slice). Returns how many
        instructions to run before the next try, or None when nothing waits."""
        now = self.key_clock()
        while self.key_events and self.key_events[0][0] <= now:
            self.key_queue.append(self.key_events.pop(0)[1])
        if not self.key_queue:
            return None
        if not delivered and not self.key_in_service and not self.ports.get(('mask', 0x21), 0) & 0x02:
            self.key_data = self.key_queue.pop(0)
            v = self.vector(9)
            if self.emu.interrupt(v.seg, v.off):
                self.key_in_service = True
                self.irq1_count += 1
            else:
                self.key_queue.insert(0, self.key_data)
        return KEY_SLICE

    def bios_int9(self, uc, address, size, _):
        """The BIOS's INT 9: the byte from port 0x60 into env.keys (make codes only), then EOI."""
        b = self.key_data
        if b == 0xE0:
            self.bios_e0 = True
        else:
            if not b & 0x80:
                self.keys.append(b << 8 | (0 if self.bios_e0 else ASCII.get(b, 0)))
            self.bios_e0 = False
        self.key_in_service = False
