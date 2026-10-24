/* A small 32-bit DLL for tools/pe/apitable.py's test (tests/run.sh): a decorated stdcall
 * export, an undecorated stdcall one, two cdecl ones (one returns a float), a void one and a
 * data export. Built with zig cc -target x86-windows-gnu -shared, exports from api.def. */
#include "api.h"

int table_data = 7;

int __stdcall decorated(int a, int b) { return a + b; }
int __stdcall plain_std(int a, int b, int c) { return a * b + c; }
int __cdecl plain_c(int a, int b) { return a - b; }
float __cdecl ratio(int a) { return a / 3.0f; }
void __cdecl nothing(void) { table_data++; }
