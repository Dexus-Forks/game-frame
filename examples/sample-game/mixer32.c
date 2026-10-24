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

/* A table and two writers, for tools/triage/refs.py: it must list both writes and the read. */
__declspec(dllexport) int order_table[8];

__declspec(dllexport) void __stdcall swap_order(int a, int b) {
    int t = order_table[a];
    order_table[a] = order_table[b];
    order_table[b] = t;
}

__declspec(dllexport) void reset_order(void) {
    for (int i = 0; i < 8; i++) order_table[i] = i;
}
