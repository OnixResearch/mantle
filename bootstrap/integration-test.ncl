# Integration test: build a multi-file C project using ONLY the
# bootstrap chain. No Nix-provided tools in the build.
#
# Bootstrap chain exercised:
#   musl-gcc (fetched seed) → make (from source) → dash (from source)
#   All three are inputs; dash is the builder shell.
#
# The test project has 3 source files, 2 headers, and a Makefile.
# Sources are embedded as heredocs — no external fetch needed.
#
# Usage:
#   crunch build bootstrap/integration-test.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let toolchain = crunch.fetchTarball {
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = "musl-gcc",
} in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in

{
  name = "integration-test",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep \
                 tr head tail sort wc basename dirname \
                 install find touch true false tee; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"

      # Find bootstrap tools.
      TOOLCHAIN=""
      for d in /nix/store/*-musl-gcc; do
        if [ -d "$d/bin" ]; then TOOLCHAIN="$d"; break; fi
      done
      MAKE=""
      for d in /nix/store/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d/bin/make"; break; fi
      done
      DASH=""
      for d in /nix/store/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d/bin/dash"; break; fi
      done
      for tool in TOOLCHAIN MAKE DASH; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
      done

      export SHELL="$DASH"
      export CC="$TOOLCHAIN/bin/x86_64-linux-musl-gcc"

      mkdir -p /tmp/build
      cd /tmp/build

      # ── Embedded source files ──
      cat > vec.h << 'SRCEOF'
#ifndef VEC_H
#define VEC_H
#include <stdint.h>
struct vec { int32_t *data; uint32_t len; uint32_t cap; };
struct vec vec_new(uint32_t cap);
void vec_push(struct vec *v, int32_t val);
int32_t vec_get(const struct vec *v, uint32_t idx);
void vec_sort(struct vec *v);
void vec_free(struct vec *v);
#endif
SRCEOF

      cat > vec.c << 'SRCEOF'
#include "vec.h"
#include <stdlib.h>
#include <assert.h>
struct vec vec_new(uint32_t cap) {
    assert(cap > 0 && cap <= 1048576);
    struct vec v;
    v.data = malloc(cap * sizeof(int32_t));
    assert(v.data != NULL);
    v.len = 0; v.cap = cap;
    return v;
}
void vec_push(struct vec *v, int32_t val) {
    assert(v->len < v->cap);
    v->data[v->len++] = val;
}
int32_t vec_get(const struct vec *v, uint32_t idx) {
    assert(idx < v->len);
    return v->data[idx];
}
static int cmp_i32(const void *a, const void *b) {
    int32_t x = *(const int32_t *)a, y = *(const int32_t *)b;
    return (x > y) - (x < y);
}
void vec_sort(struct vec *v) {
    if (v->len > 1) qsort(v->data, v->len, sizeof(int32_t), cmp_i32);
}
void vec_free(struct vec *v) {
    free(v->data); v->data = NULL; v->len = 0; v->cap = 0;
}
SRCEOF

      cat > hash.h << 'SRCEOF'
#ifndef HASH_H
#define HASH_H
#include <stdint.h>
#include <stddef.h>
uint32_t hash_djb2(const char *str);
uint32_t hash_fnv1a(const void *data, size_t len);
#endif
SRCEOF

      cat > hash.c << 'SRCEOF'
#include "hash.h"
uint32_t hash_djb2(const char *str) {
    uint32_t h = 5381; int c;
    while ((c = *str++) != 0) h = ((h << 5) + h) + (uint32_t)c;
    return h;
}
uint32_t hash_fnv1a(const void *data, size_t len) {
    const uint8_t *p = data; uint32_t h = 0x811c9dc5;
    for (size_t i = 0; i < len; i++) { h ^= p[i]; h *= 0x01000193; }
    return h;
}
SRCEOF

      cat > main.c << 'SRCEOF'
#include <stdio.h>
#include <assert.h>
#include "vec.h"
#include "hash.h"
static void test_vec(void) {
    struct vec v = vec_new(16);
    vec_push(&v, 42); vec_push(&v, 7); vec_push(&v, 99);
    vec_push(&v, -3); vec_push(&v, 0);
    assert(v.len == 5);
    assert(vec_get(&v, 0) == 42);
    vec_sort(&v);
    assert(vec_get(&v, 0) == -3);
    assert(vec_get(&v, 1) == 0);
    assert(vec_get(&v, 2) == 7);
    assert(vec_get(&v, 3) == 42);
    assert(vec_get(&v, 4) == 99);
    vec_push(&v, 10); vec_sort(&v);
    assert(vec_get(&v, 3) == 10);
    assert(v.len == 6);
    vec_free(&v);
    printf("  vec: 10 assertions ok\n");
}
static void test_hash(void) {
    uint32_t h1 = hash_djb2("hello"), h2 = hash_djb2("world");
    assert(h1 != h2);
    assert(h1 == hash_djb2("hello"));
    int32_t a = 1, b = 2;
    assert(hash_fnv1a(&a, sizeof(a)) != hash_fnv1a(&b, sizeof(b)));
    assert(hash_djb2("") == 5381);
    assert(hash_fnv1a("", 0) == 0x811c9dc5);
    printf("  hash: 6 assertions ok\n");
}
int main(void) {
    printf("bootstrap integration test\n");
    test_vec();
    test_hash();
    printf("all tests passed\n");
    return 0;
}
SRCEOF

      cd /tmp/build

      echo "=== Compiling with musl-gcc ==="
      CFLAGS="-Wall -Wextra -O2 -static"
      for f in main.c vec.c hash.c; do
        echo "CC $f"
        $CC $CFLAGS -c $f || exit 1
      done

      echo "LINK test-project"
      $CC -static -o test-project main.o vec.o hash.o || exit 1

      echo "=== Running tests ==="
      ./test-project || exit 1

      # Now test make-driven rebuild from clean.
      rm -f *.o test-project

      echo "=== Rebuild with make ==="
      # Write Makefile using dash + printf for tab characters.
      "$DASH" -c '
        T=$(printf "\t")
        echo "all: test-project" > Makefile
        echo "${T}./test-project" >> Makefile
        echo "" >> Makefile
        echo "test-project: main.o vec.o hash.o" >> Makefile
        echo "${T}\$(CC) \$(LDFLAGS) -o test-project main.o vec.o hash.o" >> Makefile
        echo "" >> Makefile
        echo ".c.o:" >> Makefile
        echo "${T}\$(CC) \$(CFLAGS) -c \$<" >> Makefile
        echo "" >> Makefile
        echo "install: test-project" >> Makefile
        echo "${T}mkdir -p \$(DESTDIR)/bin" >> Makefile
        echo "${T}cp test-project \$(DESTDIR)/bin/" >> Makefile
      ' || exit 1

      cat Makefile
      "$MAKE" SHELL="$DASH" \
              CC="$CC" CFLAGS="-Wall -Wextra -O2 -static" LDFLAGS="-static" \
              DESTDIR="$out" all install || exit 1

      echo "=== Verify output ==="
      ls -la $out/bin/test-project
    "%,
  ],
  inputs = [toolchain, gnumake, dash],
} | crunch.Derivation
