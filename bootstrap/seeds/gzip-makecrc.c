#include <stdio.h>
typedef unsigned long ulg;
int main(void) {
    ulg c;
    int n;
    int k;
    FILE *output = fopen("crc.c", "w");
    if (!output) return 1;
    fprintf(output, "ulg crc_32_tab[] = {\n");
    for (n = 0; n < 256; n++) {
        c = (ulg)n;
        for (k = 0; k < 8; k++) {
            if (c & 1) c = 0xedb88320L ^ (c >> 1);
            else c = c >> 1;
        }
        if (n % 4 == 3) fprintf(output, "  0x%08lxL,\n", c);
        else fprintf(output, "  0x%08lxL, ", c);
    }
    fprintf(output, "};\n");
    if (fclose(output) != 0) return 2;
    return 0;
}
