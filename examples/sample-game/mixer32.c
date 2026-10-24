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

/* A linked list and records with pointers, for pe32_emu's array/put/get layout helpers. */
struct node { int value; const struct node *next; };

__declspec(dllexport) int __cdecl sum_list(const struct node *n) {
    int s = 0;
    for (; n; n = n->next) s += n->value;
    return s;
}

struct link { int weight; const struct link *to[2]; };

__declspec(dllexport) int __cdecl heaviest_path(const struct link *l) {
    int s = 0;
    while (l) {
        s += l->weight;
        const struct link *a = l->to[0], *b = l->to[1];
        l = !a ? b : !b ? a : (a->weight >= b->weight ? a : b);
    }
    return s;
}

/* A routine that takes its arguments in registers in Watcom's order (EAX, EDX, EBX, ECX), and
 * a caller that loads them with constants: for ExportCallArgs' --regs watcom, which reads the
 * values a register call passes, where the decompiler sees none. */
__declspec(dllexport) int __cdecl reg_sum(void) {
    int a, d, b, c;
    __asm__ volatile ("" : "=a"(a), "=d"(d), "=b"(b), "=c"(c));
    return a + d * 2 + b * 3 + c * 4;
}

__declspec(dllexport) int __cdecl reg_call(void) {
    int r;
    __asm__ volatile ("movl $1234, %%eax\n\tmovl $7, %%edx\n\tmovl $0x55, %%ebx\n\tmovl $3, %%ecx\n\tcall %P1"
                      : "=a"(r) : "i"(reg_sum) : "ebx", "ecx", "edx", "memory");
    return r;
}

/* A routine Ghidra's analysis misses, for exportcov and ghidra-run fill-gaps: hand-written, with
 * no prologue to match, after padding, and reached only through a pointer that the code XORs
 * with a key at run time, so no reference leads to it. 1990s games hid code the same way. */
__asm__ (".text\n\t.balign 16\n\t.byte 0xcc,0xcc,0xcc,0xcc\n\t.balign 16\n"
         "_hidden_triple:\n\tmovl 4(%esp), %eax\n\tleal (%eax,%eax,2), %eax\n\tretl\n");
extern int __cdecl hidden_triple(int);
#define HIDE_KEY 0x5a5a5a5au
static volatile unsigned hidden_slot;

__declspec(dllexport) int __cdecl call_hidden(int x) {
    if (!hidden_slot) hidden_slot = (unsigned)(unsigned long)&hidden_triple ^ HIDE_KEY;
    int (__cdecl *f)(int) = (int (__cdecl *)(int))(unsigned long)(hidden_slot ^ HIDE_KEY);
    return f(x);
}
