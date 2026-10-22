// ==== 140001000 FUN_140001000
void FUN_140001000(longlong param_1,float param_2)
{
  *(float *)(param_1 + 0x10) = param_2;
  *(int *)(param_1 + 0x18) = *(int *)(param_1 + 0x18) + 1;
  if (*(char *)(param_1 + 0x1c) != '\0') {
    *(undefined8 *)(param_1 + 0x20) = 0;
  }
  return;
}
// ==== 140001100 FUN_140001100
int FUN_140001100(int *param_1)
{
  if (param_1[6] == 3) {
    *param_1 = param_1[1];
    return *(int *)((longlong)param_1 + 0x1c);
  }
  return param_1[4];
}
