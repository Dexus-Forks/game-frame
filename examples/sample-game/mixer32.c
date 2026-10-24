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

/* (a * b) / 65536 by the "magic number" add of 1.5 x 2^68, as 1990s games converted 16.16
 * products: the result's rounding is the FPU's current mode (pe32_emu's fpu_cw). */
__declspec(dllexport) int __cdecl fix_mul(int a, int b) {
    static const double magic = 442721857769029238784.0;   /* 1.5 * 2^68 */
    union { double d; int i[2]; } u;
    __asm__ volatile ("fildl %1\n\tfimull %2\n\tfaddl %3\n\tfstpl %0"
                      : "=m"(u.d) : "m"(a), "m"(b), "m"(magic));
    return u.i[0];
}

/* Formats through a library routine, for pe32_emu's until=: stop at the call, read the arguments. */
__declspec(dllexport) int __cdecl report(int a, int b) {
    char buf[32];
    return wsprintfA(buf, "%d %d", a * 2, b + 1);
}
