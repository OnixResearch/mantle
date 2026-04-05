#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "vec.h"
#include "hash.h"

static void test_vec(void) {
    struct vec v = vec_new(16);

    vec_push(&v, 42);
    vec_push(&v, 7);
    vec_push(&v, 99);
    vec_push(&v, -3);
    vec_push(&v, 0);

    assert(v.len == 5);
    assert(vec_get(&v, 0) == 42);
    assert(vec_get(&v, 4) == 0);

    vec_sort(&v);

    assert(vec_get(&v, 0) == -3);
    assert(vec_get(&v, 1) == 0);
    assert(vec_get(&v, 2) == 7);
    assert(vec_get(&v, 3) == 42);
    assert(vec_get(&v, 4) == 99);

    /* Sorted order preserved after push+sort cycle. */
    vec_push(&v, 10);
    vec_sort(&v);
    assert(vec_get(&v, 0) == -3);
    assert(vec_get(&v, 3) == 10);
    assert(v.len == 6);

    vec_free(&v);
    printf("  vec: ok (%u tests)\n", 10);
}

static void test_hash(void) {
    /* DJB2: known values. */
    uint32_t h1 = hash_djb2("hello");
    uint32_t h2 = hash_djb2("world");
    assert(h1 != h2);
    assert(h1 == hash_djb2("hello")); /* deterministic */

    /* FNV-1a: different inputs → different hashes. */
    int32_t a = 1, b = 2;
    uint32_t fa = hash_fnv1a(&a, sizeof(a));
    uint32_t fb = hash_fnv1a(&b, sizeof(b));
    assert(fa != fb);

    /* Empty string edge case. */
    assert(hash_djb2("") == 5381);
    assert(hash_fnv1a("", 0) == 0x811c9dc5);

    printf("  hash: ok (%u tests)\n", 6);
}

int main(void) {
    printf("bootstrap integration test\n");
    test_vec();
    test_hash();
    printf("all tests passed\n");
    return 0;
}
