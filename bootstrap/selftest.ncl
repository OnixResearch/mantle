# Self-test: compile a C program using ONLY from-source tools.
#
# This is the proof that the reduced bootstrap seed can be removed.
# The dependency tree is:
#   gcc (from source) + binutils (from source) + musl (from source)
#   → selftest binary
#
# The reduced bootstrap seed is still an *indirect* dependency (it was used
# to build gcc/binutils/musl), but this derivation's direct inputs
# are all from-source. A 3-stage bootstrap would eliminate even the
# indirect dependency.
#
# Usage:
#   crunch build bootstrap/selftest.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let seed = import "seed.ncl" in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in
let binutils = (import "binutils.ncl") in
let musl = (import "musl.ncl") in
let gcc = (import "gcc.ncl") in

{
  name = "selftest",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink sleep touch true false; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done

      # Find from-source tools (NOT the reduced bootstrap seed).
      GCC=""
      for d in $NIX_STORE/*-gcc; do
        if [ -x "$d/bin/gcc" ]; then GCC="$d"; break; fi
      done
      BINUTILS=""
      for d in $NIX_STORE/*-binutils; do
        if [ -d "$d/bin" ]; then BINUTILS="$d"; break; fi
      done
      MUSL=""
      for d in $NIX_STORE/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done
      DASH=""
      for d in $NIX_STORE/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d"; break; fi
      done
      MAKE=""
      for d in $NIX_STORE/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d/bin/make"; break; fi
      done

      for tool in GCC BINUTILS MUSL DASH MAKE; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
        echo "$tool=$val"
      done

      $BB ln -sf "$DASH" /tmp/tools/sh

      export PATH="/tmp/tools:$GCC/bin:$BINUTILS/bin"

      # The from-source binutils are dynamically linked, needing the
      # musl dynamic linker. Create this symlink pointing at the
      # from-source musl's libc.so (which IS the dynamic linker).
      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf $MUSL/lib/libc.so /lib/%{seed_dynamic_linker} 2>/dev/null || true

      # Create unprefixed symlinks for binutils tools.
      # collect2 looks for plain 'ld', not the target-prefixed linker.
      for tool in as ld ld.bfd ar nm objcopy objdump ranlib readelf strip; do
        if [ -x "$BINUTILS/bin/%{seed_target}-$tool" ] && [ ! -e "/tmp/tools/$tool" ]; then
          $BB ln -sf "$BINUTILS/bin/%{seed_target}-$tool" /tmp/tools/$tool
        fi
      done

      echo "=== Dynamic linker ==="
      ls -la /lib/%{seed_dynamic_linker} 2>/dev/null || echo "/lib/ld-musl not available"

      # Verify: the gcc we're using is the FROM-SOURCE one, not the fetched seed.
      echo "=== GCC version ==="
      $GCC/bin/gcc --version | head -1

      # Write a test program that exercises multiple features.
      mkdir -p /tmp/build
      cd /tmp/build

      cat > selftest.c << 'SRCEOF'
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

/* Test basic arithmetic, control flow, and stdlib. */
static int fibonacci(int n) {
    assert(n >= 0 && n <= 46);
    if (n <= 1) return n;
    int a = 0, b = 1;
    for (int i = 2; i <= n; i++) {
        int t = a + b;
        a = b;
        b = t;
    }
    return b;
}

/* Test string operations and malloc. */
static char *reverse(const char *s) {
    int len = strlen(s);
    char *r = malloc(len + 1);
    assert(r != NULL);
    for (int i = 0; i < len; i++) {
        r[i] = s[len - 1 - i];
    }
    r[len] = '\0';
    return r;
}

/* Test struct and pointer operations. */
struct point { int x; int y; };

static int manhattan(struct point a, struct point b) {
    int dx = a.x - b.x;
    int dy = a.y - b.y;
    if (dx < 0) dx = -dx;
    if (dy < 0) dy = -dy;
    return dx + dy;
}

int main(void) {
    printf("=== crunch bootstrap self-test ===\n");
    printf("Compiled with from-source GCC, no fetched binaries.\n\n");

    /* Fibonacci */
    assert(fibonacci(0) == 0);
    assert(fibonacci(1) == 1);
    assert(fibonacci(10) == 55);
    assert(fibonacci(20) == 6765);
    printf("  fibonacci: 4 assertions ok\n");

    /* String reverse */
    char *r1 = reverse("hello");
    assert(strcmp(r1, "olleh") == 0);
    char *r2 = reverse("");
    assert(strcmp(r2, "") == 0);
    char *r3 = reverse("a");
    assert(strcmp(r3, "a") == 0);
    free(r1); free(r2); free(r3);
    printf("  reverse: 3 assertions ok\n");

    /* Struct/pointer */
    struct point p1 = {3, 7};
    struct point p2 = {-2, 4};
    assert(manhattan(p1, p2) == 8);
    struct point p3 = {0, 0};
    assert(manhattan(p1, p3) == 10);
    printf("  manhattan: 2 assertions ok\n");

    /* malloc stress */
    int total = 0;
    for (int i = 0; i < 1000; i++) {
        int *p = malloc(sizeof(int) * 100);
        assert(p != NULL);
        for (int j = 0; j < 100; j++) p[j] = j;
        total += p[99];
        free(p);
    }
    assert(total == 99000);
    printf("  malloc: 1001 assertions ok\n");

    printf("\nall tests passed (1010 assertions)\n");
    return 0;
}
SRCEOF

      echo "=== Testing as ==="
      $BINUTILS/bin/%{seed_target}-as --version 2>&1 | head -1 || echo "as not working"

      echo "=== Compiling with from-source gcc ==="
      # Use explicit paths to ensure we're using from-source tools.
      # -B tells gcc where to find as/ld and crt*.o/libc.
      $GCC/bin/gcc \
        -static \
        -B$BINUTILS/bin/ \
        -B$BINUTILS/bin/%{seed_target}- \
        -B$MUSL/lib/ \
        --sysroot=$MUSL \
        -isystem $MUSL/include \
        -O2 -Wall -Wextra \
        -o selftest selftest.c \
        2>&1 || exit 1

      echo "=== Running selftest ==="
      ./selftest || exit 1

      # Install the binary.
      mkdir -p $out/bin
      cp selftest $out/bin/

      echo "=== Verify ==="
      ls -la $out/bin/selftest
    "%,
  ],
  inputs = [gnumake, dash, binutils, musl, gcc],
} | crunch.Derivation
