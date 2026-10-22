#include <stdio.h>
__declspec(noinline) void save(int n) { volatile char name[16]; name[0]='S';name[1]='A';name[2]='V';name[3]='E';name[4]='0'+n;name[5]='.';name[6]='D';name[7]='A';name[8]='T';name[9]=0; printf("%s", (char*)name); }
int main(void){ save(1); return 0; }
