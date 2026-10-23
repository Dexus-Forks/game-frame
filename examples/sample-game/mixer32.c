/* A 32-bit (x86) DLL for tools/emu/pe32_emu.py: a stdcall mixer step with a saturating add,
 * the kind of routine 1990s games wrote in assembly, and a function that calls Windows, to
 * show how an import is stubbed. Our own code: no game material. */
#include <windows.h>

__declspec(dllexport) void __stdcall mix4(short *out, const short *in, int volume) {
    for (int i = 0; i < 4; i++) {
        int v = out[i] + ((in[i] * volume) >> 16);
        out[i] = v > 32767 ? 32767 : v < -32768 ? -32768 : v;
    }
}

__declspec(dllexport) int add_ticks(int a, int b) {
    return a + b + (int)(GetTickCount() & 0xff);
}
