#ifndef HASH_H
#define HASH_H

#include <stdint.h>
#include <stddef.h>

/* DJB2 string hash. */
uint32_t hash_djb2(const char *str);

/* FNV-1a hash over raw bytes. */
uint32_t hash_fnv1a(const void *data, size_t len);

#endif
