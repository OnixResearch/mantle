#include <stdio.h>
#include <string.h>

int main(int argc, char **argv) {
    FILE *output;
    if (argc != 3) return 64;
    if (strcmp(argv[1], "-c") != 0) return 65;
    if (strcmp(argv[2], "write smoke/out smoke-ok") != 0) return 66;
    output = fopen("smoke/out", "w");
    if (!output) return 67;
    if (fputs("smoke-ok\n", output) < 0) return 68;
    if (fclose(output) != 0) return 69;
    return 0;
}
