# Run the shared StageX libc behavior contract against the Picolibc diagnostic.
#
# The positive program is the exact StageX native-musl smoke contract from
# bootstrap/musl-1.1.24-native.ncl. The negative case is the shared malformed
# source. This derivation is diagnostic-only evidence for the
# `evaluate-picolibc-stagex-runtime` comparison; it changes no provider or
# parity authority.

let crunch = import "../lib/lib.ncl" in
let diag = import "picolibc-1.8.12-diagnostic.ncl" in
let tc = import "picolibc-diagnostic-tools.ncl" in

{
  picolibc-behavior = {
    name = "picolibc-1.8.12-x86_64-linux-behavior",
    builder = "/bin/sh",
    system = 'x86_64-linux,
    args = [
      "-c",
      m%"
        set -eu
        BB=/bin/busybox
        PICO=$(echo $NIX_STORE/*-picolibc-1.8.12-x86_64-linux-diagnostic)

        $BB mkdir -p tools shims
        $BB ln -sf %{tc.gcc}/bin/gcc shims/x86_64-linux-gnu-gcc
        for tool in as ld ar ranlib; do
          $BB ln -sf %{tc.binutils}/bin/$tool tools/$tool
        done
        $BB ln -sf /bin/sh tools/sh
        export PATH="$PWD/shims:$PWD/tools:%{tc.coreutils}/bin"
        export HOME=$PWD

        $BB cat > smoke.c <<'SMOKE_EOF'
#include <errno.h>
#include <locale.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>

enum {
    allocation_bytes = 32,
    grown_bytes = 64,
    value_count = 4,
    second_sort_offset = 2,
    allocation_fill_byte = 0x5a,
    output_bytes = 32,
    formatted_number = 124,
    formatted_characters = 8,
    truncated_capacity = 4,
    truncated_characters = 9,
    failure_locale = 10,
    failure_allocation = 11,
    failure_reallocation = 12,
    failure_oversized_allocation = 13,
    failure_stat = 14,
    failure_time = 15,
    failure_invalid_signal = 16,
    failure_signal_install = 17,
    failure_signal_delivery = 18,
    failure_sort = 19,
    failure_format_length = 20,
    failure_format_content = 21,
    failure_format_truncation = 22,
    failure_stdio_write = 23,
    failure_stdio_flush = 24,
    failure_strtod_zero = 25,
    failure_strtod_one = 26
};
static volatile sig_atomic_t signal_seen;
static void mark_signal(int signal_number) { signal_seen = signal_number; }
static int compare_ints(const void *left, const void *right)
{
    int left_value = *(const int *)left;
    int right_value = *(const int *)right;
    return (left_value > right_value) - (left_value < right_value);
}
int main(void)
{
    char *memory;
    char output[output_bytes];
    int values[value_count] = { value_count, 1, value_count - 1, value_count - second_sort_offset };
    struct stat status;
    struct timespec timestamp;
    struct sigaction action;
    char *number_end;
    double parsed_number;
    if (!setlocale(LC_ALL, "C")) return failure_locale;
    parsed_number = strtod("0", &number_end);
    if ((int)parsed_number != 0 || *number_end != '\0') return failure_strtod_zero;
    parsed_number = strtod("1", &number_end);
    if ((int)parsed_number != 1 || *number_end != '\0') return failure_strtod_one;
    memory = malloc(allocation_bytes);
    if (!memory) return failure_allocation;
    memset(memory, allocation_fill_byte, allocation_bytes);
    memory = realloc(memory, grown_bytes);
    if (!memory || memory[allocation_bytes - 1] != allocation_fill_byte) return failure_reallocation;
    free(memory);
    if (malloc(SIZE_MAX) != NULL) return failure_oversized_allocation;
    if (stat("/dev/null", &status) != 0) return failure_stat;
    if (clock_gettime(CLOCK_REALTIME, &timestamp) != 0) return failure_time;
    memset(&action, 0, sizeof(action));
    action.sa_handler = mark_signal;
    sigemptyset(&action.sa_mask);
    errno = 0;
    if (sigaction(0, &action, NULL) != -1 || errno != EINVAL) return failure_invalid_signal;
    if (sigaction(SIGUSR1, &action, NULL) != 0) return failure_signal_install;
    if (raise(SIGUSR1) != 0 || signal_seen != SIGUSR1) return failure_signal_delivery;
    qsort(values, value_count, sizeof(values[0]), compare_ints);
    if (values[0] != 1 || values[value_count - 1] != value_count) return failure_sort;
    if (snprintf(output, sizeof(output), "\045s-\045d", "musl", formatted_number) != formatted_characters) return failure_format_length;
    if (strcmp(output, "musl-124") != 0) return failure_format_content;
    if (snprintf(output, truncated_capacity, "\045s", "truncated") != truncated_characters) return failure_format_truncation;
    if (fputs("native-musl-smoke-ok\n", stdout) < 0) return failure_stdio_write;
    if (fflush(stdout) != 0) return failure_stdio_flush;
    return 0;
}
SMOKE_EOF

        printf 'int broken( {\n' > malformed.c

        LIBGCC=$(x86_64-linux-gnu-gcc -print-libgcc-file-name)
        POSITIVE_PASS=0
        POSITIVE_FAIL=""
        if x86_64-linux-gnu-gcc -static -nostdlib -nostartfiles -fno-pic -march=core2 \
            -I$PICO/include -L$PICO/lib -T $PICO/lib/picolibc_linux.ld \
            $PICO/lib/crt0-linux.o smoke.c \
            -Wl,--start-group -lc -llinux -Wl,--end-group "$LIBGCC" \
            -o picolibc-smoke 2> positive-compile.stderr; then
          SMOKE_OUTPUT=$(./picolibc-smoke) && SMOKE_STATUS=$? || SMOKE_STATUS=$?
          if [ "$SMOKE_STATUS" -eq 0 ] && [ "$SMOKE_OUTPUT" = "native-musl-smoke-ok" ]; then
            POSITIVE_PASS=1
          else
            POSITIVE_FAIL="runtime mismatch: status=$SMOKE_STATUS output=$SMOKE_OUTPUT"
          fi
        else
          POSITIVE_FAIL="compile failed: $($BB tail -1 positive-compile.stderr)"
        fi

        NEGATIVE_PASS=0
        NEGATIVE_FAIL=""
        if x86_64-linux-gnu-gcc -static -nostdlib -nostartfiles -fno-pic \
            -I$PICO/include -c malformed.c -o malformed.o 2> negative-compile.stderr; then
          NEGATIVE_FAIL="malformed source compiled unexpectedly"
        else
          NEGATIVE_PASS=1
        fi

        $BB mkdir -p $out/share/picolibc-behavior
        $BB cp smoke.c malformed.c positive-compile.stderr negative-compile.stderr \
          $out/share/picolibc-behavior/
        if [ -f picolibc-smoke ]; then
          $BB cp picolibc-smoke $out/share/picolibc-behavior/
        fi
        {
          printf '{\n'
          printf '  \042positive_passed\042: %s,\n' "$POSITIVE_PASS"
          printf '  \042positive_total\042: 1,\n'
          printf '  \042negative_passed\042: %s,\n' "$NEGATIVE_PASS"
          printf '  \042negative_total\042: 1,\n'
          printf '  \042failures\042: ['
          SEP=""
          if [ -n "$POSITIVE_FAIL" ]; then printf '\042%s\042' "$POSITIVE_FAIL"; SEP=","; fi
          if [ -n "$NEGATIVE_FAIL" ]; then printf '%s\042%s\042' "$SEP" "$NEGATIVE_FAIL"; fi
          printf ']\n}\n'
        } > $out/share/picolibc-behavior/behavior.json

        # Behavior failures are comparison evidence, not build failures. The
        # matrix always completes and records its facts; the comparison core
        # classifies the outcome.
        if [ "$POSITIVE_PASS" -ne 1 ] || [ "$NEGATIVE_PASS" -ne 1 ]; then
          echo "NOTE: picolibc behavior contract failure recorded as evidence" >&2
          $BB cat $out/share/picolibc-behavior/behavior.json >&2
        fi
      "%,
    ],
    outputs = ["out"],
    env = {},
    inputs = [
      diag.picolibc-diagnostic,
      tc.gcc,
      tc.binutils,
      tc.coreutils,
    ],
    addressing_mode = 'input-addressed,
    sandbox = 'native,
  } | crunch.Derivation,
}
