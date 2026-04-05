#include "hash.h"

uint32_t hash_djb2(const char *str) {
    uint32_t h = 5381;
    int c;
    while ((c = *str++) != 0) {
        h = ((h << 5) + h) + (uint32_t)c;
    }
    return h;
}

uint32_t hash_fnv1a(const void *data, size_t len) {
    const uint8_t *p = data;
    uint32_t h = 0x811c9dc5;
    for (size_t i = 0; i < len; i++) {
        h ^= p[i];
        h *= 0x01000193;
    }
    return h;
}
