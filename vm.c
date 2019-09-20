#include <stdio.h>

int main (int argc, char** argv) {
  if (argc < 2) {
    printf("Usage: %s [filename]", argv[0]);
  }
  FILE fp = *fopen(argv[1], "r");

  int ip;

  while (1) {
    switch()
    ip++;
  }
  return 0;
}
