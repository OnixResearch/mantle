#ifndef VEC_H
#define VEC_H

#include <stdint.h>

/* Fixed-capacity growable int32 array. */
struct vec {
    int32_t *data;
    uint32_t len;
    uint32_t cap;
};

struct vec vec_new(uint32_t cap);
void vec_push(struct vec *v, int32_t val);
int32_t vec_get(const struct vec *v, uint32_t idx);
void vec_sort(struct vec *v);
void vec_free(struct vec *v);

#endif
