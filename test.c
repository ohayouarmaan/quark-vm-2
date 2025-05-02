#include <stdio.h>
#include <stdlib.h>

void* add(int a, int b) {
  int* addition = (int*)malloc(sizeof(int));
  *addition = a + b;
  return (void*)addition;
}
