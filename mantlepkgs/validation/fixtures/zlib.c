#include <stdio.h>
#include <string.h>
#include <zlib.h>

int main(void) {
    static const unsigned char input[] = "Mantle zlib round-trip: \x00\x01\xff deterministic payload";
    unsigned char compressed[256], restored[256];
    uLongf compressed_length = sizeof(compressed), restored_length = sizeof(restored);
    if (compress2(compressed, &compressed_length, input, sizeof(input), Z_BEST_COMPRESSION) != Z_OK)
        return 1;
    if (uncompress(restored, &restored_length, compressed, compressed_length) != Z_OK)
        return 2;
    if (restored_length != sizeof(input) || memcmp(input, restored, sizeof(input)) != 0)
        return 3;
    compressed[0] ^= 0xff;
    restored_length = sizeof(restored);
    if (uncompress(restored, &restored_length, compressed, compressed_length) != Z_DATA_ERROR)
        return 4;
    puts("zlib: round-trip and corrupt-stream rejection passed");
    return 0;
}
