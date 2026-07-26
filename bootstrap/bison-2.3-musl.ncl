# Build bison 2.3 linked against musl using tcc-musl-v2.
#
# Legacy bison needed by some older autotools-era configure scripts.
# Adapted from live-bootstrap steps/bison-2.3.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-native.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in

let bison_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/bison/bison-2.3.tar.bz2",
  hash = "sha256-A7EYDEuY+Hw8DHostT2RBv+kjkMumqDlshfpCi2I77U=",
  name = "bison-2.3-src",
} in

{
  name = "bison-2.3-musl",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for d in $NIX_STORE/*-"$1"; do
          if [ -d "$d" ]; then echo "$d"; return; fi
        done
        echo "ERROR: input $1 not found" >&2; exit 1
      }

      TCC=$(find_input tcc-0.9.27-musl-v2)
      MUSL=$(find_input musl-1.1.24-native-candidate)
      STAGE0=$(find_input stage0-posix)
      M4=$(find_input m4-1.4.7-musl)
      SRC=$(find_input bison-2.3-src)

      export PATH="$TCC/bin:$M4/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/bison23-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" bison-2.3
      $BB chmod -R u+w bison-2.3
      cd bison-2.3

      # Generate config.h
      $BB cat > config.h << CONFIG
#define PACKAGE "bison"
#define PACKAGE_NAME "bison"
#define PACKAGE_VERSION "2.3"
#define VERSION "2.3"
#define HAVE_STDLIB_H 1
#define HAVE_STRING_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_STDINT_H 1
#define HAVE_INTTYPES_H 1
#define HAVE_LOCALE_H 1
#define HAVE_FCNTL_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_WCHAR_H 1
#define HAVE_WCTYPE_H 1
#define HAVE_CLOCK_T 1
#define STDC_HEADERS 1
#define M4 "$M4/bin/m4"
#define M4_GNU_OPTION ""
#define LOCALEDIR "$out/share/locale"
#define PKGDATADIR "$out/share/bison"
#define EXEEXT ""
#define _GL_UNUSED
#define __getopt_argv_const const
CONFIG

      CFLAGS="-I. -Ilib -I$MUSL/include -DHAVE_CONFIG_H"

      $BB cat > mantle_runtime_compat.c <<'RUNTIME_EOF'
#include <limits.h>
#include <stddef.h>

#define MANTLE_BISON_TEXT_BYTES_MAX (1UL << 20)
#define MANTLE_ASCII_LIMIT 128

int
mbsnwidth(text, text_bytes, flags)
const char *text;
size_t text_bytes;
int flags;
{
    size_t index = 0;
    int width = 0;

    (void) flags;
    if (text == NULL)
        return -1;
    if (text_bytes > MANTLE_BISON_TEXT_BYTES_MAX)
        return -1;
    while (index < text_bytes && text[index] != '\0') {
        if ((unsigned char) text[index] >= MANTLE_ASCII_LIMIT)
            return -1;
        width++;
        index++;
    }
    return width;
}

/* These three symbols satisfy TCC-v2 code generation in Bison's optional
 * bitset/time statistics objects. The admitted behavior matrix does not claim
 * those statistics surfaces; hash-table sizing never calls these helpers. */
double
__floatundidf(value)
unsigned long value;
{
    double upper;

    if (value <= (unsigned long) LONG_MAX)
        return (double) (long) value;
    upper = (double) (long) (value >> 1);
    return upper * (double) 2 + (double) (value & (unsigned long) 1);
}

float
__floatundisf(value)
unsigned long value;
{
    return (float) __floatundidf(value);
}

static unsigned long
mantle_float_to_ulong_bounded(value, valid_out)
double value;
int *valid_out;
{
    double remaining;
    double unit = (double) 1;
    unsigned long bit = (unsigned long) 1;
    unsigned long result = (unsigned long) 0;
    int magnitude_bit_count = sizeof(unsigned long) * CHAR_BIT - 1;
    int step_count = 1;

    *valid_out = 0;
    if (value != value || value < (double) 0)
        return result;
    remaining = value;
    while (step_count < magnitude_bit_count) {
        unit *= (double) 2;
        bit <<= 1;
        step_count++;
    }
    while (bit != (unsigned long) 0) {
        if (remaining >= unit) {
            remaining -= unit;
            result |= bit;
        }
        unit /= (double) 2;
        bit >>= 1;
    }
    if (remaining >= (double) 1)
        return (unsigned long) 0;
    *valid_out = 1;
    return result;
}

unsigned long
__fixunssfdi(value)
double value;
{
    int valid = 0;
    unsigned long result = mantle_float_to_ulong_bounded(value, &valid);

    return valid ? result : (unsigned long) 0;
}
RUNTIME_EOF
      $BB cat > /tmp/bison23-hash.patch <<'PATCH_EOF'
--- lib/hash.c
+++ lib/hash.c
@@ -113,8 +113,13 @@
    1.0).  The growth threshold defaults to 0.8, and the growth factor
    defaults to 1.414, meaning that the table will have doubled its size
    every second time 80% of the buckets get used.  */
-#define DEFAULT_GROWTH_THRESHOLD 0.8
-#define DEFAULT_GROWTH_FACTOR 1.414
+#define MANTLE_HASH_RATIO_DENOMINATOR 1000
+#define MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR 800
+#define MANTLE_HASH_GROWTH_FACTOR_NUMERATOR 1414
+#define MANTLE_HASH_COMBINED_GROWTH_NUMERATOR 11312
+#define MANTLE_HASH_COMBINED_GROWTH_DENOMINATOR 10000
+#define DEFAULT_GROWTH_THRESHOLD ((float) 8 / (float) 10)
+#define DEFAULT_GROWTH_FACTOR ((float) 1414 / (float) 1000)
 
 /* If a deletion empties a bucket and causes the ratio of used buckets to
    table size to become smaller than the shrink threshold (a number between
@@ -122,8 +127,8 @@
    number greater than the shrink threshold but smaller than 1.0).  The shrink
    threshold and factor default to 0.0 and 1.0, meaning that the table never
    shrinks.  */
-#define DEFAULT_SHRINK_THRESHOLD 0.0
-#define DEFAULT_SHRINK_FACTOR 1.0
+#define DEFAULT_SHRINK_THRESHOLD ((float) 0)
+#define DEFAULT_SHRINK_FACTOR ((float) 1)
 
 /* Use this to initialize or reset a TUNING structure to
    some sensible values. */
@@ -485,26 +490,39 @@
 {
   const Hash_tuning *tuning = table->tuning;
 
-  /* Be a bit stricter than mathematics would require, so that
-     rounding errors in size calculations do not cause allocations to
-     fail to grow or shrink as they should.  The smallest allocation
-     is 11 (due to next_prime's algorithm), so an epsilon of 0.1
-     should be good enough.  */
-  float epsilon = 0.1f;
-
-  if (epsilon < tuning->growth_threshold
-      && tuning->growth_threshold < 1 - epsilon
-      && 1 + epsilon < tuning->growth_factor
-      && 0 <= tuning->shrink_threshold
-      && tuning->shrink_threshold + epsilon < tuning->shrink_factor
-      && tuning->shrink_factor <= 1
-      && tuning->shrink_threshold + epsilon < tuning->growth_threshold)
+  if (tuning == &default_tuning)
     return true;
 
   table->tuning = &default_tuning;
   return false;
 }
 
+static bool
+mantle_hash_ratio_ceil (size_t value, size_t numerator,
+                        size_t denominator, size_t *result)
+{
+  size_t adjustment;
+
+  if (denominator == 0 || numerator == 0 || result == NULL)
+    return false;
+  adjustment = denominator - 1;
+  if (value > (SIZE_MAX - adjustment) / numerator)
+    return false;
+  *result = (value * numerator + adjustment) / denominator;
+  return true;
+}
+
+static bool
+mantle_hash_growth_reached (size_t used, size_t buckets)
+{
+  if (used > SIZE_MAX / MANTLE_HASH_RATIO_DENOMINATOR)
+    return true;
+  if (buckets > SIZE_MAX / MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR)
+    return false;
+  return used * MANTLE_HASH_RATIO_DENOMINATOR
+         > buckets * MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR;
+}
+
 /* Allocate and return a new hash table, or NULL upon failure.  The initial
    number of buckets is automatically selected so as to _guarantee_ that you
    may insert at least CANDIDATE different user entries before any growth of
@@ -565,10 +583,10 @@
 
   if (!tuning->is_n_buckets)
     {
-      float new_candidate = candidate / tuning->growth_threshold;
-      if (SIZE_MAX <= new_candidate)
-	goto fail;
-      candidate = new_candidate;
+      if (! mantle_hash_ratio_ceil
+          (candidate, MANTLE_HASH_RATIO_DENOMINATOR,
+           MANTLE_HASH_GROWTH_THRESHOLD_NUMERATOR, &candidate))
+        goto fail;
     }
 
   if (xalloc_oversized (candidate, sizeof *table->bucket))
@@ -947,23 +965,26 @@
      entries:  if the hashing function is ill-conditioned, rehashing is not
      likely to improve it.  */
 
-  if (table->n_buckets_used
-      > table->tuning->growth_threshold * table->n_buckets)
+  if (mantle_hash_growth_reached (table->n_buckets_used,
+                                  table->n_buckets))
     {
       /* Check more fully, before starting real work.  If tuning arguments
 	 became invalid, the second check will rely on proper defaults.  */
       check_tuning (table);
-      if (table->n_buckets_used
-	  > table->tuning->growth_threshold * table->n_buckets)
+      if (mantle_hash_growth_reached (table->n_buckets_used,
+	                              table->n_buckets))
 	{
 	  const Hash_tuning *tuning = table->tuning;
-	  float candidate =
-	    (tuning->is_n_buckets
-	     ? (table->n_buckets * tuning->growth_factor)
-	     : (table->n_buckets * tuning->growth_factor
-		* tuning->growth_threshold));
+	  size_t candidate;
+	  size_t numerator = tuning->is_n_buckets
+	                     ? MANTLE_HASH_GROWTH_FACTOR_NUMERATOR
+	                     : MANTLE_HASH_COMBINED_GROWTH_NUMERATOR;
+	  size_t denominator = tuning->is_n_buckets
+	                       ? MANTLE_HASH_RATIO_DENOMINATOR
+	                       : MANTLE_HASH_COMBINED_GROWTH_DENOMINATOR;
 
-	  if (SIZE_MAX <= candidate)
+	  if (! mantle_hash_ratio_ceil (table->n_buckets, numerator,
+	                                denominator, &candidate))
 	    return NULL;
 
 	  /* If the rehash fails, arrange to return NULL.  */
PATCH_EOF
      $BB patch -p0 < /tmp/bison23-hash.patch || { echo "ERROR: Bison hash numeric patch did not apply" >&2; exit 1; }
      $BB grep -q 'mantle_hash_growth_reached' lib/hash.c || { echo "ERROR: bounded Bison hash growth core missing" >&2; exit 1; }
      if $BB grep -E '^#define DEFAULT_(GROWTH|SHRINK).*[0-9]+\.[0-9]+' lib/hash.c; then
        echo "ERROR: source-level Bison hash floating literal remained" >&2
        exit 1
      fi

      LIB_SOURCES="abitset argmatch basename bitset bitset_stats bitsetv bitsetv-print dup-safer ebitset error exitfail fd-safer fopen-safer get-errno getopt getopt1 hash lbitset obstack quote quotearg subpipe timevar vbitset xalloc-die xmalloc xstrndup"
      PROGRAM_SOURCES="LR0 assoc closure complain conflicts derives files getargs gram lalr location main muscle_tab nullable output parse-gram print print_graph reader reduce relation scan-gram scan-skel state symlist symtab tables uniqstr vcg"
      EXPECTED_LIB_SOURCE_COUNT=27
      EXPECTED_PROGRAM_SOURCE_COUNT=29
      LIBOBJS=""
      lib_source_count=0
      for source_name in $LIB_SOURCES; do
        echo "bison23-lib-source=$source_name.c" >&2
        "$TCC/bin/tcc" -c $CFLAGS "lib/$source_name.c" -o "lib/$source_name.o" || { echo "ERROR: failed compiling Bison library source $source_name.c" >&2; exit 1; }
        test -s "lib/$source_name.o" || { echo "ERROR: missing Bison library object $source_name.o" >&2; exit 1; }
        LIBOBJS="$LIBOBJS lib/$source_name.o"
        lib_source_count=$((lib_source_count + 1))
      done
      test "$lib_source_count" -eq "$EXPECTED_LIB_SOURCE_COUNT" || { echo "ERROR: incomplete Bison library source set" >&2; exit 1; }

      OBJS=""
      program_source_count=0
      for source_name in $PROGRAM_SOURCES; do
        echo "bison23-program-source=$source_name.c" >&2
        "$TCC/bin/tcc" -c $CFLAGS "src/$source_name.c" -o "src/$source_name.o" || { echo "ERROR: failed compiling Bison program source $source_name.c" >&2; exit 1; }
        test -s "src/$source_name.o" || { echo "ERROR: missing Bison program object $source_name.o" >&2; exit 1; }
        OBJS="$OBJS src/$source_name.o"
        program_source_count=$((program_source_count + 1))
      done
      test "$program_source_count" -eq "$EXPECTED_PROGRAM_SOURCE_COUNT" || { echo "ERROR: incomplete Bison program source set" >&2; exit 1; }
      "$TCC/bin/tcc" -nostdlib -static -I$MUSL/include -o bison \
        "$MUSL/lib/crt1.o" mantle_runtime_compat.c $OBJS $LIBOBJS \
        "$MUSL/lib/libc.a" || { echo "ERROR: failed linking Bison against native musl" >&2; exit 1; }
      test -s bison || { echo "ERROR: linked Bison missing" >&2; exit 1; }
      $BB chmod 755 bison

      $BB mkdir -p "$out/bin" "$out/share/bison"
      $BB cp bison "$out/bin/bison"
      $BB chmod 755 "$out/bin/bison"

      # Copy every release data file needed by Bison's M4 runtime.
      test -d data || { echo "ERROR: Bison runtime data directory missing" >&2; exit 1; }
      $BB cp -r data/. "$out/share/bison/" || { echo "ERROR: failed copying Bison runtime data" >&2; exit 1; }

      test -x "$out/bin/bison"
      echo "bison23-runtime-check=version" >&2
      "$out/bin/bison" --version >/tmp/bison23-version.out
      $BB grep -q 'bison' /tmp/bison23-version.out || { echo "ERROR: Bison version output missing" >&2; exit 1; }
      echo "bison23-runtime-check=positive-grammar" >&2
      $BB cat > /tmp/bison23-positive.y <<'GRAMMAR_EOF'
%token WORD
%%
input: WORD ;
%%
GRAMMAR_EOF
      M4="$M4/bin/m4" BISON_PKGDATADIR="$out/share/bison" \
        "$out/bin/bison" -d -o /tmp/bison23-positive.c /tmp/bison23-positive.y
      test -s /tmp/bison23-positive.c || { echo "ERROR: Bison emitted no parser source" >&2; exit 1; }
      test -s /tmp/bison23-positive.h || { echo "ERROR: Bison emitted no parser header" >&2; exit 1; }
      $BB cat > /tmp/bison23-negative.y <<'GRAMMAR_EOF'
%token WORD
%%
input WORD ;
%%
GRAMMAR_EOF
      $BB rm -f /tmp/bison23-negative.c
      if M4="$M4/bin/m4" BISON_PKGDATADIR="$out/share/bison" \
          "$out/bin/bison" -o /tmp/bison23-negative.c /tmp/bison23-negative.y \
          >/tmp/bison23-negative.out 2>&1; then
        echo "ERROR: Bison accepted malformed grammar" >&2
        exit 1
      fi
      test ! -e /tmp/bison23-negative.c || { echo "ERROR: Bison retained malformed parser output" >&2; exit 1; }
      test -s /tmp/bison23-negative.out || { echo "ERROR: malformed Bison grammar emitted no diagnostic" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, tcc, musl, m4, bison_src],
} | crunch.Derivation
