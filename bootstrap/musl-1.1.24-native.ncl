# Upstream-faithful musl 1.1.24 candidate built by the source-built TinyCC handoff.
#
# This is intentionally isolated from the selected bootstrap provider until its
# compiler, libc, and downstream runtime admission rails pass.  It follows the
# live-bootstrap pass2 recipe instead of the compatibility-stub musl path.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let tcc = crunch.derivationFile "tcc-musl-selfhost.ncl" in
let tcc26 = crunch.derivationFile "tcc-musl-v2.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let musl_src = crunch.derivationFile "musl-1.1.24-src.ncl" in

{
  name = "musl-1.1.24-native-candidate",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then
            echo "$candidate"
            return
          fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      TCC=$(find_input tcc-0.9.27-musl-selfhost)
      TCC26=$(find_input tcc-0.9.27-musl-v2)
      echo "native-musl-tcc=$TCC" >&2
      echo "native-musl-linker=$TCC26" >&2
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      MUSL_SRC=$(find_input musl-1.1.24-src)

      WORK=/tmp/musl-native
      $BB mkdir -p "$WORK/bin"
      for applet in awk basename cat chmod cp dirname echo expr find grep ln mkdir mv printf pwd rm rmdir sed sh sort test touch tr true uname; do
        $BB ln -sf "$BB" "$WORK/bin/$applet"
      done
      export PATH="$WORK/bin:$TCC/bin:$MAKE/bin:$STAGE0/bin:$PATH"
      $BB cat > "$WORK/bin/tcc-clean" <<EOF
#!/bin/sh
exec /bin/busybox env -i \
  PATH="$WORK/bin:$TCC/bin:$MAKE/bin:$STAGE0/bin" \
  HOME=/tmp TMPDIR=/tmp \
  "$TCC/bin/tcc" "\$@"
EOF
      $BB chmod 755 "$WORK/bin/tcc-clean"
      TCC_CLEAN="$WORK/bin/tcc-clean"
      $BB cat > "$WORK/bin/tcc-ar" <<EOF
#!/bin/sh
if [ "\$1" = rc ]; then
  shift
  archive="\$1"
  shift
  if [ "\$#" -eq 0 ]; then
    /bin/busybox printf '!<arch>\n' > "\$archive"
    exit 0
  fi
  scratch="/tmp/mantle-tcc-ar-\$\$"
  /bin/busybox rm -rf "\$scratch"
  /bin/busybox mkdir -p "\$scratch"
  short_inputs=""
  index=0
  for object in "\$@"; do
    short="\$scratch/o\$index.o"
    /bin/busybox cp "\$object" "\$short"
    short_inputs="\$short_inputs \$short"
    index=\$((index + 1))
  done
  "$TCC_CLEAN" -ar rcs "\$archive" \$short_inputs
  result=\$?
  /bin/busybox rm -rf "\$scratch"
  exit "\$result"
fi
exec "$TCC_CLEAN" -ar "\$@"
EOF
      $BB chmod 755 "$WORK/bin/tcc-ar"
      TCC_AR="$WORK/bin/tcc-ar"

      $BB cp -r "$MUSL_SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"
      if [ ! -f configure ]; then
        for configure_path in */configure; do
          if [ -f "$configure_path" ]; then
            cd "${configure_path%/configure}"
            break
          fi
        done
      fi
      test -f configure || { echo "ERROR: musl configure script missing" >&2; exit 1; }
      $BB cat > include/stdarg.h <<'EOF'
#ifndef _STDARG_H
#define _STDARG_H
#define __NEED_va_list
#include <bits/alltypes.h>
void __va_start(void *, void *);
void *__va_arg(void *, int, int, int);
#define va_start(arguments, last) __va_start((arguments), __builtin_frame_address(0))
#define va_arg(arguments, type) \
    (*(type *)__va_arg((arguments), __builtin_va_arg_types(type), sizeof(type), __alignof__(type)))
#define va_copy(destination, source) (*(destination) = *(source))
#define va_end(arguments)
#endif
EOF
      $BB cat > src/internal/va_list.c <<'EOF'
#if defined __x86_64__
#include <stddef.h>
#include <string.h>
#include <stdlib.h>
enum va_arg_type { va_general_register, va_float_register, va_stack };
enum {
    va_stack_slot_bytes = 8,
    va_general_register_bytes = 48,
    va_float_register_slot_bytes = 16,
    va_float_register_bytes = 176,
    va_frame_metadata_bytes = 16
};
typedef struct {
    unsigned int gp_offset;
    unsigned int fp_offset;
    union { unsigned int overflow_offset; char *overflow_arg_area; };
    char *reg_save_area;
} bootstrap_va_list;
void __va_start(bootstrap_va_list *arguments, void *frame_pointer)
{
    memset(arguments, 0, sizeof(*arguments));
    *arguments = *(bootstrap_va_list *)((char *)frame_pointer - va_frame_metadata_bytes);
    arguments->overflow_arg_area = (char *)frame_pointer + arguments->overflow_offset;
    arguments->reg_save_area = (char *)frame_pointer - va_float_register_bytes - va_frame_metadata_bytes;
}
void *__va_arg(bootstrap_va_list *arguments, enum va_arg_type argument_type,
               int argument_bytes, int argument_alignment)
{
    argument_bytes = (argument_bytes + va_stack_slot_bytes - 1) & -va_stack_slot_bytes;
    argument_alignment = (argument_alignment + va_stack_slot_bytes - 1) & -va_stack_slot_bytes;
    if (argument_type == va_general_register &&
        arguments->gp_offset + argument_bytes <= va_general_register_bytes) {
        arguments->gp_offset += argument_bytes;
        return arguments->reg_save_area + arguments->gp_offset - argument_bytes;
    }
    if (argument_type == va_float_register &&
        arguments->fp_offset < va_float_register_bytes) {
        arguments->fp_offset += va_float_register_slot_bytes;
        return arguments->reg_save_area + arguments->fp_offset - va_float_register_slot_bytes;
    }
    if (argument_type != va_general_register &&
        argument_type != va_float_register && argument_type != va_stack) abort();
    arguments->overflow_arg_area += argument_bytes;
    arguments->overflow_arg_area = (char *)((long long)(arguments->overflow_arg_area + argument_alignment - 1) & -argument_alignment);
    return arguments->overflow_arg_area - argument_bytes;
}
#endif
EOF

      # live-bootstrap cannot regenerate these chartable-backed sources before
      # the later toolchain exists.  Remove only that bounded generated-header
      # surface and TinyCC's unsupported complex-number implementation.
      $BB sed -i '/int iswalnum(wint_t);/d' include/wchar.h include/wctype.h
      $BB sed -i '/int iswalpha(wint_t);/d' include/wchar.h include/wctype.h
      $BB sed -i '/int iswcntrl(wint_t);/d' include/wchar.h include/wctype.h
      $BB sed -i '/int iswctype(wint_t, wctype_t);/d' include/wchar.h include/wctype.h
      $BB sed -i '/wint_t towctrans(wint_t, wctrans_t);/d' include/wctype.h
      $BB sed -i '/wint_t towlower(wint_t);/d' include/wchar.h include/wctype.h
      $BB sed -i '/wint_t towupper(wint_t);/d' include/wchar.h include/wctype.h
      $BB rm -f src/ctype/iswalpha.c src/ctype/iswalnum.c src/ctype/iswctype.c src/ctype/towctrans.c
      $BB rm -f include/iconv.h src/locale/iconv.c src/locale/iconv_close.c
      # Native-language message catalogs are outside this C-locale bootstrap
      # pass and are restored by the later GCC-built libc.
      $BB rm -f src/locale/bind_textdomain_codeset.c src/locale/catclose.c \
        src/locale/catgets.c src/locale/catopen.c src/locale/dcngettext.c \
        src/locale/strfmon.c src/locale/textdomain.c
      # The early bootstrap libc is a static compiler-support surface.  Omit
      # AIO, complex-number, floating-point-environment, and dynamic-loader
      # sources that the current TinyCC handoff cannot compile; retain the
      # upstream long-double helpers required by stdio formatting.
      BOOTSTRAP_MATH="$WORK/bootstrap-math"
      $BB mkdir -p "$BOOTSTRAP_MATH"
      for math_source in __fpclassifyl.c __signbitl.c frexp.c frexpl.c ldexp.c; do
        $BB cp "src/math/$math_source" "$BOOTSTRAP_MATH/$math_source"
      done
      $BB rm -rf src/aio src/complex src/fenv src/ldso src/legacy src/linux src/math src/network src/sched
      $BB mkdir -p src/math
      for math_source in "$BOOTSTRAP_MATH"/*.c; do
        $BB cp "$math_source" "src/math/$($BB basename "$math_source")"
      done
      # Diagnostic bridge only: the current TinyCC miscompiles floatscan's
      # signed-zero multiplication in the large decimal scanner, making
      # strtod("0") a NaN.  Return numeric zero directly; signed-zero fidelity
      # must be restored by the GCC-built libc before admission.
      $BB sed -i 's/if (!x\[0\]) return sign \* 0\.0;/if (!x[0]) return 0;/' src/internal/floatscan.c
      $BB grep -q 'if (!x\[0\]) return 0;' src/internal/floatscan.c

      # Diagnostic bridge only: the current TinyCC front end emits alloca as
      # an external call rather than caller-frame allocation.  Heap allocation
      # keeps the single-process GCC frontier executable but leaks by design;
      # the GCC-built musl pass must remove this file before admission.
      $BB cat > src/malloc/bootstrap_alloca.c <<'EOF'
#include <stddef.h>
#include <stdlib.h>

void *alloca(size_t size)
{
    return malloc(size);
}
EOF
      # Diagnostic bridge only: both source-built TinyCC front ends fail on
      # fdlibm's file-scope floating tables.  Supply bounded scalar algorithms
      # for gawk's builtin symbol closure so the chain can reach GCC.  The
      # GCC-built musl pass must restore upstream math before seed admission.
      $BB cat > src/math/bootstrap_gawk_math.c <<'EOF'
#include <float.h>
#include <math.h>

static double bootstrap_absolute(double value)
{
    return value < 0.0 ? -value : value;
}

double fabs(double value)
{
    return bootstrap_absolute(value);
}

double floor(double value)
{
    double integer_limit = 9223372036854775807.0;
    long long integer_value;
    if (value >= integer_limit || value <= -integer_limit) return value;
    integer_value = (long long)value;
    if ((double)integer_value > value) integer_value--;
    return (double)integer_value;
}

double ceil(double value)
{
    double integer_limit = 9223372036854775807.0;
    long long integer_value;
    if (value >= integer_limit || value <= -integer_limit) return value;
    integer_value = (long long)value;
    if ((double)integer_value < value) integer_value++;
    return (double)integer_value;
}

double modf(double value, double *integer_part)
{
    *integer_part = value < 0.0 ? ceil(value) : floor(value);
    return value - *integer_part;
}

static double bootstrap_scale_double(double value, int exponent)
{
    enum { exponent_limit = 4096 };
    int step_count = 0;
    while (exponent > 0 && step_count < exponent_limit) {
        value *= 2.0;
        exponent--;
        step_count++;
    }
    while (exponent < 0 && step_count < exponent_limit) {
        value *= 0.5;
        exponent++;
        step_count++;
    }
    return value;
}

double scalbn(double value, int exponent)
{
    return bootstrap_scale_double(value, exponent);
}

long double scalbnl(long double value, int exponent)
{
    enum { exponent_limit = 16384 };
    int step_count = 0;
    while (exponent > 0 && step_count < exponent_limit) {
        value *= 2.0L;
        exponent--;
        step_count++;
    }
    while (exponent < 0 && step_count < exponent_limit) {
        value *= 0.5L;
        exponent++;
        step_count++;
    }
    return value;
}

double sqrt(double value)
{
    enum { iteration_count = 32 };
    int iteration;
    double estimate;
    if (value < 0.0) return 0.0;
    if (value == 0.0) return 0.0;
    estimate = value > 1.0 ? value : 1.0;
    for (iteration = 0; iteration < iteration_count; iteration++) {
        estimate = 0.5 * (estimate + value / estimate);
    }
    return estimate;
}

static double bootstrap_reduce_angle(double value)
{
    double pi = 3.14159265358979323846;
    double full_turn = 2.0 * pi;
    double turns = floor(value / full_turn);
    value -= turns * full_turn;
    if (value > pi) value -= full_turn;
    if (value < -pi) value += full_turn;
    return value;
}

double sin(double value)
{
    enum { series_terms = 12 };
    int term_index;
    double angle = bootstrap_reduce_angle(value);
    double term = angle;
    double result = term;
    for (term_index = 1; term_index < series_terms; term_index++) {
        double left_factor = 2.0 * term_index;
        double right_factor = left_factor + 1.0;
        term *= -angle * angle / (left_factor * right_factor);
        result += term;
    }
    return result;
}

double cos(double value)
{
    enum { series_terms = 12 };
    int term_index;
    double angle = bootstrap_reduce_angle(value);
    double term = 1.0;
    double result = term;
    for (term_index = 1; term_index < series_terms; term_index++) {
        double left_factor = 2.0 * term_index - 1.0;
        double right_factor = left_factor + 1.0;
        term *= -angle * angle / (left_factor * right_factor);
        result += term;
    }
    return result;
}

double exp(double value)
{
    enum { series_terms = 24, scale_limit = 1024 };
    int term_index;
    int scale_exponent;
    double logarithm_of_two = 0.69314718055994530942;
    double maximum_input = 709.0;
    double minimum_input = -745.0;
    double reduced;
    double term = 1.0;
    double result = 1.0;
    if (value > maximum_input) return DBL_MAX * DBL_MAX;
    if (value < minimum_input) return 0.0;
    scale_exponent = (int)(value / logarithm_of_two);
    if (scale_exponent > scale_limit) scale_exponent = scale_limit;
    if (scale_exponent < -scale_limit) scale_exponent = -scale_limit;
    reduced = value - scale_exponent * logarithm_of_two;
    for (term_index = 1; term_index <= series_terms; term_index++) {
        term *= reduced / term_index;
        result += term;
    }
    return bootstrap_scale_double(result, scale_exponent);
}

double log(double value)
{
    enum { normalization_limit = 2048, series_terms = 32 };
    int exponent = 0;
    int term_index;
    double logarithm_of_two = 0.69314718055994530942;
    double ratio;
    double ratio_power;
    double result = 0.0;
    if (value <= 0.0) return -DBL_MAX;
    while (value >= 2.0 && exponent < normalization_limit) {
        value *= 0.5;
        exponent++;
    }
    while (value < 1.0 && exponent > -normalization_limit) {
        value *= 2.0;
        exponent--;
    }
    ratio = (value - 1.0) / (value + 1.0);
    ratio_power = ratio;
    for (term_index = 1; term_index <= series_terms; term_index++) {
        int denominator = 2 * term_index - 1;
        result += ratio_power / denominator;
        ratio_power *= ratio * ratio;
    }
    return 2.0 * result + exponent * logarithm_of_two;
}

double pow(double base, double exponent)
{
    double integer_exponent;
    double magnitude;
    int negative_result = 0;
    if (base == 0.0) return exponent > 0.0 ? 0.0 : DBL_MAX;
    if (base < 0.0) {
        integer_exponent = floor(exponent);
        if (integer_exponent != exponent) return 0.0;
        negative_result = ((long long)integer_exponent & 1LL) != 0;
        base = -base;
    }
    magnitude = exp(exponent * log(base));
    return negative_result ? -magnitude : magnitude;
}

static double bootstrap_atan(double value)
{
    enum { series_terms = 48 };
    int term_index;
    int negative = value < 0.0;
    int reciprocal = 0;
    double pi_over_two = 1.57079632679489661923;
    double term;
    double result;
    if (negative) value = -value;
    if (value > 1.0) {
        reciprocal = 1;
        value = 1.0 / value;
    }
    term = value;
    result = value;
    for (term_index = 1; term_index < series_terms; term_index++) {
        term *= -value * value;
        result += term / (2 * term_index + 1);
    }
    if (reciprocal) result = pi_over_two - result;
    return negative ? -result : result;
}

double atan2(double y_value, double x_value)
{
    double pi = 3.14159265358979323846;
    double pi_over_two = 0.5 * pi;
    if (x_value > 0.0) return bootstrap_atan(y_value / x_value);
    if (x_value < 0.0 && y_value >= 0.0) return bootstrap_atan(y_value / x_value) + pi;
    if (x_value < 0.0) return bootstrap_atan(y_value / x_value) - pi;
    if (y_value > 0.0) return pi_over_two;
    if (y_value < 0.0) return -pi_over_two;
    return 0.0;
}

long double copysignl(long double magnitude, long double sign_source)
{
    if (magnitude < 0.0L) magnitude = -magnitude;
    return sign_source < 0.0L ? -magnitude : magnitude;
}

long double fmodl(long double numerator, long double denominator)
{
    long long quotient;
    if (denominator == 0.0L) return 0.0L;
    quotient = (long long)(numerator / denominator);
    return numerator - quotient * denominator;
}
EOF
      # This static-only bootstrap pass does not admit PIE startup artifacts;
      # the later GCC-built musl stage owns shared/PIE runtime admission.
      $BB rm -f crt/Scrt1.c crt/rcrt1.c
      $BB cat > crt/crt1.c <<'EOF'
extern int main(int, char **, char **);
extern int __libc_start_main(int (*)(int, char **, char **), int, char **,
                             void (*)(void), void (*)(void), void (*)(void));
extern void _init(void);
extern void _fini(void);
__asm__(
".global _start\n"
".weak _init\n"
".weak _fini\n"
"_start:\n"
"    xor %rbp,%rbp\n"
"    mov (%rsp),%rsi\n"
"    lea STACK_ARGUMENT_BYTES(%rsp),%rdx\n"
"    lea main(%rip),%rdi\n"
"    xor %rcx,%rcx\n"
"    xor %r8,%r8\n"
"    xor %r9,%r9\n"
"    andq $STACK_ALIGNMENT_MASK,%rsp\n"
"    call __libc_start_main\n"
"    mov %rax,%rdi\n"
"    mov $SYS_EXIT,%rax\n"
"    syscall\n");
EOF
      # TinyCC 0.9.27 does not parse the C99 `static` minimum-bound array
      # qualifier.  Dropping the optimizer-only qualifier preserves the ABI.
      $BB sed -i 's/char __buf\[static 15+3\*sizeof(int)\]/char __buf[15+3*sizeof(int)]/' src/internal/syscall.h
      # Move x86_64 TLS lookup out of unsupported inline assembly while
      # preserving the exact fs:0 pthread-pointer ABI.
      $BB cat > arch/x86_64/pthread_arch.h <<'EOF'
hidden struct pthread *__pthread_self(void);
#define TP_ADJ(p) (p)
#define MC_PC gregs[REG_RIP]
EOF
      $BB cat > src/thread/x86_64/__pthread_self.s <<'EOF'
.global __pthread_self
.hidden __pthread_self
__pthread_self:
    mov %fs:0,%rax
    ret
EOF
      # Split chained pthread-field assignments that trigger the predecessor's
      # aggregate lvalue codegen failure while preserving assignment order.
      $BB sed -i 's/td->next = td->prev = td;/td->next = td; td->prev = td;/' src/env/__init_tls.c
      $BB sed -i 's/td->dtv = td->dtv_copy = dtv;/td->dtv = dtv; td->dtv_copy = dtv;/' src/env/__init_tls.c
      # Avoid the self-host front end's aggregate-copy path while preserving
      # the per-category global locale update.
      $BB sed -i 's/libc.global_locale = tmp_locale;/for (i=0; i<LC_ALL; i++) libc.global_locale.cat[i] = tmp_locale.cat[i];/' src/locale/setlocale.c
      # Route signal syscalls through the bounded register-array helper rather
      # than the predecessor-crashing five-argument C call emitter.  This keeps
      # real kernel signal disposition while deferring musl's threaded handler
      # bookkeeping to the later GCC-built libc.
      $BB cat > src/signal/sigaction.c <<'EOF'
#include <errno.h>
#include <signal.h>
#include <stdint.h>
#include <string.h>
#include "ksigaction.h"
#include "libc.h"
#include "syscall.h"

enum {
    SYSCALL_ARGUMENT_COUNT = 6,
    BITS_PER_BYTE = 8,
    INTERNAL_SIGNAL_BASE = 32,
    INTERNAL_SIGNAL_COUNT = 3,
};

static volatile int dummy_lock[1] = { 0 };
extern hidden volatile int __abort_lock[1];
weak_alias(dummy_lock, __abort_lock);

static unsigned long handler_set[_NSIG / (BITS_PER_BYTE * sizeof(long))];
volatile int __eintr_valid_flag;
hidden long __bootstrap_syscall_array(long, const syscall_arg_t *);

void __get_handler_set(sigset_t *set)
{
    memcpy(set, handler_set, sizeof handler_set);
}

int __libc_sigaction(int signal_number, const struct sigaction *restrict action,
                     struct sigaction *restrict old_action)
{
    struct k_sigaction kernel_action;
    struct k_sigaction old_kernel_action;
    syscall_arg_t syscall_arguments[SYSCALL_ARGUMENT_COUNT];
    syscall_arg_t action_argument = 0;
    syscall_arg_t old_action_argument = 0;
    if (action) {
        kernel_action.handler = action->sa_handler;
        kernel_action.flags = action->sa_flags | SA_RESTORER;
        if (action->sa_flags & SA_SIGINFO) {
            kernel_action.restorer = __restore_rt;
        } else {
            kernel_action.restorer = __restore;
        }
        memcpy(&kernel_action.mask, &action->sa_mask, _NSIG / BITS_PER_BYTE);
        action_argument = (syscall_arg_t)&kernel_action;
    }
    if (old_action) old_action_argument = (syscall_arg_t)&old_kernel_action;
    syscall_arguments[0] = signal_number;
    syscall_arguments[1] = action_argument;
    syscall_arguments[2] = old_action_argument;
    syscall_arguments[3] = _NSIG / BITS_PER_BYTE;
    syscall_arguments[4] = 0;
    syscall_arguments[5] = 0;
    int result = __bootstrap_syscall_array(SYS_rt_sigaction, syscall_arguments);
    if (old_action && !result) {
        old_action->sa_handler = old_kernel_action.handler;
        old_action->sa_flags = old_kernel_action.flags;
        memcpy(&old_action->sa_mask, &old_kernel_action.mask, _NSIG / BITS_PER_BYTE);
    }
    return __syscall_ret(result);
}

int __sigaction(int signal_number, const struct sigaction *restrict action,
                struct sigaction *restrict old_action)
{
    if ((unsigned)signal_number - INTERNAL_SIGNAL_BASE < INTERNAL_SIGNAL_COUNT ||
        (unsigned)signal_number - 1U >= _NSIG - 1) {
        errno = EINVAL;
        return -1;
    }
    return __libc_sigaction(signal_number, action, old_action);
}

weak_alias(__sigaction, sigaction);
EOF
      # Express the same member offset without the unsupported builtin.
      $BB sed -i 's/#define MIN_TLS_ALIGN offsetof(struct builtin_tls, pt)/#define MIN_TLS_ALIGN ((size_t)\&((struct builtin_tls *)0)->pt)/' src/env/__init_tls.c
      # Keep the six-register mmap syscall out of the predecessor's fragile C
      # call emitter while preserving the x86_64 Linux syscall contract.
      $BB sed -i '/volatile int __thread_list_lock;/i\hidden void *__bootstrap_anonymous_mmap(size_t);' src/env/__init_tls.c
      $BB sed -i '/mem = (void \*)__syscall(/,/0);/c\
        mem = __bootstrap_anonymous_mmap(libc.tls_size);' src/env/__init_tls.c
      $BB sed -i 's/weak_alias(static_init_tls, __init_tls);/void __init_tls(size_t *aux) { static_init_tls(aux); }/' src/env/__init_tls.c
      $BB sed -i 's/weak_alias(__errno_location, ___errno_location);/int *___errno_location(void) { return __errno_location(); }/' src/errno/__errno_location.c
      $BB cat > src/mman/__bootstrap_anonymous_mmap.c <<'EOF'
#include <stddef.h>
#include "libc.h"
#include "syscall.h"
hidden long __bootstrap_raw_anonymous_mmap(size_t);
hidden void *__bootstrap_anonymous_mmap(size_t size)
{
    return (void *)__syscall_ret(__bootstrap_raw_anonymous_mmap(size));
}
EOF
      $BB mkdir -p src/mman/x86_64
      $BB cat > src/mman/x86_64/__bootstrap_raw_anonymous_mmap.s <<'EOF'
.equ SYS_MMAP, 9
.equ PROT_READ_WRITE, 3
.equ MAP_ANONYMOUS_PRIVATE, 0x22
.equ INVALID_FD, -1
.global __bootstrap_raw_anonymous_mmap
.hidden __bootstrap_raw_anonymous_mmap
__bootstrap_raw_anonymous_mmap:
    mov %rdi,%rsi
    xor %rdi,%rdi
    mov $PROT_READ_WRITE,%rdx
    mov $MAP_ANONYMOUS_PRIVATE,%r10
    mov $INVALID_FD,%r8
    xor %r9,%r9
    mov $SYS_MMAP,%rax
    syscall
    ret
EOF
      # Split the locale/TLS pointer chain so the self-hosted TinyCC can emit
      # the original MB_CUR_MAX semantics without crashing in one expression.
      $BB sed -i '/return MB_CUR_MAX;/c\
    enum { C_LOCALE_MB_MAX = 1, UTF8_LOCALE_MB_MAX = 4 };\
    struct pthread *thread = __pthread_self();\
    locale_t current = thread->locale;\
    return current->cat[LC_CTYPE] ? UTF8_LOCALE_MB_MAX : C_LOCALE_MB_MAX;' src/ctype/__ctype_get_mb_cur_max.c
      # The predecessor cannot emit musl's arena allocator.  Keep a real,
      # bounded mmap allocator for this static-only handoff; the later GCC libc
      # rebuild restores musl's complete arena implementation.
      $BB cat > src/malloc/malloc.c <<'EOF'
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include "libc.h"
#include "malloc_impl.h"

hidden void *__bootstrap_anonymous_mmap(size_t);
int __malloc_replaced;

static int allocation_size(size_t requested_bytes, size_t *mapped_bytes)
{
    if (requested_bytes > SIZE_MAX - SIZE_ALIGN - PAGE_SIZE) {
        errno = ENOMEM;
        return -1;
    }
    *mapped_bytes = (requested_bytes + SIZE_ALIGN + PAGE_SIZE - 1) & -PAGE_SIZE;
    return 0;
}

void *malloc(size_t requested_bytes)
{
    size_t mapped_bytes;
    if (allocation_size(requested_bytes, &mapped_bytes) < 0) return 0;
    char *base = __bootstrap_anonymous_mmap(mapped_bytes);
    if (base == (void *)-1) return 0;
    struct chunk *chunk = (void *)(base + SIZE_ALIGN - OVERHEAD);
    chunk->psize = SIZE_ALIGN - OVERHEAD;
    chunk->csize = mapped_bytes - (SIZE_ALIGN - OVERHEAD);
    return CHUNK_TO_MEM(chunk);
}

void *calloc(size_t member_count, size_t member_bytes)
{
    if (member_bytes && member_count > SIZE_MAX / member_bytes) {
        errno = ENOMEM;
        return 0;
    }
    size_t requested_bytes = member_count * member_bytes;
    void *memory = malloc(requested_bytes);
    if (!memory) return 0;
    return memset(memory, 0, requested_bytes);
}

void free(void *memory)
{
    if (!memory) return;
    struct chunk *chunk = MEM_TO_CHUNK(memory);
    if (!IS_MMAPPED(chunk)) abort();
    size_t prefix_bytes = chunk->psize;
    char *base = (char *)chunk - prefix_bytes;
    size_t mapped_bytes = chunk->csize + prefix_bytes;
    munmap(base, mapped_bytes);
}

void *realloc(void *memory, size_t requested_bytes)
{
    if (!memory) return malloc(requested_bytes);
    struct chunk *chunk = MEM_TO_CHUNK(memory);
    if (!IS_MMAPPED(chunk)) abort();
    size_t available_bytes = chunk->csize - OVERHEAD;
    void *replacement = malloc(requested_bytes);
    if (!replacement) return 0;
    size_t copy_bytes = requested_bytes < available_bytes ? requested_bytes : available_bytes;
    memcpy(replacement, memory, copy_bytes);
    free(memory);
    return replacement;
}
EOF
      $BB cat > src/malloc/memalign.c <<'EOF'
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>
#include "malloc_impl.h"

void *__memalign(size_t alignment_bytes, size_t requested_bytes)
{
    if ((alignment_bytes & -alignment_bytes) != alignment_bytes) {
        errno = EINVAL;
        return 0;
    }
    if (alignment_bytes <= SIZE_ALIGN) return malloc(requested_bytes);
    if (requested_bytes > SIZE_MAX - alignment_bytes) {
        errno = ENOMEM;
        return 0;
    }
    unsigned char *memory = malloc(requested_bytes + alignment_bytes - 1);
    if (!memory) return 0;
    unsigned char *aligned = (void *)((uintptr_t)memory + alignment_bytes - 1 & -alignment_bytes);
    if (aligned == memory) return memory;
    struct chunk *original = MEM_TO_CHUNK(memory);
    struct chunk *adjusted = MEM_TO_CHUNK(aligned);
    size_t adjustment_bytes = aligned - memory;
    adjusted->psize = original->psize + adjustment_bytes;
    adjusted->csize = original->csize - adjustment_bytes;
    return aligned;
}

weak_alias(__memalign, memalign);
EOF
      $BB cat > src/conf/confstr.c <<'EOF'
#include <errno.h>
#include <string.h>
#include <unistd.h>
size_t confstr(int name, char *buffer, size_t buffer_bytes)
{
    enum {
        CONFSTR_VARIANT_MASK = 4,
        CONFSTR_BASE_VARIANT = 1,
        CONFSTR_MAX_OFFSET = 33,
    };
    const char *value = "";
    if (!name) {
        value = "/bin:/usr/bin";
    } else if ((name & ~CONFSTR_VARIANT_MASK) != CONFSTR_BASE_VARIANT &&
               name - _CS_POSIX_V6_ILP32_OFF32_CFLAGS > CONFSTR_MAX_OFFSET) {
        errno = EINVAL;
        return 0;
    }
    size_t value_bytes = strlen(value);
    if (buffer_bytes) {
        size_t copy_bytes = value_bytes < buffer_bytes - 1 ? value_bytes : buffer_bytes - 1;
        memcpy(buffer, value, copy_bytes);
        buffer[copy_bytes] = 0;
    }
    return value_bytes + 1;
}
EOF
      $BB cat > src/misc/ffs.c <<'EOF'
#include <limits.h>
#include <strings.h>
int ffs(int value)
{
    unsigned int bits = value;
    int bit_index;
    if (!bits) return 0;
    for (bit_index = 1; bit_index <= (int)(sizeof(bits) * CHAR_BIT); bit_index++) {
        if (bits & 1U) return bit_index;
        bits >>= 1;
    }
    return 0;
}
EOF
      $BB cat > src/misc/ffsl.c <<'EOF'
#include <limits.h>
#include <strings.h>
int ffsl(long value)
{
    unsigned long bits = value;
    int bit_index;
    if (!bits) return 0;
    for (bit_index = 1; bit_index <= (int)(sizeof(bits) * CHAR_BIT); bit_index++) {
        if (bits & 1UL) return bit_index;
        bits >>= 1;
    }
    return 0;
}
EOF
      $BB cat > src/misc/ffsll.c <<'EOF'
#include <limits.h>
#include <strings.h>
int ffsll(long long value)
{
    unsigned long long bits = value;
    int bit_index;
    if (!bits) return 0;
    for (bit_index = 1; bit_index <= (int)(sizeof(bits) * CHAR_BIT); bit_index++) {
        if (bits & 1ULL) return bit_index;
        bits >>= 1;
    }
    return 0;
}
EOF
      $BB rm -f src/misc/syslog.c src/mman/mremap.c src/process/fexecve.c \
        src/signal/x86_64/sigsetjmp.s src/thread/mtx_*.c src/thread/cnd_*.c \
        src/thread/thrd_*.c src/thread/tss_*.c src/thread/pthread_barrier_*.c \
        src/thread/pthread_cond_*.c src/thread/pthread_mutex_*.c \
        src/thread/pthread_rwlock_*.c src/thread/pthread_spin_*.c \
        src/thread/pthread_create.c src/thread/pthread_join.c \
        src/thread/pthread_detach.c src/thread/pthread_cancel.c \
        src/thread/pthread_key_*.c src/thread/pthread_getspecific.c \
        src/thread/pthread_setspecific.c src/thread/sem_*.c src/time/timer_*.c \
        src/unistd/fchownat.c src/unistd/linkat.c
      $BB cat > src/thread/x86_64/__syscall_cp.s <<'EOF'
.equ STACK_ARGUMENT_BYTES, 8
.global __syscall_cp
.hidden __syscall_cp
.global __syscall_cp_c
.hidden __syscall_cp_c
.type __syscall_cp,@function
__syscall_cp:
__syscall_cp_c:
    mov %rdi,%rax
    mov %rsi,%rdi
    mov %rdx,%rsi
    mov %rcx,%rdx
    mov %r8,%r10
    mov %r9,%r8
    mov STACK_ARGUMENT_BYTES(%rsp),%r9
    syscall
    ret
EOF
      $BB cat > src/time/clock_gettime.c <<'EOF'
#include <errno.h>
#include <stdint.h>
#include <time.h>
#include "libc.h"
#include "syscall.h"

enum {
    SYSCALL_ARGUMENT_COUNT = 6,
    NANOSECONDS_PER_MICROSECOND = 1000,
};
hidden long __bootstrap_syscall_array(long, const syscall_arg_t *);

int __clock_gettime(clockid_t clock_id, struct timespec *time_value)
{
    syscall_arg_t arguments[SYSCALL_ARGUMENT_COUNT];
    arguments[0] = clock_id;
    arguments[1] = (syscall_arg_t)time_value;
    arguments[2] = 0;
    arguments[3] = 0;
    arguments[4] = 0;
    arguments[5] = 0;
    int result = __bootstrap_syscall_array(SYS_clock_gettime, arguments);
    if (result == -ENOSYS && clock_id == CLOCK_REALTIME) {
        arguments[0] = (syscall_arg_t)time_value;
        arguments[1] = 0;
        result = __bootstrap_syscall_array(SYS_gettimeofday, arguments);
        time_value->tv_nsec = (int)time_value->tv_nsec * NANOSECONDS_PER_MICROSECOND;
    }
    return __syscall_ret(result);
}

weak_alias(__clock_gettime, clock_gettime);
EOF
      $BB cat > src/thread/vmlock.c <<'EOF'
#include <stdlib.h>
#include "pthread_impl.h"
static int virtual_memory_lock;
void __vm_wait(void)
{
    if (libc.threads_minus_1) abort();
    if (virtual_memory_lock) abort();
}
void __vm_lock(void)
{
    if (libc.threads_minus_1) abort();
    if (virtual_memory_lock) abort();
    virtual_memory_lock = 1;
}
void __vm_unlock(void)
{
    if (libc.threads_minus_1) abort();
    if (!virtual_memory_lock) abort();
    virtual_memory_lock = 0;
}
EOF
      $BB cat > src/thread/pthread_once.c <<'EOF'
#include <stdlib.h>
#include "pthread_impl.h"

enum {
    ONCE_INITIAL = 0,
    ONCE_RUNNING = 1,
    ONCE_COMPLETE = 2,
};

hidden int __pthread_once_full(pthread_once_t *control, void (*initialize)(void))
{
    if (libc.threads_minus_1) abort();
    if (*control == ONCE_COMPLETE) return 0;
    if (*control != ONCE_INITIAL) abort();
    *control = ONCE_RUNNING;
    initialize();
    *control = ONCE_COMPLETE;
    return 0;
}

int __pthread_once(pthread_once_t *control, void (*initialize)(void))
{
    if (*control == ONCE_COMPLETE) return 0;
    return __pthread_once_full(control, initialize);
}

weak_alias(__pthread_once, pthread_once);
EOF
      $BB cat > src/thread/__wait.c <<'EOF'
#include <stdlib.h>
#include "pthread_impl.h"
void __wait(volatile int *address, volatile int *waiters, int expected, int private_wait)
{
    if (libc.threads_minus_1) abort();
    (void)waiters;
    (void)private_wait;
    if (*address == expected) abort();
}
EOF
      $BB cat > src/thread/__lock.c <<'EOF'
#include <stdlib.h>
#include "pthread_impl.h"
void __lock(volatile int *lock)
{
    if (libc.threads_minus_1) abort();
    (void)lock;
}
void __unlock(volatile int *lock)
{
    if (libc.threads_minus_1) abort();
    (void)lock;
}
EOF
      $BB cat > src/stdlib/qsort.c <<'EOF'
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

static void swap_objects(unsigned char *left, unsigned char *right, size_t object_bytes)
{
    size_t byte_index;
    for (byte_index = 0; byte_index < object_bytes; byte_index++) {
        unsigned char saved = left[byte_index];
        left[byte_index] = right[byte_index];
        right[byte_index] = saved;
    }
}

void qsort(void *base, size_t object_count, size_t object_bytes,
           int (*compare)(const void *, const void *))
{
    enum { MIN_SORTABLE_OBJECTS = 2 };
    if (object_count < MIN_SORTABLE_OBJECTS || !object_bytes) return;
    if (object_count > SIZE_MAX / object_bytes) return;
    unsigned char *objects = base;
    size_t object_index;
    for (object_index = 1; object_index < object_count; object_index++) {
        size_t insertion_index = object_index;
        while (insertion_index > 0) {
            unsigned char *left = objects + (insertion_index - 1) * object_bytes;
            unsigned char *right = left + object_bytes;
            if (compare(left, right) <= 0) break;
            swap_objects(left, right, object_bytes);
            insertion_index--;
        }
    }
}
EOF
      $BB cat > src/stdio/getc.h <<'EOF'
#include "stdio_impl.h"
static int locking_getc(FILE *stream)
{
    int acquired = __lockfile(stream);
    int character = getc_unlocked(stream);
    if (acquired) __unlockfile(stream);
    return character;
}
static int do_getc(FILE *stream)
{
    if (stream->lock < 0) return getc_unlocked(stream);
    return locking_getc(stream);
}
EOF
      $BB cat > src/stdio/putc.h <<'EOF'
#include "stdio_impl.h"
static int locking_putc(int character, FILE *stream)
{
    int acquired = __lockfile(stream);
    int result = putc_unlocked(character, stream);
    if (acquired) __unlockfile(stream);
    return result;
}
static int do_putc(int character, FILE *stream)
{
    if (stream->lock < 0) return putc_unlocked(character, stream);
    return locking_putc(character, stream);
}
EOF
      $BB cat > src/stdio/ftrylockfile.c <<'EOF'
#include <limits.h>
#include <stdlib.h>
#include "libc.h"
#include "pthread_impl.h"
#include "stdio_impl.h"

enum { ORPHANED_LOCK_VALUE = 0x40000000 };

void __do_orphaned_stdio_locks(void)
{
    FILE *stream;
    if (libc.threads_minus_1) abort();
    for (stream = __pthread_self()->stdio_locks; stream; stream = stream->next_locked) {
        stream->lock = ORPHANED_LOCK_VALUE;
    }
}

void __unlist_locked_file(FILE *stream)
{
    if (stream->lockcount) {
        if (stream->next_locked) stream->next_locked->prev_locked = stream->prev_locked;
        if (stream->prev_locked) stream->prev_locked->next_locked = stream->next_locked;
        else __pthread_self()->stdio_locks = stream->next_locked;
    }
}

void __register_locked_file(FILE *stream, pthread_t self)
{
    stream->lockcount = 1;
    stream->prev_locked = 0;
    stream->next_locked = self->stdio_locks;
    if (stream->next_locked) stream->next_locked->prev_locked = stream;
    self->stdio_locks = stream;
}

int ftrylockfile(FILE *stream)
{
    if (libc.threads_minus_1) abort();
    pthread_t self = __pthread_self();
    int thread_id = self->tid;
    if (stream->lock == thread_id) {
        if (stream->lockcount == LONG_MAX) return -1;
        stream->lockcount++;
        return 0;
    }
    if (stream->lock < 0) stream->lock = 0;
    if (stream->lock) return -1;
    stream->lock = thread_id;
    __register_locked_file(stream, self);
    return 0;
}
EOF
      $BB cat > src/stdio/__lockfile.c <<'EOF'
#include <stdlib.h>
#include "libc.h"
#include "pthread_impl.h"
#include "stdio_impl.h"

int __lockfile(FILE *stream)
{
    if (libc.threads_minus_1) abort();
    int thread_id = __pthread_self()->tid;
    if (stream->lock == thread_id) return 0;
    if (stream->lock) abort();
    stream->lock = thread_id;
    return 1;
}

void __unlockfile(FILE *stream)
{
    if (libc.threads_minus_1) abort();
    stream->lock = 0;
}
EOF
      $BB cat > src/stat/fstatat.c <<'EOF'
#define _BSD_SOURCE
#include <fcntl.h>
#include <stdint.h>
#include <sys/stat.h>
#include "kstat.h"
#include "libc.h"
#include "syscall.h"

enum { SYSCALL_ARGUMENT_COUNT = 6 };
hidden long __bootstrap_syscall_array(long, const syscall_arg_t *);

int fstatat(int fd, const char *restrict path, struct stat *restrict status, int flags)
{
    struct kstat kernel_status;
    syscall_arg_t arguments[SYSCALL_ARGUMENT_COUNT];
    arguments[0] = fd;
    arguments[1] = (syscall_arg_t)path;
    arguments[2] = (syscall_arg_t)&kernel_status;
    arguments[3] = flags;
    arguments[4] = 0;
    arguments[5] = 0;
    int result = __bootstrap_syscall_array(SYS_fstatat, arguments);
    if (result) return __syscall_ret(result);
    status->st_dev = kernel_status.st_dev;
    status->st_ino = kernel_status.st_ino;
    status->st_mode = kernel_status.st_mode;
    status->st_nlink = kernel_status.st_nlink;
    status->st_uid = kernel_status.st_uid;
    status->st_gid = kernel_status.st_gid;
    status->st_rdev = kernel_status.st_rdev;
    status->st_size = kernel_status.st_size;
    status->st_blksize = kernel_status.st_blksize;
    status->st_blocks = kernel_status.st_blocks;
    status->st_atim.tv_sec = kernel_status.st_atime_sec;
    status->st_atim.tv_nsec = kernel_status.st_atime_nsec;
    status->st_mtim.tv_sec = kernel_status.st_mtime_sec;
    status->st_mtim.tv_nsec = kernel_status.st_mtime_nsec;
    status->st_ctim.tv_sec = kernel_status.st_ctime_sec;
    status->st_ctim.tv_nsec = kernel_status.st_ctime_nsec;
    return 0;
}
EOF
      $BB cat > src/mman/mmap.c <<'EOF'
#include <errno.h>
#include <limits.h>
#include <stdint.h>
#include <sys/mman.h>
#include <unistd.h>
#include "libc.h"
#include "syscall.h"

static void dummy(void) { }
weak_alias(dummy, __vm_wait);

enum {
    SYSCALL_ARGUMENT_COUNT = 6,
    BITS_PER_BYTE = 8,
};
#define MMAP2_UNIT SYSCALL_MMAP2_UNIT
#define MMAP2_OFFSET_MASK_BASE 0x2000ULL
#define MMAP2_OFFSET_MASK \
    ((-MMAP2_OFFSET_MASK_BASE << (BITS_PER_BYTE * sizeof(syscall_arg_t) - 1)) | (MMAP2_UNIT - 1))

hidden long __bootstrap_syscall_array(long, const syscall_arg_t *);

void *__mmap(void *start, size_t length_bytes, int protection, int flags, int fd, off_t offset_bytes)
{
    long result;
    syscall_arg_t arguments[SYSCALL_ARGUMENT_COUNT];
    if (offset_bytes & MMAP2_OFFSET_MASK) {
        errno = EINVAL;
        return MAP_FAILED;
    }
    if (length_bytes >= PTRDIFF_MAX) {
        errno = ENOMEM;
        return MAP_FAILED;
    }
    if (flags & MAP_FIXED) __vm_wait();
    arguments[0] = (syscall_arg_t)start;
    arguments[1] = length_bytes;
    arguments[2] = protection;
    arguments[3] = flags;
    arguments[4] = fd;
#ifdef SYS_mmap2
    arguments[5] = offset_bytes / MMAP2_UNIT;
    result = __bootstrap_syscall_array(SYS_mmap2, arguments);
#else
    arguments[5] = offset_bytes;
    result = __bootstrap_syscall_array(SYS_mmap, arguments);
#endif
    if (result == -EPERM && !start && (flags & MAP_ANON) && !(flags & MAP_FIXED)) {
        result = -ENOMEM;
    }
    return (void *)__syscall_ret(result);
}

weak_alias(__mmap, mmap);
weak_alias(mmap, mmap64);
EOF
      $BB cat > src/misc/syscall.c <<'EOF'
#define _BSD_SOURCE
#include <stdarg.h>
#include <unistd.h>
#include "libc.h"
#include "syscall.h"

#undef syscall

enum { SYSCALL_ARGUMENT_COUNT = 6 };
hidden long __bootstrap_syscall_array(long, const syscall_arg_t *);

long syscall(long number, ...)
{
    syscall_arg_t arguments[SYSCALL_ARGUMENT_COUNT];
    va_list variable_arguments;
    int argument_index;
    va_start(variable_arguments, number);
    for (argument_index = 0; argument_index < SYSCALL_ARGUMENT_COUNT; argument_index++) {
        arguments[argument_index] = va_arg(variable_arguments, syscall_arg_t);
    }
    va_end(variable_arguments);
    return __syscall_ret(__bootstrap_syscall_array(number, arguments));
}
EOF
      $BB mkdir -p src/misc/x86_64
      $BB cat > src/misc/x86_64/__bootstrap_syscall_array.s <<'EOF'
.equ ARGUMENT_0_OFFSET, 0
.equ ARGUMENT_1_OFFSET, 8
.equ ARGUMENT_2_OFFSET, 16
.equ ARGUMENT_3_OFFSET, 24
.equ ARGUMENT_4_OFFSET, 32
.equ ARGUMENT_5_OFFSET, 40
.global __bootstrap_syscall_array
.hidden __bootstrap_syscall_array
.type __bootstrap_syscall_array,@function
__bootstrap_syscall_array:
    mov %rdi,%rax
    mov %rsi,%r11
    mov ARGUMENT_0_OFFSET(%r11),%rdi
    mov ARGUMENT_1_OFFSET(%r11),%rsi
    mov ARGUMENT_2_OFFSET(%r11),%rdx
    mov ARGUMENT_3_OFFSET(%r11),%r10
    mov ARGUMENT_4_OFFSET(%r11),%r8
    mov ARGUMENT_5_OFFSET(%r11),%r9
    syscall
    ret
EOF
      # This TinyCC assembler crashes on symbolic .equ operands.  Keep named
      # constants in the reviewed source and lower them deterministically only
      # in the sandbox copy consumed by that predecessor.
      $BB sed -i -e 's/STACK_ARGUMENT_BYTES/8/g' \
        -e 's/STACK_ALIGNMENT_MASK/-16/g' \
        -e 's/SYS_EXIT/60/g' crt/crt1.c
      $BB sed -i -e '/^\.equ /d' \
        -e 's/SYS_MMAP/9/g' \
        -e 's/PROT_READ_WRITE/3/g' \
        -e 's/MAP_ANONYMOUS_PRIVATE/0x22/g' \
        -e 's/INVALID_FD/-1/g' src/mman/x86_64/__bootstrap_raw_anonymous_mmap.s
      $BB sed -i -e '/^\.equ /d' \
        -e 's/ARGUMENT_0_OFFSET/0/g' \
        -e 's/ARGUMENT_1_OFFSET/8/g' \
        -e 's/ARGUMENT_2_OFFSET/16/g' \
        -e 's/ARGUMENT_3_OFFSET/24/g' \
        -e 's/ARGUMENT_4_OFFSET/32/g' \
        -e 's/ARGUMENT_5_OFFSET/40/g' src/misc/x86_64/__bootstrap_syscall_array.s
      $BB sed -i -e '/^\.equ /d' \
        -e 's/STACK_ARGUMENT_BYTES/8/g' src/thread/x86_64/__syscall_cp.s
      # TinyCC's assembler conflates numeric local labels in the optimized
      # x86_64 memory primitives.  Select musl's portable C implementations.
      $BB rm -f src/string/x86_64/memcpy.s \
        src/string/x86_64/memmove.s src/string/x86_64/memset.s

      # Standalone assembly gets malformed section metadata from this TinyCC.
      # Re-express selected x86_64 assembly as inline assembly after a tiny C
      # function has established the text section, then compile it with the
      # declared TinyCC 0.9.26 assembler predecessor.
      INLINE_ASSEMBLY_C="$WORK/inline-assembly.c"
      for assembly_source in src/*/x86_64/*.s; do
        test -f "$assembly_source" || continue
        assembly_object="obj/${assembly_source%.s}.o"
        $BB mkdir -p "$($BB dirname "$assembly_object")"
        $BB awk '
          BEGIN { print "static void force_text_section(void) {}"; print "__asm__(" }
          /^\.text$/ { next }
          { gsub(/\\/, "\\\\"); gsub(/\"/, "\\\""); print "\"" $0 "\\n\"" }
          END { print ");" }
        ' "$assembly_source" > "$INLINE_ASSEMBLY_C"
        "$TCC26/bin/tcc" -c "$INLINE_ASSEMBLY_C" -o "$assembly_object"
        $BB chmod u+rw "$assembly_object"
      done

      ALLTYPES=arch/x86_64/bits/alltypes.h.in
      test -f "$ALLTYPES" || { echo "ERROR: musl x86_64 alltypes input missing" >&2; exit 1; }

      ./configure \
        --host=x86_64-linux-musl \
        --disable-shared \
        --prefix="$out" \
        --libdir="$out/lib" \
        --includedir="$out/include" \
        CC="$TCC_CLEAN"

      make -j1 obj/include/bits/alltypes.h
      ALLTYPES_GENERATED=obj/include/bits/alltypes.h
      $BB sed -i '1i\
#if defined(__TINYC__) && !defined(__DEFINED_mantle_va_list_struct)\
#define __DEFINED_mantle_va_list_struct\
typedef struct {\
    unsigned int gp_offset;\
    unsigned int fp_offset;\
    union { unsigned int overflow_offset; char *overflow_arg_area; };\
    char *reg_save_area;\
} __mantle_va_list_struct;\
#endif' "$ALLTYPES_GENERATED"
      $BB sed -i 's/typedef __builtin_va_list va_list;/#if defined(__TINYC__)\
typedef __mantle_va_list_struct va_list[1];\
#else\
typedef __builtin_va_list va_list;\
#endif/' "$ALLTYPES_GENERATED"
      $BB sed -i 's/typedef __builtin_va_list __isoc_va_list;/#if defined(__TINYC__)\
typedef __mantle_va_list_struct __isoc_va_list[1];\
#else\
typedef __builtin_va_list __isoc_va_list;\
#endif/' "$ALLTYPES_GENERATED"

      # TinyCC emits read-restricted objects in this bootstrap configuration;
      # restore owner read/write permission before archive/install rules consume them.
      $BB sed -i 's|^\(CC_CMD =.*\)|\1 \&\& chmod u+rw $@|' Makefile
      # The self-hosted front end compiles libc C; its declared TinyCC 0.9.26
      # linker predecessor emits the startup object with a valid text section.
      $BB mkdir -p obj/crt
      "$TCC26/bin/tcc" -c crt/crt1.c -o obj/crt/crt1.o
      $BB chmod u+rw obj/crt/crt1.o
      # Compile the bounded confstr replacement without the predecessor-crashing
      # aggregate of configure-probed optimization flags.
      $BB mkdir -p obj/src/conf
      "$TCC_CLEAN" -std=c99 -nostdinc \
        -I./arch/x86_64 -I./arch/generic -Iobj/src/internal \
        -I./src/include -I./src/internal -Iobj/include -I./include \
        -D_XOPEN_SOURCE=700 -DSYSCALL_NO_TLS -fno-stack-protector \
        -c src/conf/confstr.c -o obj/src/conf/confstr.o
      $BB chmod u+rw obj/src/conf/confstr.o
      # The first self-host front end still crashes on the legacy fdlibm
      # translation units.  Compile this explicit upstream source closure with
      # the declared source-built predecessor; the later GCC-built libc must
      # replace this bounded two-compiler handoff before provider admission.
      $BB mkdir -p obj/src/math
      for math_source in src/math/*.c; do
        math_object="obj/${math_source%.c}.o"
        echo "native-musl-math-source=$math_source" >&2
        "$TCC26/bin/tcc" -std=c99 -nostdinc \
          -I./arch/x86_64 -I./arch/generic -Iobj/src/internal \
          -I./src/include -I./src/internal -Iobj/include -I./include \
          -D_XOPEN_SOURCE=700 -DSYSCALL_NO_TLS -fno-stack-protector \
          -c "$math_source" -o "$math_object"
        $BB chmod u+rw "$math_object"
      done
      make -j1 \
        CROSS_COMPILE= \
        AR="$TCC_AR" \
        RANLIB=true \
        CFLAGS_AUTO= \
        CFLAGS="-DSYSCALL_NO_TLS"
      make install

      $BB cat > "$WORK/native-musl-smoke.c" <<'EOF'
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
EOF
      NATIVE_SMOKE_OBJECT="$WORK/native-musl-smoke.o"
      NATIVE_SMOKE_BINARY="$WORK/native-musl-smoke"
      "$TCC_CLEAN" -c -I"$out/include" "$WORK/native-musl-smoke.c" -o "$NATIVE_SMOKE_OBJECT"
      $BB chmod u+rw "$NATIVE_SMOKE_OBJECT"
      "$TCC_CLEAN" -nostdlib -static -o "$NATIVE_SMOKE_BINARY" \
        "$out/lib/crt1.o" "$NATIVE_SMOKE_OBJECT" "$out/lib/libc.a"
      $BB chmod 755 "$NATIVE_SMOKE_BINARY"
      NATIVE_SMOKE_OUTPUT=$("$NATIVE_SMOKE_BINARY")
      test "$NATIVE_SMOKE_OUTPUT" = native-musl-smoke-ok || {
        echo "ERROR: native musl runtime smoke output mismatch" >&2
        exit 1
      }

      test -f "$out/lib/libc.a"
      test -f "$out/lib/crt1.o"
      test -f "$out/include/stdio.h"
      test -f "$out/include/stdlib.h"
    "%,
  ],
  inputs = [stage0, tcc, tcc26, make, musl_src],
} | crunch.Derivation
