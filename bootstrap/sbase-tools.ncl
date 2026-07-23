# Build the bounded POSIX utility set needed by the native GCC ladder.
#
# These are real static sbase executables compiled from one pinned source tree.
# The output is a build-tool input only; it is not part of the normalized seed
# provider runtime contract.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let tcc = crunch.derivationFile "tcc-musl-selfhost.ncl" in
let tcc_native = crunch.derivationFile "tcc-musl-native.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-native.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let binutils = crunch.derivationFile "binutils-2.30-gas-source.ncl" in

let sbase_src = crunch.fetchGit {
  url = "git://git.suckless.org/sbase",
  rev = "c546c3a5724c81cee9a11d816a38ccdf17472129",
  hash = "sha256-B6NHmaKvp/PtQaUtOmLOt8mKe2+jklNA95h48ECv/Iw=",
  name = "sbase-c546c3a-src",
} in

let sbase_touch_src = crunch.fetchGit {
  # This upstream revision predates sbase touch's utimensat conversion. The
  # early reduced musl rejects that newer nanosecond path with EINVAL.
  url = "git://git.suckless.org/sbase",
  rev = "1cdb424ac63ebd7eb5eda1b8af53871d89e1ead9",
  hash = "sha256-GSQ3YCJh6oSXKzofiyPnsS47WhcYkoVsTl67HMQjEzg=",
  name = "sbase-1cdb424-touch-src",
} in

{
  name = "sbase-tools-c546c3a",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      COMPILER_TEARDOWN_SIGNAL_STATUS=139
      EXECUTABLE_MODE=755
      JOB_COUNT=1
      EXPR_REGEX_MATCH_LENGTH=3

      find_input() {
        for d in $NIX_STORE/*-"$1"; do
          if [ -d "$d" ]; then echo "$d"; return; fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      TCC=$(find_input tcc-0.9.27-musl-selfhost)
      TCC_NATIVE=$(find_input tcc-0.9.27-musl-native-runtime)
      MUSL=$(find_input musl-1.1.24-native-candidate)
      MAKE=$(find_input make-3.82-tcc)
      BINUTILS=$(find_input binutils-2.30-gas-source-v41)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input sbase-c546c3a-src)
      TOUCH_SRC=$(find_input sbase-1cdb424-touch-src)

      export PATH="$TCC/bin:$MAKE/bin:$BINUTILS/bin:$STAGE0/bin"
      export SHELL=/bin/sh

      WORK=/tmp/sbase-tools-build
      $BB mkdir -p "$WORK"
      $BB cp -r "$SRC" "$WORK/sbase"
      $BB chmod -R u+w "$WORK/sbase"
      cd "$WORK/sbase"
      test -s "$TOUCH_SRC/touch.c"
      $BB cp "$TOUCH_SRC/touch.c" touch.c
      $BB chmod u+rw touch.c

      # The reduced early musl omits the NSS/network closure reached by
      # getpwuid/getgrgid. Keep ls source-built and force its long-format owner
      # fields to the POSIX numeric representation instead of delegating lookup.
      $BB sed -i 's/if (!nflag && (pw = getpwuid(ent->uid)))/if (0)/; s/if (!nflag && (gr = getgrgid(ent->gid)))/if (0)/' ls.c
      if $BB grep -Eq 'getpwuid|getgrgid' ls.c; then
        echo "ERROR: numeric sbase ls retained an NSS lookup" >&2
        exit 1
      fi

      # This compiler generation can finish an output before its known teardown
      # fault. Accept that bounded status only when the requested output exists.
      CC_WRAPPER="$WORK/tcc-complete-output-wrapper"
      $BB cat > "$CC_WRAPPER" <<EOF
#!/bin/sh
set -u
REAL_TCC="$TCC/bin/tcc"
MUSL_LIB="$MUSL/lib"
TCC_RUNTIME="$TCC/lib/tcc/libtcc1.a"
WCTYPE_ADAPTER="$WORK/sbase/sbase-wctype-adapter.o"
WCTYPE_OBJECTS="libutf/isalnumrune.o libutf/isalpharune.o libutf/isblankrune.o libutf/iscntrlrune.o libutf/isdigitrune.o libutf/isgraphrune.o libutf/isprintrune.o libutf/ispunctrune.o libutf/isspacerune.o libutf/isxdigitrune.o libutf/lowerrune.o libutf/upperrune.o"
TEARDOWN_STATUS=$COMPILER_TEARDOWN_SIGNAL_STATUS
mode=link
output_file=
previous=
for argument in "\$@"; do
  if [ "\$previous" = -o ]; then output_file="\$argument"; fi
  if [ "\$argument" = -c ]; then mode=compile; fi
  previous="\$argument"
done
if [ "\$mode" = compile ]; then
  "\$REAL_TCC" "\$@"
else
  "\$REAL_TCC" -nostdlib -static "\$MUSL_LIB/crt1.o" \$WCTYPE_OBJECTS "\$WCTYPE_ADAPTER" \
    "\$@" "\$MUSL_LIB/libc.a" "\$TCC_RUNTIME"
fi
compiler_status=\$?
if [ "\$compiler_status" -eq 0 ]; then exit 0; fi
if [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ] && [ -n "\$output_file" ] && [ -s "\$output_file" ]; then
  echo "sbase-tools: accepted complete output before known compiler teardown fault: \$output_file" >&2
  exit 0
fi
if [ -n "\$output_file" ]; then /bin/busybox rm -f "\$output_file"; fi
exit "\$compiler_status"
EOF
      $BB chmod "$EXECUTABLE_MODE" "$CC_WRAPPER"

      # The early native musl deliberately omits four chartable-backed wide
      # character functions. Implement them over sbase's source Unicode tables
      # so musl regex and expr retain real category and lowercase semantics.
      $BB cat > sbase-wctype-adapter.c <<'EOF'
#include <fcntl.h>
#include <string.h>
#include <sys/syscall.h>
#include <time.h>
#include <unistd.h>
#include <utime.h>
#include <wctype.h>
#include "utf.h"

enum sbase_wctype_descriptor {
    sbase_wctype_invalid,
    sbase_wctype_alnum,
    sbase_wctype_alpha,
    sbase_wctype_blank,
    sbase_wctype_cntrl,
    sbase_wctype_digit,
    sbase_wctype_graph,
    sbase_wctype_lower,
    sbase_wctype_print,
    sbase_wctype_punct,
    sbase_wctype_space,
    sbase_wctype_upper,
    sbase_wctype_xdigit
};

enum sbase_wctrans_descriptor {
    sbase_wctrans_invalid,
    sbase_wctrans_lower,
    sbase_wctrans_upper
};

int linkat(int old_directory, const char *old_path, int new_directory,
           const char *new_path, int flags)
{
    return (int)syscall(SYS_linkat, old_directory, old_path,
                        new_directory, new_path, flags);
}

int utimensat(int directory, const char *path,
              const struct timespec times[2], int flags)
{
    return (int)syscall(SYS_utimensat, directory, path, times, flags);
}

int utime(const char *path, const struct utimbuf *times)
{
    struct timespec translated[2];
    if (!times) return utimensat(AT_FDCWD, path, 0, 0);
    translated[0].tv_sec = times->actime;
    translated[0].tv_nsec = 0;
    translated[1].tv_sec = times->modtime;
    translated[1].tv_nsec = 0;
    return utimensat(AT_FDCWD, path, translated, 0);
}

int iswalnum(wint_t character) { return isalnumrune((Rune)character); }
int iswalpha(wint_t character) { return isalpharune((Rune)character); }
int iswcntrl(wint_t character) { return iscntrlrune((Rune)character); }
wint_t towlower(wint_t character) { return (wint_t)tolowerrune((Rune)character); }
wint_t towupper(wint_t character) { return (wint_t)toupperrune((Rune)character); }
wint_t towctrans(wint_t character, wctrans_t descriptor)
{
    if (descriptor == (wctrans_t)sbase_wctrans_lower) return towlower(character);
    if (descriptor == (wctrans_t)sbase_wctrans_upper) return towupper(character);
    return character;
}

wctype_t wctype(const char *name)
{
    static const char *const names[] = {
        "", "alnum", "alpha", "blank", "cntrl", "digit", "graph",
        "lower", "print", "punct", "space", "upper", "xdigit"
    };
    unsigned int index;
    for (index = sbase_wctype_alnum; index <= sbase_wctype_xdigit; index++) {
        if (strcmp(name, names[index]) == 0) return (wctype_t)index;
    }
    return (wctype_t)sbase_wctype_invalid;
}

int iswctype(wint_t character, wctype_t descriptor)
{
    switch ((enum sbase_wctype_descriptor)descriptor) {
    case sbase_wctype_alnum: return isalnumrune((Rune)character);
    case sbase_wctype_alpha: return isalpharune((Rune)character);
    case sbase_wctype_blank: return isblankrune((Rune)character);
    case sbase_wctype_cntrl: return iscntrlrune((Rune)character);
    case sbase_wctype_digit: return isdigitrune((Rune)character);
    case sbase_wctype_graph: return isgraphrune((Rune)character);
    case sbase_wctype_lower: return islowerrune((Rune)character);
    case sbase_wctype_print: return isprintrune((Rune)character);
    case sbase_wctype_punct: return ispunctrune((Rune)character);
    case sbase_wctype_space: return isspacerune((Rune)character);
    case sbase_wctype_upper: return isupperrune((Rune)character);
    case sbase_wctype_xdigit: return isxdigitrune((Rune)character);
    case sbase_wctype_invalid: return 0;
    }
    return 0;
}
EOF
      set +e
      "$TCC_NATIVE/bin/tcc" -c -I. -I"$MUSL/include" sbase-wctype-adapter.c -o sbase-wctype-adapter.o
      adapter_compile_status=$?
      set -e
      if ! { [ "$adapter_compile_status" -eq 0 ] || \
          [ "$adapter_compile_status" -eq "$COMPILER_TEARDOWN_SIGNAL_STATUS" ]; } || \
          [ ! -s sbase-wctype-adapter.o ]; then
        $BB rm -f sbase-wctype-adapter.o
        echo "ERROR: native TinyCC rejected the sbase wctype adapter: $adapter_compile_status" >&2
        exit 1
      fi
      $BB chmod u+rw sbase-wctype-adapter.o

      TOOLS="basename cat chmod cmp cp cut dirname echo expr false head ln ls mkdir mv od printf pwd rm rmdir sort tail test touch tr true uname uniq wc"
      test -n "$TOOLS"

      "$MAKE/bin/make" -j "$JOB_COUNT" \
        CC="$CC_WRAPPER" \
        AR="$TCC/bin/tcc -ar" \
        ARFLAGS=cr \
        RANLIB="$BINUTILS/bin/ranlib" \
        CFLAGS="-I$MUSL/include" \
        LDFLAGS=-static \
        $TOOLS

      $BB mkdir -p "$out/bin"
      for tool in $TOOLS; do
        test -s "$tool" || { echo "ERROR: sbase tool missing: $tool" >&2; exit 1; }
        $BB chmod "$EXECUTABLE_MODE" "$tool"
        test -x "$tool" || { echo "ERROR: sbase tool is not executable: $tool" >&2; exit 1; }
        if $BB head -c 2 "$tool" | $BB grep -q '^#!'; then
          echo "ERROR: sbase output delegates through a script: $tool" >&2
          exit 1
        fi
        $BB cp "$tool" "$out/bin/$tool"
        $BB chmod "$EXECUTABLE_MODE" "$out/bin/$tool"
      done

      SMOKE="$WORK/smoke"
      "$out/bin/mkdir" "$SMOKE"
      "$out/bin/printf" 'beta\nalpha\nalpha\n' > "$SMOKE/input"
      "$out/bin/ls" "$SMOKE/input" | $BB grep -q '/input$'
      "$out/bin/ls" -ln "$SMOKE/input" | $BB grep -q '/input$'
      test "$("$out/bin/basename" /one/two)" = two
      test "$("$out/bin/dirname" /one/two)" = /one
      test "$("$out/bin/expr" 1 + 1)" = 2
      test "$("$out/bin/expr" abc : 'a.*')" = "$EXPR_REGEX_MATCH_LENGTH"
      test "$("$out/bin/head" -n 1 "$SMOKE/input")" = beta
      test "$("$out/bin/tail" -n 1 "$SMOKE/input")" = alpha
      test "$("$out/bin/cut" -c 1 "$SMOKE/input" | "$out/bin/head" -n 1)" = b
      "$out/bin/sort" "$SMOKE/input" | "$out/bin/uniq" > "$SMOKE/sorted"
      test "$("$out/bin/wc" -l < "$SMOKE/sorted" | "$out/bin/tr" -d ' ')" = 2
      "$out/bin/cp" -p "$SMOKE/input" "$SMOKE/copied"
      "$out/bin/mv" "$SMOKE/copied" "$SMOKE/moved"
      "$out/bin/ln" "$SMOKE/moved" "$SMOKE/linked"
      "$out/bin/chmod" 600 "$SMOKE/linked"
      "$out/bin/touch" "$SMOKE/touched"
      test -e "$SMOKE/touched"
      "$out/bin/rm" "$SMOKE/linked" "$SMOKE/moved" "$SMOKE/touched"
      test ! -e "$SMOKE/linked"

      if "$out/bin/expr" 1 + > "$SMOKE/rejected-output" 2> "$SMOKE/rejected-error"; then
        echo "ERROR: sbase expr accepted malformed input" >&2
        exit 1
      fi
      test ! -s "$SMOKE/rejected-output"
      test -s "$SMOKE/rejected-error"
      if "$out/bin/ls" "$SMOKE/missing" > "$SMOKE/ls-rejected-output" 2> "$SMOKE/ls-rejected-error"; then
        echo "ERROR: sbase ls accepted a missing path" >&2
        exit 1
      fi
      test ! -s "$SMOKE/ls-rejected-output"
      test -s "$SMOKE/ls-rejected-error"

      echo "sbase source-built utility runtime and rejection checks passed for $TARGET" >&2
    "%
  ],
  inputs = [stage0, tcc, tcc_native, musl, make, binutils, sbase_src, sbase_touch_src],
} | crunch.Derivation
