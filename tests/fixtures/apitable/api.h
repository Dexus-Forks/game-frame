/* The API of api.c, for tools/pe/apitable.py's test: one call of each kind it tells apart. */
int __stdcall decorated(int a, int b);
int __stdcall plain_std(int a, int b, int c);
int __cdecl plain_c(int a, int b);
float __cdecl ratio(int a);
void __cdecl nothing(void);
extern int table_data;
