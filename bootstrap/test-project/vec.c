#include "vec.h"
#include <stdlib.h>
#include <assert.h>

struct vec vec_new(uint32_t cap) {
    assert(cap > 0 && cap <= 1048576);
    struct vec v;
    v.data = malloc(cap * sizeof(int32_t));
    assert(v.data != NULL);
    v.len = 0;
    v.cap = cap;
    return v;
}

void vec_push(struct vec *v, int32_t val) {
    assert(v->len < v->cap);
    v->data[v->len] = val;
    v->len++;
}

int32_t vec_get(const struct vec *v, uint32_t idx) {
    assert(idx < v->len);
    return v->data[idx];
}

static int cmp_i32(const void *a, const void *b) {
    int32_t x = *(const int32_t *)a;
    int32_t y = *(const int32_t *)b;
    return (x > y) - (x < y);
}

void vec_sort(struct vec *v) {
    if (v->len > 1) {
        qsort(v->data, v->len, sizeof(int32_t), cmp_i32);
    }
}

void vec_free(struct vec *v) {
    free(v->data);
    v->data = NULL;
    v->len = 0;
    v->cap = 0;
}
