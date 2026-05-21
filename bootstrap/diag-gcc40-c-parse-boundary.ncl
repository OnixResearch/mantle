# Diagnostic: reproduce GCC 4.0.4 c-parse TinyCC boundary.
# Derived from bootstrap/gcc-4.0.ncl; intentionally stops at c-parse.o.
#
# This is the first real C compiler in the bootstrap chain, replacing
# TinyCC. GCC 4.0.4 is the oldest GCC that TCC can build. Attempts
# C and C++ languages; falls back to C-only if C++ fails.
#
# Adapted from live-bootstrap steps/gcc-4.0.4/pass1.sh.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let binutils = import "binutils-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in

let gcc_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2",
  hash = "sha256-QNHCUqWDcOtqVLokzAd3MfsWWR/jN3HdAld/Pm+GSM0=",
  name = "gcc-4.0.4-src",
} in

let tcc_src = crunch.fetchTarball {
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-iaWFw3NwCH5fV71ybBYKzXJKC8ALl6Hn6woEW29aXfs=",
  name = "tcc-0.9.27-src",
} in

{
  name = "diag-gcc40-c-parse-boundary",
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
      TCC_SRC=$(find_input tcc-0.9.27-src)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      BINUTILS=$(find_input binutils-2.30-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      M4=$(find_input m4-1.4.7-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      SRC=$(find_input gcc-4.0.4-src)

      # Avoid a direct bash-2.05b-tcc dependency here: this diagnostic forces
      # CONFIG_SHELL/SHELL to /bin/sh below. The binutils bridge still carries
      # the broader GCC-toolchain dependency chain.
      export PATH="$BINUTILS/bin:$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$GREP/bin:$DIFF/bin:$STAGE0/bin:$PATH"
      TOOLBIN=/tmp/gcc-tools
      $BB mkdir -p "$TOOLBIN"
      for applet in rm rmdir mkdir cp mv ln chmod touch cat sed grep egrep fgrep awk expr basename dirname tr sort uniq cmp diff test printf echo pwd uname sleep false true head tail wc cut; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$TOOLBIN:$PATH"
      cat > "$TOOLBIN/gcc40-cc" <<EOF
#!/bin/sh
out=a.out
saw_c=0
saw_conftest=0
saw_genmodes_src=0
saw_genmddeps_src=0
saw_gengenrtl_src=0
saw_gengtype_src=0
saw_gengtype_lex_src=0
saw_gengtype_yacc_src=0
saw_rtl_src=0
saw_read_rtl_src=0
saw_ggc_none_src=0
saw_min_insn_modes_src=0
saw_gensupport_src=0
saw_genconstants_src=0
saw_genflags_src=0
saw_genconfig_src=0
saw_gencodes_src=0
saw_genconditions_src=0
saw_genpreds_src=0
saw_print_rtl_src=0
saw_gencheck_src=0
saw_errors_src=0
prev_o=0
for arg in "\$@"; do
  if [ "\$prev_o" = 1 ]; then
    out="\$arg"
    prev_o=0
    continue
  fi
  case "\$arg" in
    -c) saw_c=1 ;;
    -o) prev_o=1 ;;
    *conftest.c) saw_conftest=1 ;;
    */genmodes.c|genmodes.c) saw_genmodes_src=1 ;;
    */genmddeps.c|genmddeps.c) saw_genmddeps_src=1 ;;
    */gengenrtl.c|gengenrtl.c) saw_gengenrtl_src=1 ;;
    */gengtype.c|gengtype.c) saw_gengtype_src=1 ;;
    */gengtype-lex.c|gengtype-lex.c) saw_gengtype_lex_src=1 ;;
    */gengtype-yacc.c|gengtype-yacc.c) saw_gengtype_yacc_src=1 ;;
    */rtl.c|rtl.c) saw_rtl_src=1 ;;
    */read-rtl.c|read-rtl.c) saw_read_rtl_src=1 ;;
    */ggc-none.c|ggc-none.c) saw_ggc_none_src=1 ;;
    */min-insn-modes.c|min-insn-modes.c) saw_min_insn_modes_src=1 ;;
    */gensupport.c|gensupport.c) saw_gensupport_src=1 ;;
    */genconstants.c|genconstants.c) saw_genconstants_src=1 ;;
    */genflags.c|genflags.c) saw_genflags_src=1 ;;
    */genconfig.c|genconfig.c) saw_genconfig_src=1 ;;
    */gencodes.c|gencodes.c) saw_gencodes_src=1 ;;
    */genconditions.c|genconditions.c) saw_genconditions_src=1 ;;
    */genpreds.c|genpreds.c) saw_genpreds_src=1 ;;
    */print-rtl.c|print-rtl.c) saw_print_rtl_src=1 ;;
    */gencheck.c|gencheck.c) saw_gencheck_src=1 ;;
    */errors.c|errors.c) saw_errors_src=1 ;;
  esac
done
if [ "\$saw_genmodes_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-genmodes-empty.c
  echo 'int gcc40_genmodes_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_genmddeps_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-genmddeps-empty.c
  echo 'int gcc40_genmddeps_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_gengenrtl_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-gengenrtl-empty.c
  echo 'int gcc40_gengenrtl_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_gengtype_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-gengtype-empty.c
  echo 'int gcc40_gengtype_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_gengtype_lex_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-gengtype-lex-empty.c
  echo 'int gcc40_gengtype_lex_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_gengtype_yacc_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-gengtype-yacc-empty.c
  echo 'int gcc40_gengtype_yacc_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_rtl_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-rtl-empty.c
  echo 'int gcc40_rtl_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_read_rtl_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-read-rtl-empty.c
  echo 'int gcc40_read_rtl_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_ggc_none_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-ggc-none-empty.c
  echo 'int gcc40_ggc_none_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_min_insn_modes_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-min-insn-modes-empty.c
  echo 'int gcc40_min_insn_modes_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_gensupport_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-gensupport-empty.c
  echo 'int gcc40_gensupport_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_genconstants_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-genconstants-empty.c
  echo 'int gcc40_genconstants_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_genflags_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-genflags-empty.c
  echo 'int gcc40_genflags_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_genconfig_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-genconfig-empty.c
  echo 'int gcc40_genconfig_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_gencodes_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-gencodes-empty.c
  echo 'int gcc40_gencodes_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_genconditions_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-genconditions-empty.c
  echo 'int gcc40_genconditions_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_genpreds_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-genpreds-empty.c
  echo 'int gcc40_genpreds_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_print_rtl_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-print-rtl-empty.c
  echo 'int gcc40_print_rtl_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_gencheck_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-gencheck-empty.c
  echo 'int gcc40_gencheck_bootstrap_stub;' > "\$tmp"
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
if [ "\$saw_errors_src" = 1 ] && [ "\$saw_c" = 1 ]; then
  tmp=/tmp/gcc40-errors-bridge.c
  cat > "\$tmp" <<'ERRORS_C'
int have_error;
const char *progname;
void warning(const char *s, ...) { (void)s; }
void error(const char *s, ...) { (void)s; have_error = 1; }
void fatal(const char *s, ...) { (void)s; for (;;) { } }
void internal_error(const char *s, ...) { (void)s; for (;;) { } }
const char *trim_filename(const char *s) { return s; }
ERRORS_C
  exec "$TCC/bin/tcc" -c "\$tmp" -o "\$out"
fi
case "\$out" in
  */build/genmodes|build/genmodes)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENMODES_SCRIPT'
#!/bin/sh
emit_h() {
cat <<'H'
#ifndef GCC_INSN_MODES_H
#define GCC_INSN_MODES_H
#define CONST_MODE_SIZE const
#define CONST_MODE_BASE_ALIGN const
enum machine_mode { VOIDmode, BLKmode, CCmode, BImode, QImode, HImode, SImode, DImode, TImode, SFmode, DFmode, XFmode, TFmode, MAX_MACHINE_MODE, NUM_MACHINE_MODES = MAX_MACHINE_MODE };
#define MIN_MODE_RANDOM VOIDmode
#define MAX_MODE_RANDOM BLKmode
#define MIN_MODE_CC CCmode
#define MAX_MODE_CC CCmode
#define MIN_MODE_INT QImode
#define MAX_MODE_INT TImode
#define MIN_MODE_FLOAT SFmode
#define MAX_MODE_FLOAT TFmode
#define MIN_MODE_PARTIAL_INT VOIDmode
#define MAX_MODE_PARTIAL_INT VOIDmode
#define MIN_MODE_COMPLEX_INT VOIDmode
#define MAX_MODE_COMPLEX_INT VOIDmode
#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
#define MIN_MODE_VECTOR_INT VOIDmode
#define MAX_MODE_VECTOR_INT VOIDmode
#define MIN_MODE_VECTOR_FLOAT VOIDmode
#define MAX_MODE_VECTOR_FLOAT VOIDmode
#endif
H
}
emit_c() {
cat <<'C'
#include "config.h"
#include "system.h"
#include "coretypes.h"
#include "tm.h"
#include "machmode.h"
#include "real.h"
const char * const mode_name[NUM_MACHINE_MODES] = { "VOID", "BLK", "CC", "BI", "QI", "HI", "SI", "DI", "TI", "SF", "DF", "XF", "TF" };
const unsigned char mode_class[NUM_MACHINE_MODES] = { MODE_RANDOM, MODE_RANDOM, MODE_CC, MODE_INT, MODE_INT, MODE_INT, MODE_INT, MODE_INT, MODE_INT, MODE_FLOAT, MODE_FLOAT, MODE_FLOAT, MODE_FLOAT };
CONST_MODE_SIZE unsigned char mode_size[NUM_MACHINE_MODES] = { 0, 0, 4, 1, 1, 2, 4, 8, 16, 4, 8, 12, 16 };
const unsigned short mode_precision[NUM_MACHINE_MODES] = { 0, 0, 32, 1, 8, 16, 32, 64, 128, 32, 64, 96, 128 };
const unsigned HOST_WIDE_INT mode_mask_array[NUM_MACHINE_MODES] = { 0, 0, ~0U, 1, 0xff, 0xffff, ~0U, ~0U, ~0U, 0, 0, 0, 0 };
const unsigned char mode_inner[NUM_MACHINE_MODES] = { VOIDmode };
const unsigned char mode_nunits[NUM_MACHINE_MODES] = { 1 };
const unsigned char mode_wider[NUM_MACHINE_MODES] = { VOIDmode, BLKmode, CCmode, QImode, HImode, SImode, DImode, TImode, TImode, DFmode, XFmode, TFmode, TFmode };
CONST_MODE_BASE_ALIGN unsigned char mode_base_align[NUM_MACHINE_MODES] = { 1, 1, 4, 1, 1, 2, 4, 8, 16, 4, 8, 16, 16 };
const unsigned char class_narrowest_mode[MAX_MODE_CLASS] = { VOIDmode, CCmode, QImode, VOIDmode, SFmode, VOIDmode, VOIDmode, VOIDmode, VOIDmode };
enum machine_mode byte_mode = QImode;
enum machine_mode word_mode = DImode;
enum machine_mode ptr_mode = DImode;
void init_adjust_machine_modes(void) { }
unsigned get_mode_alignment(enum machine_mode mode) { return mode_base_align[mode]; }
enum machine_mode mode_for_size(unsigned int size, enum mode_class class, int limit) { (void)limit; if (class == MODE_INT) { if (size <= 8) return QImode; if (size <= 16) return HImode; if (size <= 32) return SImode; if (size <= 64) return DImode; return TImode; } if (class == MODE_FLOAT) { if (size <= 32) return SFmode; if (size <= 64) return DFmode; return TFmode; } return BLKmode; }
enum machine_mode smallest_mode_for_size(unsigned int size, enum mode_class class) { return mode_for_size(size, class, 0); }
enum machine_mode int_mode_for_mode(enum machine_mode mode) { (void)mode; return SImode; }
enum machine_mode get_best_mode(int bitsize, int bitpos, unsigned int align, enum machine_mode mode, int volatilep) { (void)bitsize; (void)bitpos; (void)align; (void)volatilep; return mode == VOIDmode ? SImode : mode; }
C
}
case "\$1" in
  -h) emit_h ;;
  -m) emit_c ;;
  *) emit_c ;;
esac
GENMODES_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/genmddeps|build/genmddeps)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENMDDEPS_SCRIPT'
#!/bin/sh
printf 'MD_INCLUDES =\n'
GENMDDEPS_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/genconstants|build/genconstants)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENCONSTANTS_SCRIPT'
#!/bin/sh
cat <<'H'
#ifndef GCC_INSN_CONSTANTS_H
#define GCC_INSN_CONSTANTS_H
/* bootstrap genconstants stub: no md constants required before the next GCC boundary */
#endif
H
GENCONSTANTS_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/genconditions|build/genconditions)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENCONDITIONS_SCRIPT'
#!/bin/sh
cat <<'C'
#include <stddef.h>
struct c_test { const char *expr; int value; };
const struct c_test insn_conditions[1] = { { 0, 0 } };
const size_t n_insn_conditions = 0;
const int insn_elision_unavailable = 1;
C
GENCONDITIONS_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/gencheck|build/gencheck)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENCHECK_SCRIPT'
#!/bin/sh
cat <<'H'
#ifndef GCC_TREE_CHECK_H
#define GCC_TREE_CHECK_H
/* bootstrap gencheck stub */
#endif
H
GENCHECK_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/genpreds|build/genpreds)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENPREDS_SCRIPT'
#!/bin/sh
if [ "x\$1" = "x-h" ]; then
  cat <<'H'
#ifndef GCC_TM_PREDS_H
#define GCC_TM_PREDS_H
/* bootstrap genpreds header stub */
#endif
H
else
  cat <<'C'
#include "config.h"
#include "system.h"
#include "coretypes.h"
#include "tm.h"
#include "rtl.h"
#include "tree.h"
#include "insn-config.h"
#include "recog.h"
/* bootstrap genpreds source stub */
C
fi
GENPREDS_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/genflags|build/genflags)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENFLAGS_SCRIPT'
#!/bin/sh
cat <<'H'
#ifndef GCC_INSN_FLAGS_H
#define GCC_INSN_FLAGS_H
/* bootstrap genflags stub: generator-only boundary bridge */
#endif
H
GENFLAGS_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/genconfig|build/genconfig)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENCONFIG_SCRIPT'
#!/bin/sh
cat <<'H'
#ifndef GCC_INSN_CONFIG_H
#define GCC_INSN_CONFIG_H
#define MAX_RECOG_OPERANDS 10
#define MAX_DUP_OPERANDS 0
#define MAX_INSNS_PER_SPLIT 1
#define MAX_INSNS_PER_PEEP2 1
#endif
H
GENCONFIG_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/gencodes|build/gencodes)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENCODES_SCRIPT'
#!/bin/sh
cat <<'H'
#ifndef GCC_INSN_CODES_H
#define GCC_INSN_CODES_H
enum insn_code { CODE_FOR_nothing };
#endif
H
GENCODES_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/gengtype|build/gengtype)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENGTYPE_SCRIPT'
#!/bin/sh
files="gt-cgraph.h gt-coverage.h gtype-desc.h gt-except.h gt-function.h gt-integrate.h gt-tree.h gt-varasm.h gt-emit-rtl.h gt-explow.h gt-stor-layout.h gt-regclass.h gt-lists.h gt-alias.h gt-cselib.h gt-gcse.h gt-expr.h gt-sdbout.h gt-optabs.h gt-bitmap.h gt-dojump.h gt-dwarf2out.h gt-reg-stack.h gt-dwarf2asm.h gt-dbxout.h gt-c-common.h gt-c-decl.h gt-c-parse.h gt-c-pragma.h gtype-c.h gt-cfglayout.h gt-tree-mudflap.h gt-tree-complex.h gt-tree-eh.h gt-tree-ssanames.h gt-tree-iterator.h gt-gimplify.h gt-tree-phinodes.h gt-tree-cfg.h gt-tree-nested.h gt-tree-ssa-operands.h gt-tree-ssa-propagate.h gt-stringpool.h"
for f in \$files; do
  printf '/* bootstrap gengtype stub: %s */\n' "\$f" > "\$f"
done
cat > gtype-desc.c <<'GTYPE_DESC_C'
/* bootstrap gengtype stub */
#include "config.h"
#include "system.h"
#include "coretypes.h"
#include "tm.h"
GTYPE_DESC_C
exit 0
GENGTYPE_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
  */build/gengenrtl|build/gengenrtl)
    if [ "\$saw_c" = 0 ]; then
      cat > "\$out" <<'GENGENRTL_SCRIPT'
#!/bin/sh
if [ "x\$1" = "x-h" ]; then
  cat <<'H'
#ifndef GCC_GENRTL_H
#define GCC_GENRTL_H
extern rtx gen_rtx_fmt_0 (RTX_CODE, enum machine_mode);
extern rtx gen_rtx_fmt_i (RTX_CODE, enum machine_mode, int);
extern rtx gen_rtx_fmt_ii (RTX_CODE, enum machine_mode, int, int);
extern rtx gen_rtx_fmt_e (RTX_CODE, enum machine_mode, rtx);
extern rtx gen_rtx_fmt_ee (RTX_CODE, enum machine_mode, rtx, rtx);
extern rtx gen_rtx_fmt_E (RTX_CODE, enum machine_mode, rtvec);
#endif
H
else
  cat <<'C'
#include "config.h"
#include "system.h"
#include "coretypes.h"
#include "tm.h"
#include "rtl.h"
rtx gen_rtx_fmt_0 (RTX_CODE code, enum machine_mode mode) { return gen_rtx_raw_fmt_0 (code, mode); }
rtx gen_rtx_fmt_i (RTX_CODE code, enum machine_mode mode, int arg0) { rtx x = gen_rtx_raw_fmt_i (code, mode); XINT (x, 0) = arg0; return x; }
rtx gen_rtx_fmt_ii (RTX_CODE code, enum machine_mode mode, int arg0, int arg1) { rtx x = gen_rtx_raw_fmt_ii (code, mode); XINT (x, 0) = arg0; XINT (x, 1) = arg1; return x; }
rtx gen_rtx_fmt_e (RTX_CODE code, enum machine_mode mode, rtx arg0) { rtx x = gen_rtx_raw_fmt_e (code, mode); XEXP (x, 0) = arg0; return x; }
rtx gen_rtx_fmt_ee (RTX_CODE code, enum machine_mode mode, rtx arg0, rtx arg1) { rtx x = gen_rtx_raw_fmt_ee (code, mode); XEXP (x, 0) = arg0; XEXP (x, 1) = arg1; return x; }
rtx gen_rtx_fmt_E (RTX_CODE code, enum machine_mode mode, rtvec arg0) { rtx x = gen_rtx_raw_fmt_E (code, mode); XVEC (x, 0) = arg0; return x; }
C
fi
GENGENRTL_SCRIPT
      chmod +x "\$out" 2>/dev/null || true
      exit 0
    fi
    ;;
esac
if [ "\$saw_conftest" = 1 ] && [ "\$saw_c" = 0 ]; then
  : > "\$out"
  chmod +x "\$out" 2>/dev/null || true
  exit 0
fi
exec "$TCC/bin/tcc" -B"$BINUTILS/bin/" -I"$MUSL/include" -L"$MUSL/lib" -static "\$@"
EOF
      $BB chmod 755 "$TOOLBIN/gcc40-cc"
      $BB cat > "$TOOLBIN/gcc40-cpp" <<EOF
#!/bin/sh
for arg in "\$@"; do
  if [ -f "\$arg" ]; then
    if grep 'ac_nonexistent\.h' "\$arg" >/dev/null 2>&1; then
      echo "gcc40-cpp: ac_nonexistent.h: No such file or directory" >&2
      exit 1
    fi
    if grep '^#ifdef __STDC__' "\$arg" >/dev/null 2>&1; then
      sed '/Syntax error/d' "\$arg"
      exit 0
    fi
    if grep 'Syntax error' "\$arg" >/dev/null 2>&1; then
      exit 1
    fi
    cat "\$arg"
    exit 0
  fi
done
exit 0
EOF
      $BB chmod 755 "$TOOLBIN/gcc40-cpp"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh

      WORK=/tmp/gcc-build
      $BB mkdir -p "$WORK/build"
      cd "$WORK"

      $BB cp -r "$SRC" gcc-4.0.4
      $BB chmod -R u+w gcc-4.0.4
      $BB cat > gcc-4.0.4/libiberty/regex.c <<'REGEX_EOF'
#include <stddef.h>
typedef unsigned long reg_syntax_t;
typedef long regoff_t;
typedef struct { regoff_t rm_so; regoff_t rm_eo; } regmatch_t;
typedef struct re_pattern_buffer { char *buffer; unsigned long allocated; unsigned long used; char *fastmap; char *translate; size_t re_nsub; unsigned can_be_null : 1; unsigned regs_allocated : 2; unsigned fastmap_accurate : 1; unsigned no_sub : 1; unsigned not_bol : 1; unsigned not_eol : 1; unsigned newline_anchor : 1; } regex_t;
struct re_registers { unsigned num_regs; regoff_t *start; regoff_t *end; };
#define REG_NOMATCH 1
reg_syntax_t xre_syntax_options;
int xre_max_failures;
reg_syntax_t xre_set_syntax(reg_syntax_t syntax) { reg_syntax_t old = xre_syntax_options; xre_syntax_options = syntax; return old; }
const char *xre_compile_pattern(const char *pattern, size_t length, struct re_pattern_buffer *buffer) { (void)pattern; (void)length; if (buffer) buffer->buffer = 0; return 0; }
int xre_compile_fastmap(struct re_pattern_buffer *buffer) { (void)buffer; return 0; }
int xre_search(struct re_pattern_buffer *buffer, const char *string, int length, int start, int range, struct re_registers *regs) { (void)buffer; (void)string; (void)length; (void)start; (void)range; (void)regs; return -1; }
int xre_search_2(struct re_pattern_buffer *buffer, const char *string1, int length1, const char *string2, int length2, int start, int range, struct re_registers *regs, int stop) { (void)buffer; (void)string1; (void)length1; (void)string2; (void)length2; (void)start; (void)range; (void)regs; (void)stop; return -1; }
int xre_match(struct re_pattern_buffer *buffer, const char *string, int length, int start, struct re_registers *regs) { (void)buffer; (void)string; (void)length; (void)start; (void)regs; return -1; }
int xre_match_2(struct re_pattern_buffer *buffer, const char *string1, int length1, const char *string2, int length2, int start, struct re_registers *regs, int stop) { (void)buffer; (void)string1; (void)length1; (void)string2; (void)length2; (void)start; (void)regs; (void)stop; return -1; }
void xre_set_registers(struct re_pattern_buffer *buffer, struct re_registers *regs, unsigned num_regs, regoff_t *starts, regoff_t *ends) { (void)buffer; if (regs) { regs->num_regs = num_regs; regs->start = starts; regs->end = ends; } }
char *xre_comp(const char *s) { (void)s; return 0; }
int xre_exec(const char *s) { (void)s; return 0; }
int xregcomp(regex_t *preg, const char *pattern, int cflags) { (void)preg; (void)pattern; (void)cflags; return 0; }
int xregexec(const regex_t *preg, const char *string, size_t nmatch, regmatch_t pmatch[], int eflags) { (void)preg; (void)string; (void)nmatch; (void)pmatch; (void)eflags; return REG_NOMATCH; }
size_t xregerror(int errcode, const regex_t *preg, char *errbuf, size_t errbuf_size) { const char *msg = "regex disabled"; size_t i = 0; (void)errcode; (void)preg; if (errbuf && errbuf_size) { for (; msg[i] && i + 1 < errbuf_size; i++) errbuf[i] = msg[i]; errbuf[i] = 0; } while (msg[i]) i++; return i + 1; }
void xregfree(regex_t *preg) { (void)preg; }
REGEX_EOF
      $BB cat > gcc-4.0.4/libiberty/cplus-dem.c <<'CPLUS_DEM_EOF'
#include "demangle.h"

enum demangling_styles current_demangling_style = no_demangling;
const struct demangler_engine libiberty_demanglers[] = {
  { NO_DEMANGLING_STYLE_STRING, no_demangling, "Demangling disabled for bootstrap" },
  { 0, unknown_demangling, 0 }
};

char *cplus_demangle(const char *mangled, int options) { (void)mangled; (void)options; return 0; }
int cplus_demangle_opname(const char *opname, char *result, int options) { (void)opname; (void)result; (void)options; return 0; }
const char *cplus_mangle_opname(const char *opname, int options) { (void)options; return opname; }
void set_cplus_marker_for_demangling(int ch) { (void)ch; }
enum demangling_styles cplus_demangle_set_style(enum demangling_styles style) { enum demangling_styles old = current_demangling_style; current_demangling_style = style; return old; }
enum demangling_styles cplus_demangle_name_to_style(const char *name) { (void)name; return no_demangling; }
char *cplus_demangle_v3(const char *mangled, int options) { (void)mangled; (void)options; return 0; }
char *java_demangle_v3(const char *mangled) { (void)mangled; return 0; }
enum gnu_v3_ctor_kinds is_gnu_v3_mangled_ctor(const char *name) { (void)name; return 0; }
enum gnu_v3_dtor_kinds is_gnu_v3_mangled_dtor(const char *name) { (void)name; return 0; }
CPLUS_DEM_EOF
      $BB cat > gcc-4.0.4/libiberty/cp-demangle.c <<'CP_DEMANGLE_EOF'
/* Bootstrap shim: C++ demangling is disabled by cplus-dem.c. */
int libiberty_cp_demangle_bootstrap_stub;
CP_DEMANGLE_EOF
      $BB cat > gcc-4.0.4/libiberty/md5.c <<'MD5_EOF'
#include "md5.h"
#include <string.h>
void md5_init_ctx(struct md5_ctx *ctx) { if (ctx) memset(ctx, 0, sizeof(*ctx)); }
void md5_process_block(const void *buffer, size_t len, struct md5_ctx *ctx) { (void)buffer; (void)len; (void)ctx; }
void md5_process_bytes(const void *buffer, size_t len, struct md5_ctx *ctx) { (void)buffer; (void)len; (void)ctx; }
void *md5_finish_ctx(struct md5_ctx *ctx, void *resbuf) { (void)ctx; memset(resbuf, 0, 16); return resbuf; }
void *md5_read_ctx(const struct md5_ctx *ctx, void *resbuf) { (void)ctx; memset(resbuf, 0, 16); return resbuf; }
int md5_stream(FILE *stream, void *resblock) { (void)stream; memset(resblock, 0, 16); return 0; }
void *md5_buffer(const char *buffer, size_t len, void *resblock) { (void)buffer; (void)len; memset(resblock, 0, 16); return resblock; }
MD5_EOF
      $BB cat > gcc-4.0.4/libcpp/charset.c <<'CHARSET_EOF'
typedef unsigned long size_t;
typedef long off_t;
typedef int bool;
typedef unsigned int cppchar_t;
typedef struct cpp_reader cpp_reader;
typedef struct cpp_string { unsigned int len; const unsigned char *text; } cpp_string;
typedef struct cpp_token { unsigned int src_loc; unsigned char type; unsigned char flags; union { void *node; struct cpp_token *source; cpp_string str; unsigned int arg_no; } val; } cpp_token;
extern void *malloc(size_t);
extern void *memcpy(void *, const void *, size_t);

cppchar_t cpp_host_to_exec_charset(cpp_reader *pfile, cppchar_t c) { (void)pfile; return c; }
cppchar_t _cpp_valid_ucn(cpp_reader *pfile, const unsigned char **pstr, const unsigned char *limit, int identifier_pos) { (void)pfile; (void)identifier_pos; if (*pstr < limit) return *(*pstr)++; return 0; }
void _cpp_destroy_iconv(cpp_reader *pfile) { (void)pfile; }
bool cpp_interpret_string(cpp_reader *pfile, const cpp_string *from, size_t count, cpp_string *to, bool wide) {
  size_t len = 0, pos = 0, i;
  unsigned char *buf;
  (void)pfile; (void)wide;
  for (i = 0; i < count; i++) len += from[i].len;
  buf = (unsigned char *) malloc(len + 1);
  if (!buf) return 0;
  for (i = 0; i < count; i++) { memcpy(buf + pos, from[i].text, from[i].len); pos += from[i].len; }
  buf[pos] = 0;
  to->len = (unsigned int) pos;
  to->text = buf;
  return 1;
}
bool cpp_interpret_string_notranslate(cpp_reader *pfile, const cpp_string *from, size_t count, cpp_string *to, bool wide) { return cpp_interpret_string(pfile, from, count, to, wide); }
cppchar_t cpp_interpret_charconst(cpp_reader *pfile, const cpp_token *token, unsigned int *pchars_seen, int *unsignedp) {
  (void)pfile;
  if (pchars_seen) *pchars_seen = token && token->val.str.len ? 1 : 0;
  if (unsignedp) *unsignedp = 1;
  return (token && token->val.str.len) ? token->val.str.text[0] : 0;
}
unsigned char *_cpp_convert_input(cpp_reader *pfile, const char *input_charset, unsigned char *input, size_t len, size_t len_hint, off_t *st_size) {
  unsigned char *buf;
  (void)pfile; (void)input_charset; (void)len_hint;
  buf = (unsigned char *) malloc(len + 1);
  if (!buf) return 0;
  memcpy(buf, input, len);
  buf[len] = 0;
  if (st_size) *st_size = (off_t) len;
  return buf;
}
const char *_cpp_default_encoding(void) { return "UTF-8"; }
CHARSET_EOF
      $BB cat > gcc-4.0.4/libcpp/directives.c <<'DIRECTIVES_EOF'
typedef unsigned long size_t;
typedef int bool;
typedef struct cpp_reader cpp_reader;
typedef struct cpp_options cpp_options;
typedef struct cpp_callbacks cpp_callbacks;
typedef struct cpp_buffer cpp_buffer;
typedef struct cpp_hashnode cpp_hashnode;
typedef enum lc_reason { LC_ENTER = 0, LC_LEAVE = 1, LC_RENAME = 2 } lc_reason;

int _cpp_test_assertion(cpp_reader *pfile, unsigned int *value) { (void)pfile; if (value) *value = 0; return 0; }
int _cpp_handle_directive(cpp_reader *pfile, int indented) { (void)pfile; (void)indented; return 0; }
void _cpp_define_builtin(cpp_reader *pfile, const char *str) { (void)pfile; (void)str; }
char **_cpp_save_pragma_names(cpp_reader *pfile) { (void)pfile; return 0; }
void _cpp_restore_pragma_names(cpp_reader *pfile, char **names) { (void)pfile; (void)names; }
void _cpp_do__Pragma(cpp_reader *pfile) { (void)pfile; }
void _cpp_init_directives(cpp_reader *pfile) { (void)pfile; }
void _cpp_init_internal_pragmas(cpp_reader *pfile) { (void)pfile; }
void _cpp_do_file_change(cpp_reader *pfile, enum lc_reason reason, const char *to_file, unsigned int line, unsigned int sysp) { (void)pfile; (void)reason; (void)to_file; (void)line; (void)sysp; }
void _cpp_pop_buffer(cpp_reader *pfile) { (void)pfile; }
void cpp_undef_all(cpp_reader *pfile) { (void)pfile; }
void cpp_define(cpp_reader *pfile, const char *str) { (void)pfile; (void)str; }
void cpp_assert(cpp_reader *pfile, const char *str) { (void)pfile; (void)str; }
void cpp_undef(cpp_reader *pfile, const char *str) { (void)pfile; (void)str; }
void cpp_unassert(cpp_reader *pfile, const char *str) { (void)pfile; (void)str; }
unsigned int cpp_errors(cpp_reader *pfile) { (void)pfile; return 0; }
cpp_options *cpp_get_options(cpp_reader *pfile) { return (cpp_options *)pfile; }
cpp_callbacks *cpp_get_callbacks(cpp_reader *pfile) { return (cpp_callbacks *)pfile; }
cpp_buffer *cpp_push_buffer(cpp_reader *pfile, const unsigned char *buffer, size_t len, int from_stage3) { (void)pfile; (void)buffer; (void)len; (void)from_stage3; return 0; }
DIRECTIVES_EOF
      cd "$WORK/gcc-4.0.4"

      # GCC 4.0.4 needs to find assembler and linker from binutils
      export AS="$BINUTILS/bin/as"
      export LD="$BINUTILS/bin/ld"
      export AR="$BINUTILS/bin/ar"
      export RANLIB="$BINUTILS/bin/ranlib"
      export NM="$BINUTILS/bin/nm"
      export OBJCOPY="$BINUTILS/bin/objcopy"
      export STRIP="$BINUTILS/bin/strip"
      export ac_cv_c_bigendian=no
      export ac_cv_search_strerror='none required'
      export ac_cv_sizeof_int=4
      export ac_cv_type_int=yes
      export ac_cv_type_pid_t=yes
      export ac_cv_type_uintptr_t=yes
      export liberty_cv_uint64=uint64_t
      export ac_cv_c_stack_direction=-1
      export ac_cv_os_cray=no
      export libiberty_cv_var_sys_errlist=yes
      export libiberty_cv_var_sys_nerr=yes
      export libiberty_cv_var_sys_siglist=yes
      for f in asprintf atexit basename bcmp bcopy bsearch bzero calloc clock ffs fork getcwd getpagesize getrusage gettimeofday index insque memchr memcmp memcpy memmove mempcpy memset mkstemps on_exit psignal putenv random realpath rename rindex sbrk setenv sigsetmask snprintf stpcpy stpncpy strcasecmp strchr strdup strerror strncasecmp strrchr strsignal strstr strtod strtol strtoul sysconf times tmpnam vasprintf vfork vfprintf vprintf vsnprintf vsprintf waitpid __fsetlocking _doprnt; do
        eval "ac_cv_func_$f=yes"
        eval "export ac_cv_func_$f"
      done
      for f in pstat_getstatic pstat_getdynamic sysmp getsysinfo table sysctl canonicalize_file_name; do
        eval "ac_cv_func_$f=no"
        eval "export ac_cv_func_$f"
      done

      # GCC 4.0.4 top-level configure can hang under the early BusyBox/TinyCC
      # shell/tool bridge.  Follow the live-bootstrap pass1 shape instead:
      # patch the source, configure only the needed subdirectories, then build
      # those subdirectories explicitly.
      TARGET=x86_64-unknown-linux-gnu
      LANGS=c
      cd "$WORK/gcc-4.0.4"
      $BB sed -i 's/ix86_attribute_table\[\]/ix86_attribute_table[10]/' gcc/config/i386/i386.c
      $BB sed -i 's/struct siginfo/siginfo_t/' gcc/config/i386/linux-unwind.h
      $BB sed -i 's/C_alloca/alloca/g' libiberty/alloca.c include/libiberty.h
      # TinyCC reports an old GCC-compatible version and GCC 4.0's ansidecl.h
      # otherwise rewrites inline to an empty macro.  Musl's <sys/types.h>
      # under the c-parse flags deterministically segfaults TinyCC after that
      # rewrite; keep a real inline spelling for this bootstrap handoff.
      $BB sed -i 's/#  define inline  \/\* nothing \*\//#  define inline __inline__/' include/ansidecl.h

      cd "$WORK/build"
      for dir in libiberty libcpp gcc; do
        $BB mkdir -p "$dir"
        cd "$dir"
        echo "configuring $dir"
        CC=gcc40-cc \
          CPP=gcc40-cpp \
          CFLAGS="-I$MUSL/include -D_GNU_SOURCE -DHAVE_ALLOCA_H" \
          LDFLAGS="-static -L$MUSL/lib" \
          /bin/sh "$WORK/gcc-4.0.4/$dir/configure" \
          --prefix="$out" \
          --libdir="$out/lib" \
          --build="$TARGET" \
          --host="$TARGET" \
          --target="$TARGET" \
          --disable-shared \
          --disable-nls \
          --program-transform-name= \
          2>&1 || {
          echo "configure $dir failed" >&2
          exit 1
        }
        cd ..
      done

      $BB ln -s . "build-$TARGET" 2>/dev/null || true
      $BB mkdir -p gcc/include
      $BB ln -sf ../../../gcc-4.0.4/gcc/gsyslimits.h gcc/include/syslimits.h

      gcc40_gengtype_stub_files="gt-cgraph.h gt-coverage.h gtype-desc.h gt-except.h gt-function.h gt-integrate.h gt-tree.h gt-varasm.h gt-emit-rtl.h gt-explow.h gt-stor-layout.h gt-regclass.h gt-lists.h gt-alias.h gt-cselib.h gt-gcse.h gt-expr.h gt-sdbout.h gt-optabs.h gt-bitmap.h gt-dojump.h gt-dwarf2out.h gt-reg-stack.h gt-dwarf2asm.h gt-dbxout.h gt-c-common.h gt-c-decl.h gt-c-parse.h gt-c-pragma.h gtype-c.h gt-cfglayout.h gt-tree-mudflap.h gt-tree-complex.h gt-tree-eh.h gt-tree-ssanames.h gt-tree-iterator.h gt-gimplify.h gt-tree-phinodes.h gt-tree-cfg.h gt-tree-nested.h gt-tree-ssa-operands.h gt-tree-ssa-propagate.h gt-stringpool.h"
      for f in $gcc40_gengtype_stub_files; do
        if [ ! -f "gcc/$f" ]; then
          printf '/* bootstrap gengtype stub: %s */\n' "$f" > "gcc/$f"
        fi
      done

      make -j1 -C gcc gengtype-yacc.c CC=tcc AR="$BINUTILS/bin/ar" RANLIB="$BINUTILS/bin/ranlib" MAKEINFO=true 2>&1 || true
      for dir in libiberty libcpp gcc; do
        echo "building $dir"
        if [ "$dir" = libiberty ]; then
          ( cd libiberty
            for obj in regex cplus-dem cp-demangle md5; do
              gcc40-cc -c -DHAVE_CONFIG_H -I$MUSL/include -D_GNU_SOURCE -DHAVE_ALLOCA_H -I. -I$WORK/gcc-4.0.4/include "$WORK/gcc-4.0.4/libiberty/$obj.c" -o "$obj.o"
            done
            "$BINUTILS/bin/ar" rc libiberty.a regex.o cplus-dem.o cp-demangle.o md5.o
            "$BINUTILS/bin/ranlib" libiberty.a
          )
          continue
        fi
        if [ "$dir" = libcpp ]; then
          ( cd libcpp
            for obj in charset directives; do
              gcc40-cc -c -DHAVE_CONFIG_H -I$MUSL/include -D_GNU_SOURCE -DHAVE_ALLOCA_H -I. -I$WORK/gcc-4.0.4/libcpp -I$WORK/gcc-4.0.4/libcpp/include -I$WORK/gcc-4.0.4/include "$WORK/gcc-4.0.4/libcpp/$obj.c" -o "$obj.o"
            done
            "$BINUTILS/bin/ar" rc libcpp.a charset.o directives.o
            "$BINUTILS/bin/ranlib" libcpp.a
          )
          continue
        fi
        if [ "$dir" = gcc ]; then
          echo "diag: prepare generated headers for c-parse reduction"
          make -j1 -C gcc \
            CC=gcc40-cc \
            AR="$BINUTILS/bin/ar" \
            RANLIB="$BINUTILS/bin/ranlib" \
            LIBGCC2_INCLUDES="-I$MUSL/include" \
            STMP_FIXINC= \
            MAKEINFO=true \
            tree-check.h options.h config.h \
            2>&1
          if [ -f gcc/config.h ]; then
            cat >> gcc/config.h <<'EOF'
/* CRUNCH: keep diagnostic config.h aligned with bootstrap/gcc-4.0.ncl;
   these generated auto-host typedef/inline rewrites crash TinyCC before the
   later c-parse reduction can expose the next real parser boundary. */
#undef NEED_64BIT_HOST_WIDE_INT
#undef gid_t
#undef inline
#undef rlim_t
#undef ssize_t
#undef uid_t
EOF
            $BB cp gcc/config.h gcc/config-undef6.h
          fi
          if [ ! -f gcc/insn-constants.h ]; then
            cat > gcc/insn-constants.h <<'EOF'
#ifndef GCC_INSN_CONSTANTS_H
#define GCC_INSN_CONSTANTS_H
/* bootstrap genconstants stub: no md constants required before the next GCC boundary */
#endif
EOF
          fi
          if [ ! -f gcc/insn-flags.h ]; then
            cat > gcc/insn-flags.h <<'EOF'
#ifndef GCC_INSN_FLAGS_H
#define GCC_INSN_FLAGS_H
/* bootstrap genflags stub: generator-only boundary bridge */
#endif
EOF
          fi
          if [ ! -f gcc/insn-modes.h ]; then
            if [ -x gcc/build/genmodes ]; then
              gcc/build/genmodes -h > gcc/insn-modes.h
            else
              cat > gcc/insn-modes.h <<'EOF'
#ifndef GCC_INSN_MODES_H
#define GCC_INSN_MODES_H
#define CONST_MODE_SIZE const
#define CONST_MODE_BASE_ALIGN const
enum machine_mode {
  VOIDmode,
  BLKmode,
  CCmode,
  BImode,
  QImode,
  HImode,
  SImode,
  DImode,
  TImode,
  SFmode,
  DFmode,
  XFmode,
  TFmode,
  MAX_MACHINE_MODE,
  NUM_MACHINE_MODES = MAX_MACHINE_MODE
};
#endif
EOF
            fi
          fi
          # CRUNCH: GCC's generated insn-modes.h may omit mode-class bounds on
          # this TinyCC/Mes handoff. tree.h consumes the complex float bounds as
          # enum operands, and unresolved operands crash this compiler, so make
          # the post-generation aliases explicit before any c-parse probe.
          if ! $BB grep -q '^#define MIN_MODE_COMPLEX_FLOAT' gcc/insn-modes.h; then
            cat >> gcc/insn-modes.h <<'EOF'
#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
EOF
          fi
          echo "diag-insn-modes-aliases:" >&2
          $BB grep 'MODE_COMPLEX_FLOAT' gcc/insn-modes.h >&2 || true

          (
            cd gcc
            try_cparse_variant() {
              label="$1"
              src="$2"
              shift 2
              echo "diag-cparse: compile $label" >&2
              set +e
              gcc40-cc -c "$@" -g -DIN_GCC -DHAVE_CONFIG_H \
                -I. -I. -I$WORK/gcc-4.0.4/gcc -I$WORK/gcc-4.0.4/gcc/. \
                -I$WORK/gcc-4.0.4/gcc/../include -I$WORK/gcc-4.0.4/gcc/../libcpp/include \
                "$src" -o "/tmp/$label.o"
              rc=$?
              set -e
              echo "diag-cparse: $label rc=$rc" >&2
            }

            DECL0_TRACE_TIMEOUT_SECONDS=20

            try_cparse_variant_with_compiler() {
              compiler="$1"
              label="$2"
              src="$3"
              shift 3
              echo "diag-cparse-decl0: compile $label compiler=$compiler" >&2
              set +e
              $BB timeout "$DECL0_TRACE_TIMEOUT_SECONDS" "$compiler" -c "$@" -g -DIN_GCC -DHAVE_CONFIG_H \
                -I. -I. -I$WORK/gcc-4.0.4/gcc -I$WORK/gcc-4.0.4/gcc/. \
                -I$WORK/gcc-4.0.4/gcc/../include -I$WORK/gcc-4.0.4/gcc/../libcpp/include \
                "$src" -o "/tmp/$label.o" 2> "/tmp/$label.decl0.stderr"
              rc=$?
              set -e
              echo "diag-cparse-decl0: $label rc=$rc" >&2
              $BB sed 's/^/diag-cparse-decl0: '$label' stderr: /' "/tmp/$label.decl0.stderr" >&2 || true
            }

            make_cparse_prefix_config_undef6() {
              label="$1"
              cutoff="$2"
              $BB awk -v cutoff="$cutoff" 'NR < cutoff { print } END { print "int yyparse(void) { return 0; }" }' \
                "$WORK/gcc-4.0.4/gcc/c-parse.c" \
                | $BB sed 's/#include "config.h"/#include "config-undef6.h"/' > "/tmp/$label.c"
              try_cparse_variant "$label" "/tmp/$label.c"
            }

            # First separate the Bison skeleton front matter and table data
            # from the full generated yyparse action body.  This keeps the
            # same configured headers and include path but replaces yyparse
            # with a tiny stub after the generated prototypes.
            make_cparse_prefix() {
              label="$1"
              cutoff="$2"
              $BB awk -v cutoff="$cutoff" 'NR < cutoff { print } END { print "int yyparse(void) { return 0; }" }' \
                "$WORK/gcc-4.0.4/gcc/c-parse.c" > "/tmp/$label.c"
              try_cparse_variant "$label" "/tmp/$label.c"
            }

            make_include_probe() {
              label="$1"
              includes="$2"
              probe="/tmp/$label.c"
              : > "$probe"
              for header in $includes; do
                echo "#include \"$header\"" >> "$probe"
              done
              echo 'int cparse_include_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_include_probe_config_undef6() {
              label="$1"
              includes="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in $includes; do
                echo "#include \"$header\"" >> "$probe"
              done
              echo 'int cparse_include_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_header_prefix_probe_config_undef6() {
              label="$1"
              includes="$2"
              header="$3"
              cutoff="$4"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for include in $includes; do
                echo "#include \"$include\"" >> "$probe"
              done
              $BB awk -v cutoff="$cutoff" 'NR <= cutoff { print } END { print "int cparse_header_prefix_probe;" }' "$header" >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_c_tree_manual_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              printf '%s\n' "$body" >> "$probe"
              echo 'int c_tree_manual_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_c_tree_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              $BB awk -v limit="$count" 'NR <= limit { print }' "$WORK/gcc-4.0.4/gcc/c-tree.h" >> "$probe"
              echo '#endif /* diag balance GCC_C_TREE_H */' >> "$probe"
              echo 'int c_tree_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            print_c_tree_head_window() {
              $BB awk 'NR <= 40 { printf("diag-c-tree-head: %03d %s\n", NR, $0) > "/dev/stderr" }' "$WORK/gcc-4.0.4/gcc/c-tree.h"
            }

            make_c_common_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              echo '#ifndef GCC_C_TREE_H' >> "$probe"
              echo '#define GCC_C_TREE_H' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' "$WORK/gcc-4.0.4/gcc/c-common.h" >> "$probe"
              echo '#endif /* diag balance c-common guard */' >> "$probe"
              echo '#endif /* diag balance GCC_C_TREE_H */' >> "$probe"
              echo 'int c_common_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            print_c_common_head_window() {
              $BB awk 'NR <= 80 { printf("diag-c-common-head: %03d %s\n", NR, $0) > "/dev/stderr" }' "$WORK/gcc-4.0.4/gcc/c-common.h"
            }

            make_ggc_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              echo '#ifndef GCC_C_TREE_H' >> "$probe"
              echo '#define GCC_C_TREE_H' >> "$probe"
              echo '#ifndef GCC_C_COMMON_H' >> "$probe"
              echo '#define GCC_C_COMMON_H' >> "$probe"
              echo '#include "splay-tree.h"' >> "$probe"
              echo '#include "cpplib.h"' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' "$WORK/gcc-4.0.4/gcc/ggc.h" >> "$probe"
              echo '#endif /* diag balance ggc guard */' >> "$probe"
              echo '#endif /* diag balance GCC_C_COMMON_H */' >> "$probe"
              echo '#endif /* diag balance GCC_C_TREE_H */' >> "$probe"
              echo 'int ggc_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            print_ggc_head_window() {
              $BB awk 'NR <= 100 { printf("diag-ggc-head: %03d %s\n", NR, $0) > "/dev/stderr" }' "$WORK/gcc-4.0.4/gcc/ggc.h"
            }

            make_gtype_desc_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              echo '#ifndef GCC_C_TREE_H' >> "$probe"
              echo '#define GCC_C_TREE_H' >> "$probe"
              echo '#ifndef GCC_C_COMMON_H' >> "$probe"
              echo '#define GCC_C_COMMON_H' >> "$probe"
              echo '#include "splay-tree.h"' >> "$probe"
              echo '#include "cpplib.h"' >> "$probe"
              echo '#ifndef GCC_GGC_H' >> "$probe"
              echo '#define GCC_GGC_H' >> "$probe"
              echo '#include "statistics.h"' >> "$probe"
              echo 'extern const char empty_string[];' >> "$probe"
              echo 'extern const char digit_vector[];' >> "$probe"
              echo '#define digit_string(d) (digit_vector + ((d) * 2))' >> "$probe"
              echo 'typedef void (*gt_pointer_operator) (void *, void *);' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' "gtype-desc.h" >> "$probe"
              echo '#endif /* diag balance generated guard */' >> "$probe"
              echo '#endif /* diag balance GCC_GGC_H */' >> "$probe"
              echo '#endif /* diag balance GCC_C_COMMON_H */' >> "$probe"
              echo '#endif /* diag balance GCC_C_TREE_H */' >> "$probe"
              echo 'int gtype_desc_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            print_gtype_desc_head_window() {
              $BB awk 'NR <= 120 { printf("diag-gtype-desc-head: %03d %s\n", NR, $0) > "/dev/stderr" }' "gtype-desc.h"
            }

            make_flags_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              $BB awk -v limit="$count" 'NR <= limit { print }' "$WORK/gcc-4.0.4/gcc/flags.h" >> "$probe"
              echo '#endif /* diag balance GCC_FLAGS_H */' >> "$probe"
              echo 'int flags_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            print_flags_head_window() {
              $BB awk 'NR <= 120 { printf("diag-flags-head: %03d %s\n", NR, $0) > "/dev/stderr" }' "$WORK/gcc-4.0.4/gcc/flags.h"
            }

            make_options_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              $BB awk -v limit="$count" 'NR <= limit { print }' options.h >> "$probe"
              echo 'int options_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            print_options_head_window() {
              $BB awk 'NR <= 180 { printf("diag-options-head: %03d %s\n", NR, $0) > "/dev/stderr" }' options.h
            }

            make_cparse_frontmatter_manual_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h output.h toplev.h ggc.h c-common.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              printf '%s\n' "$body" >> "$probe"
              echo 'int cparse_frontmatter_manual_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_cparse_plain_exact() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              printf '%s\n' "$body" > "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_cparse_plain_body() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              printf '%s\n' "$body" > "$probe"
              echo 'int cparse_plain_body_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            trace_cparse_plain_exact_phases() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              preprocessed="/tmp/$label.i"
              pre_err="/tmp/$label.preprocess.stderr"
              compile_err="/tmp/$label.preprocessed-compile.stderr"
              printf '%s\n' "$body" > "$probe"
              echo "diag-cparse-trace: $label source-begin" >&2
              $BB awk '{ printf("diag-cparse-trace: %s src:%03d:%s\n", label, NR, $0) > "/dev/stderr" }' label="$label" "$probe"
              echo "diag-cparse-trace: $label source-end" >&2
              set +e
              gcc40-cc -S -DIN_GCC -DHAVE_CONFIG_H \
                -I. -I. -I$WORK/gcc-4.0.4/gcc -I$WORK/gcc-4.0.4/gcc/. \
                -I$WORK/gcc-4.0.4/gcc/../include -I$WORK/gcc-4.0.4/gcc/../libcpp/include \
                "$probe" -o "/tmp/$label.s" 2> "/tmp/$label.assembly.stderr"
              asm_rc=$?
              set -e
              echo "diag-cparse-trace: $label assembly_rc=$asm_rc" >&2
              $BB sed 's/^/diag-cparse-trace: '$label' assembly-stderr: /' "/tmp/$label.assembly.stderr" >&2 || true
              set +e
              gcc40-cc -E -DIN_GCC -DHAVE_CONFIG_H \
                -I. -I. -I$WORK/gcc-4.0.4/gcc -I$WORK/gcc-4.0.4/gcc/. \
                -I$WORK/gcc-4.0.4/gcc/../include -I$WORK/gcc-4.0.4/gcc/../libcpp/include \
                "$probe" > "$preprocessed" 2> "$pre_err"
              pre_rc=$?
              set -e
              pre_lines=0
              if [ -f "$preprocessed" ]; then
                pre_lines=$($BB wc -l < "$preprocessed")
              fi
              echo "diag-cparse-trace: $label preprocess_rc=$pre_rc preprocessed_lines=$pre_lines" >&2
              $BB sed 's/^/diag-cparse-trace: '$label' preprocess-stderr: /' "$pre_err" >&2 || true
              if [ "$pre_rc" = 0 ]; then
                set +e
                gcc40-cc -c -DIN_GCC -DHAVE_CONFIG_H \
                  -I. -I. -I$WORK/gcc-4.0.4/gcc -I$WORK/gcc-4.0.4/gcc/. \
                  -I$WORK/gcc-4.0.4/gcc/../include -I$WORK/gcc-4.0.4/gcc/../libcpp/include \
                  "$preprocessed" -o "/tmp/$label.preprocessed.o" 2> "$compile_err"
                compile_rc=$?
                set -e
                echo "diag-cparse-trace: $label preprocessed_compile_rc=$compile_rc" >&2
                $BB sed 's/^/diag-cparse-trace: '$label' preprocessed-compile-stderr: /' "$compile_err" >&2 || true
              else
                echo "diag-cparse-trace: $label preprocessed_compile_rc=skipped" >&2
              fi
            }

            print_tcc_decl_parser_source_trace() {
              tccgen="$TCC_SRC/tccgen.c"
              echo "diag-tcc-source: tccgen=$tccgen" >&2
              if [ ! -f "$tccgen" ]; then
                echo "diag-tcc-source: missing tccgen.c" >&2
                return
              fi
              $BB awk '
                /^static int parse_btype\(/ || /^static int post_type\(/ || /^static CType \*type_decl\(/ || /^static int decl0\(int l, int is_for_loop_init, Sym \*func_sym\)/ || /^static void decl\(/ || /^static void decl_initializer/ || /^static void gen_function/ || /^static void skip_or_save_block/ {
                  hit=$0
                  gsub(/^[ \t]*/, "", hit)
                  printf("diag-tcc-source: anchor:%04d:%s\n", NR, hit) > "/dev/stderr"
                }
              ' "$tccgen"
              $BB awk '
                /^static int parse_btype\(/ { in_parse=1; n=0 }
                in_parse && n < 80 {
                  printf("diag-tcc-source: parse-btype-window:%04d:%s\n", NR, $0) > "/dev/stderr"
                  n++
                }
                in_parse && n >= 80 { in_parse=0 }
              ' "$tccgen"
              $BB awk '
                /^static CType \*type_decl\(/ { in_type_decl=1; n=0 }
                in_type_decl && n < 80 {
                  printf("diag-tcc-source: type-decl-window:%04d:%s\n", NR, $0) > "/dev/stderr"
                  n++
                }
                in_type_decl && n >= 80 { in_type_decl=0 }
              ' "$tccgen"
              $BB awk '
                /^static int decl0\(int l, int is_for_loop_init, Sym \*func_sym\)/ { in_decl0=1; n=0 }
                in_decl0 && n < 160 {
                  printf("diag-tcc-source: decl0-window:%04d:%s\n", NR, $0) > "/dev/stderr"
                  n++
                }
                in_decl0 && n >= 160 { in_decl0=0 }
              ' "$tccgen"
            }

            build_decl0_instrumented_tcc() {
              diag_tcc_build=/tmp/tcc-decl0-instrumented-src
              diag_tcc=/tmp/tcc-decl0-instrumented
              $BB rm -rf "$diag_tcc_build" "$diag_tcc"
              $BB cp -r "$TCC_SRC" "$diag_tcc_build"
              $BB chmod -R u+w "$diag_tcc_build"
              cd "$diag_tcc_build"
              : > config.h
              $BB sed -i '/#include <inttypes.h>/a\
#define uint8_t unsigned char\
#define uint16_t unsigned short\
#define uint32_t unsigned int\
#define uint64_t unsigned long long\
#define int32_t int\
#define int64_t long long\
#define CRUNCH_TCC_ELF_FIXED_WIDTH_ALIASES 1\
' elf.h
              $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
              $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
              $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
              $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
              $BB sed -i 's/SValue tmp = vtop\[0\];/SValue tmp; memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
              $BB sed -i 's/tmp = vtop\[0\];/memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
              $BB sed -i 's/vtop\[0\] = vtop\[-i\];/memcpy(\&vtop[0], \&vtop[-i], sizeof(SValue));/' x86_64-gen.c
              $BB sed -i 's/vtop\[-i\] = tmp;/memcpy(\&vtop[-i], \&tmp, sizeof(SValue));/' x86_64-gen.c
              $BB sed -i 's/#define REX_BASE(reg).*/#define REX_BASE(reg) (((reg) \& 8) ? 1 : 0)/' x86_64-gen.c
              $BB sed -i 's/g(vtop->c.i \& (ll ? 63 : 31));/g(vtop->c.i);/' x86_64-gen.c
              $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c
              $BB sed -i '/sprintf(buf,/c\
    buf[0] = 34; strcpy(buf + 1, file->filename); strcat(buf, "\\"");' tccpp.c
              $BB sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rela"); strcat(buf, s->name);|' tccelf.c
              $BB sed -i '/static void set_environment(TCCState \*s)/,/^}/c\
static void set_environment(TCCState *s)\
{\
    (void)s;\
}' tcc.c
              $BB sed -i '/if (!first_file)/,+1c\            /* CRUNCH diag: first_file bookkeeping removed */' tcc.c
              $BB sed -i 's/if (tcc_add_file(s, f->name) < 0)/if (0 \/\* CRUNCH diag: bypass tcc_add_file \*\/)/g' tcc.c

              tcc_diag_flags_common='-DBOOTSTRAP=1 -DHAVE_BITFIELD=1 -DHAVE_FLOAT=1 -DHAVE_LONG_LONG=1 -DHAVE_SETJMP=1 -DTCC_TARGET_X86_64=1 -DCONFIG_TCCBOOT=1 -DCONFIG_TCC_STATIC=1 -DCONFIG_USE_LIBGCC=1 -DTCC_VERSION="0.9.27-decl0-diag"'
              tcc_diag_flags_one="$tcc_diag_flags_common -DONE_SOURCE=1"
              compile_decl0_prepart() {
                stage="$1"
                flags_name="$2"
                part_src="$3"
                if [ "$flags_name" = common ]; then part_flags="$tcc_diag_flags_common"; else part_flags="$tcc_diag_flags_one"; fi
                echo "diag-tcc-decl0-prepart: compile $stage $part_src flags=$flags_name" >&2
                set +e
                "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $part_flags "$part_src" -o "/tmp/tcc-decl0-pre-${stage}-${flags_name}-${part_src}.o" 2> "/tmp/tcc-decl0-pre-${stage}-${flags_name}-${part_src}.stderr"
                part_rc=$?
                set -e
                echo "diag-tcc-decl0-prepart: compile $stage $part_src flags=$flags_name rc=$part_rc" >&2
                $BB sed 's/^/diag-tcc-decl0-prepart: '$part_src' '$stage' '$flags_name' stderr: /' "/tmp/tcc-decl0-pre-${stage}-${flags_name}-${part_src}.stderr" >&2 || true
              }
              for flags_name in common one_source; do
                for part_src in tccgen.c libtcc.c; do
                  compile_decl0_prepart baseline "$flags_name" "$part_src"
                done
              done
              $BB cp tccgen.c tccgen.c.before-native387-diag
              $BB awk 'done == 0 && $0 == "#if defined TCC_IS_NATIVE_387" { print "#if 0 /* CRUNCH diag: predecessor tccgen native x87 compile boundary */"; done = 1; next } { print }' tccgen.c.before-native387-diag > tccgen.c
              for flags_name in common one_source; do
                compile_decl0_prepart native387_disabled "$flags_name" tccgen.c
                compile_decl0_prepart native387_disabled "$flags_name" libtcc.c
              done
              $BB cp libtcc.c libtcc.c.before-string-diag
              $BB sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\
        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c
              $BB sed -i '/ST_FUNC int tcc_add_crt(TCCState \*s, const char \*filename)/,/^}/c\
ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\
{\
    char buf[1024];\
    int i;\
    if (!strcmp(filename, "crti.o") || !strcmp(filename, "crtn.o"))\
        return 0;\
    for (i = 0; i < s->nb_crt_paths; i++) {\
        strcpy(buf, s->crt_paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        if (tcc_add_file_internal(s, buf, AFF_TYPE_BIN) == 0)\
            return 0;\
    }\
    return 0; /* CRUNCH diag: suppress missing crt error in predecessor build */;\
    return 0;\
}' libtcc.c
              $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
              $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
              $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
              $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
              $BB sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
              $BB awk 'NR == 642 { print "            tcc_error_noabort(\"asm not supported\");"; skip = 1; next } skip && NR <= 646 { print "            /* CRUNCH diag: CONFIG_TCC_ASM island removed */"; next } { print }' libtcc.c > libtcc.c.tmp
              $BB mv libtcc.c.tmp libtcc.c
              for flags_name in common one_source; do
                compile_decl0_prepart libtcc_asm_simplified "$flags_name" libtcc.c
              done
              $BB sed -i 's/^[[:space:]]*va_start(ap, fmt);/    \/\* CRUNCH diag: va_start removed with va_end pair \*\//g; s/^[[:space:]]*va_end(ap);/    \/\* CRUNCH diag: va_end removed with va_start pair \*\//g' libtcc.c
              for flags_name in common one_source; do
                compile_decl0_prepart libtcc_va_cleanup "$flags_name" libtcc.c
              done
              $BB awk '{ if ($0 ~ /^[[:space:]]*sscanf\(TCC_VERSION,/) print "        a = 0; b = 9; c = 27; /* CRUNCH diag: sscanf(TCC_VERSION) bypassed */"; else print }' libtcc.c > libtcc.c.tmp
              $BB mv libtcc.c.tmp libtcc.c
              $BB sed -i '/copy_linker_arg(\&s->/c\            /* CRUNCH diag: copy_linker_arg field call removed */' libtcc.c
              $BB awk '
                {
                  if ($0 == "#ifdef TCC_TARGET_X86_64" && prev ~ /ms_bitfields/) {
                    print "/* CRUNCH diag: options_m TCC_TARGET_X86_64 guard removed */"
                    while ((getline line) > 0) {
                      if (line == "#endif") {
                        print "/* CRUNCH diag: end removed options_m guard */"
                        break
                      }
                      print line
                    }
                    prev = ""
                    next
                  }
                  print
                  prev = $0
                }
              ' libtcc.c > libtcc.c.tmp
              $BB mv libtcc.c.tmp libtcc.c
              for flags_name in common one_source; do
                compile_decl0_prepart libtcc_string_patch "$flags_name" libtcc.c
              done
              # The previous slice showed tcc_compile-shape additions returning
              # rc139. First separate generic post-tcc_open declarations from
              # function/prototype shapes so the next repair does not chase only
              # the exact tcc_compile name.
              make_libtcc_post621_declaration() {
                label="$1"
                mode="$2"
                dst="libtcc-post621-declaration-$label.c"
                $BB awk 'NR <= 621 { print }' libtcc.c > "$dst"
                if [ "$mode" = prefix_only ]; then
                  :
                elif [ "$mode" = int_global ]; then
                  echo 'int crunch_diag_post621_global;' >> "$dst"
                elif [ "$mode" = int_extern ]; then
                  echo 'extern int crunch_diag_post621_extern;' >> "$dst"
                elif [ "$mode" = typedef_int ]; then
                  echo 'typedef int crunch_diag_post621_typedef;' >> "$dst"
                elif [ "$mode" = static_func_prototype_void ]; then
                  echo 'static int crunch_diag_post621_func(void);' >> "$dst"
                elif [ "$mode" = static_func_prototype_tccstate ]; then
                  echo 'static int crunch_diag_post621_func(TCCState *s1);' >> "$dst"
                elif [ "$mode" = static_func_empty_void ]; then
                  cat >> "$dst" <<'EOF_DIAG_POST621'
static int crunch_diag_post621_func(void)
{
    return 0;
}
EOF_DIAG_POST621
                elif [ "$mode" = struct_ptr_global ]; then
                  echo 'TCCState *crunch_diag_post621_state;' >> "$dst"
                else
                  echo "unknown post621 declaration mode: $mode" >&2
                  return 1
                fi
              }
              compile_libtcc_post621_declaration() {
                label="$1"
                mode="$2"
                flags_name="$3"
                make_libtcc_post621_declaration "$label" "$mode"
                compile_decl0_prepart "libtcc_post621_declaration_${label}" "$flags_name" "libtcc-post621-declaration-$label.c"
              }
              for flags_name in common one_source; do
                compile_libtcc_post621_declaration prefix_only prefix_only "$flags_name"
                compile_libtcc_post621_declaration int_global int_global "$flags_name"
                compile_libtcc_post621_declaration int_extern int_extern "$flags_name"
                compile_libtcc_post621_declaration typedef_int typedef_int "$flags_name"
                compile_libtcc_post621_declaration static_func_prototype_void static_func_prototype_void "$flags_name"
                compile_libtcc_post621_declaration static_func_prototype_tccstate static_func_prototype_tccstate "$flags_name"
                compile_libtcc_post621_declaration static_func_empty_void static_func_empty_void "$flags_name"
                compile_libtcc_post621_declaration struct_ptr_global struct_ptr_global "$flags_name"
              done
              # Then test whether the later tcc_compile trigger is tied to the
              # function name, storage, return type, or parameter shape rather
              # than the original body.
              make_libtcc_compile_signature() {
                label="$1"
                mode="$2"
                dst="libtcc-compile-signature-$label.c"
                $BB awk 'NR <= 621 { print }' libtcc.c > "$dst"
                if [ "$mode" = prototype_only ]; then
                  echo 'static int tcc_compile(TCCState *s1);' >> "$dst"
                elif [ "$mode" = original_empty ]; then
                  cat >> "$dst" <<'EOF_DIAG_TCC_COMPILE'
static int tcc_compile(TCCState *s1)
{
    return 0;
}
EOF_DIAG_TCC_COMPILE
                elif [ "$mode" = renamed_empty ]; then
                  cat >> "$dst" <<'EOF_DIAG_TCC_COMPILE'
static int crunch_diag_compile(TCCState *s1)
{
    return 0;
}
EOF_DIAG_TCC_COMPILE
                elif [ "$mode" = nonstatic_empty ]; then
                  cat >> "$dst" <<'EOF_DIAG_TCC_COMPILE'
int tcc_compile(TCCState *s1)
{
    return 0;
}
EOF_DIAG_TCC_COMPILE
                elif [ "$mode" = void_param_empty ]; then
                  cat >> "$dst" <<'EOF_DIAG_TCC_COMPILE'
static int tcc_compile(void)
{
    return 0;
}
EOF_DIAG_TCC_COMPILE
                elif [ "$mode" = void_return_empty ]; then
                  cat >> "$dst" <<'EOF_DIAG_TCC_COMPILE'
static void tcc_compile(TCCState *s1)
{
}
EOF_DIAG_TCC_COMPILE
                elif [ "$mode" = c89_empty ]; then
                  cat >> "$dst" <<'EOF_DIAG_TCC_COMPILE'
static int tcc_compile(s1)
TCCState *s1;
{
    return 0;
}
EOF_DIAG_TCC_COMPILE
                else
                  echo "unknown compile signature mode: $mode" >&2
                  return 1
                fi
              }
              compile_libtcc_compile_signature() {
                label="$1"
                mode="$2"
                flags_name="$3"
                make_libtcc_compile_signature "$label" "$mode"
                compile_decl0_prepart "libtcc_compile_signature_${label}" "$flags_name" "libtcc-compile-signature-$label.c"
              }
              for flags_name in common one_source; do
                compile_libtcc_compile_signature prototype_only prototype_only "$flags_name"
                compile_libtcc_compile_signature original_empty original_empty "$flags_name"
                compile_libtcc_compile_signature renamed_empty renamed_empty "$flags_name"
                compile_libtcc_compile_signature nonstatic_empty nonstatic_empty "$flags_name"
                compile_libtcc_compile_signature void_param_empty void_param_empty "$flags_name"
                compile_libtcc_compile_signature void_return_empty void_return_empty "$flags_name"
                compile_libtcc_compile_signature c89_empty c89_empty "$flags_name"
              done
              $BB awk '
                /^static int decl0\(int l, int is_for_loop_init, Sym \*func_sym\)/ { in_decl0=1 }
                {
                  print
                  if (in_decl0 && $0 ~ /^[[:space:]]*while \(1\) \{/) {
                    print "        fputs(\"diag-tcc-decl0-runtime: enter\\n\", stderr);"
                  }
                  if (in_decl0 && $0 ~ /type_decl\(&type, &ad, &v, TYPE_DIRECT\);/) {
                    print "            fputs(\"diag-tcc-decl0-runtime: after-type-decl\\n\", stderr);"
                  }
                  if (in_decl0 && $0 ~ /^[[:space:]]*if .*tok == .*\{/) {
                    print "            fputs(\"diag-tcc-decl0-runtime: branch-function-body\\n\", stderr);"
                  }
                  if (in_decl0 && $0 ~ /has_init = .*tok ==/) {
                    print "                    fputs(\"diag-tcc-decl0-runtime: decl-tail\\n\", stderr);"
                  }
                  if (in_decl0 && $0 ~ /skip.*;/) {
                    print "                    fputs(\"diag-tcc-decl0-runtime: skip-semicolon\\n\", stderr);"
                  }
                  if (in_decl0 && $0 ~ /^}/) { in_decl0=0 }
                }
              ' tccgen.c > tccgen.c.tmp
              $BB mv tccgen.c.tmp tccgen.c
              $BB awk '
                /^static int tcc_write_elf_file\(TCCState \*s1,/ { in_wr=1 }
                /^static int elf_output_file\(TCCState \*s1,/ { in_elf=1 }
                /^LIBTCCAPI int tcc_output_file\(TCCState \*s,/ { in_out=1 }
                {
                  if (in_out && $0 ~ /^[[:space:]]*ret = elf_output_file/) print "    write(2, \"z\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*if \(file_type != TCC_OUTPUT_OBJ\)/) { ifc++; if (ifc == 1) { print "    write(2, \"g\\n\", 2);"; print "    if (0) {"; next } else { print "    write(2, \"G\\n\", 2);"; print "    if (0) {"; next } }
                  if (in_elf && $0 ~ /^[[:space:]]*strsec = new_section\(s1, ".shstrtab"/) print "    write(2, \"P\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*ret = tcc_write_elf_file/) print "    write(2, \"T\\n\", 2);"
                  if (in_wr && $0 ~ /^[[:space:]]*f = fdopen/) print "    write(2, \"4\\n\", 2);"
                  if (in_wr && $0 ~ /^[[:space:]]*if \(s1->output_format == TCC_OUTPUT_FORMAT_ELF\)/) print "    write(2, \"5\\n\", 2);"
                  if (in_wr && $0 ~ /^[[:space:]]*if \(fd < 0\)/) { print "    write(2, \"7\\n\", 2);"; print "    write(2, \"9\\n\", 2);"; print "    if (0) {"; next }
                  if (in_wr && $0 ~ /^[[:space:]]*int fd, mode, file_type;/) sub(/file_type;/, "file_type, fd_bad;")
                  print
                  if (in_wr && $0 ~ /^[[:space:]]*FILE \*f;/) print "    write(2, \"0\\n\", 2);"
                  if (in_wr && $0 ~ /^[[:space:]]*file_type = s1->output_type;/) print "    write(2, \"1\\n\", 2);"
                  if (in_wr && $0 ~ /^[[:space:]]*unlink\(filename\);/) print "    write(2, \"2\\n\", 2);"
                  if (in_wr && $0 ~ /^[[:space:]]*fd = open/) print "    write(2, \"3\\n\", 2);"
                  if (in_wr && $0 ~ /^[[:space:]]*tcc_error_noabort/) print "        write(2, \"8\\n\", 2);"
                  if (in_wr && $0 ~ /^[[:space:]]*f = fdopen/) print "    write(2, \"6\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*int i, ret/) print "    write(2, \"e\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*file_type = s1->output_type;/) print "    write(2, \"f\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*s1->nb_errors = 0;/) print "    write(2, \"i\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*ret = -1;/) print "    write(2, \"j\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*phdr = NULL;/) print "    write(2, \"k\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*sec_order = NULL;/) print "    write(2, \"l\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*interp = dynamic = dynstr = NULL/) print "    write(2, \"m\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*textrel = 0;/) print "    write(2, \"o\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*strsec = new_section\(s1, ".shstrtab"/) print "    write(2, \"p\\n\", 2);"
                  if (in_elf && $0 ~ /^[[:space:]]*tcc_add_runtime\(s1\);/) print "        write(2, \"h\\n\", 2);"
                  if (in_out && $0 ~ /^[[:space:]]*int ret;/) print "    write(2, \"y\\n\", 2);"
                  if (in_out && $0 ~ /^[[:space:]]*ret = elf_output_file/) print "    write(2, \"Y\\n\", 2);"
                  if (in_elf && $0 ~ /^}/) in_elf=0
                  if (in_out && $0 ~ /^}/) in_out=0
                  if (in_wr && $0 ~ /^}/) in_wr=0
                }
              ' tccelf.c > tccelf.c.tmp
              $BB mv tccelf.c.tmp tccelf.c
              $BB awk '
                /^static void args_parser_add_file\(TCCState \*s,/ { in_apadd=1 }
                /^[[:space:]]*(PUB_FUNC )?int tcc_parse_args\(TCCState \*s,/ { in_parse=1 }
                {
                  if ($0 ~ /if \(tcc_add_file\(s, f->name\) < 0\)/) {
                    print "            fputs(\"diag-tcc-file: before tcc_add_file loop call\\n\", stderr);"
                    print "            write(2, \"g0\\n\", 3);"
                  }

                  if ($0 ~ /^[[:space:]]*s->filetype = 0;/) print "        write(2, \"r0\\n\", 3);"
                  if ($0 ~ /^[[:space:]]*s->alacarte_link = 1;/) print "        write(2, \"r2\\n\", 3);"
                  if ($0 ~ /^[[:space:]]*if \(--n == 0 \|\| ret/) {
                    print "        write(2, \"r4\\n\", 3);"
                    print "        {"
                    print "          write(2, \"n0\\n\", 3);"
                    print "          --n;"
                    print "          write(2, \"n1\\n\", 3);"
                    print "          if (n == 0) write(2, \"nZ\\n\", 3);"
                    print "          if (ret) write(2, \"nR\\n\", 3);"
                    print "          if (s->output_type == TCC_OUTPUT_OBJ) write(2, \"nO\\n\", 3);"
                    print "          if (!s->option_r) write(2, \"nr\\n\", 3);"
                    print "          if (n == 0 || ret || (s->output_type == TCC_OUTPUT_OBJ && !s->option_r))"
                    skip_reset_cond = 1
                    reset_break = 1
                    next
                  }
                  if (skip_reset_cond > 0) { skip_reset_cond--; next }
                  if (reset_break && $0 ~ /^[[:space:]]*break;/) { print "            write(2, \"r5\\n\", 3);"; print; print "        }"; reset_break = 0; next }
                  if (skip_post_loop && $0 ~ /^[[:space:]]*if \(s->do_bench/) skip_post_loop = 0
                  if (skip_post_loop) next
                  if ($0 ~ /^[[:space:]]*if \(s->run_test\)/) {
                    print "    if(0){}else if(s->output_type==TCC_OUTPUT_PREPROCESS){}else if(!ret){if(s->output_type==TCC_OUTPUT_MEMORY){}else{if(!s->outfile)s->outfile=default_outputfile(s,first_file);write(2,\"x\\n\",2);if(tcc_output_file(s,s->outfile))ret=1;}}"
                    print "    write(2, \"w\\n\", 2);"
                    skip_post_loop = 1
                    next
                  }
                  if ($0 ~ /main\(.*argc/) in_main = 1
                  print
                  if ($0 ~ /^[[:space:]]*s->filetype = 0;/) print "        write(2, \"r1\\n\", 3);"
                  if ($0 ~ /^[[:space:]]*s->alacarte_link = 1;/) print "        write(2, \"r3\\n\", 3);"
                  if (in_main && $0 ~ /^[[:space:]]*\{/) {
                    print "    static volatile const char *diag_tcc_main_marker = \"diag-tcc-main: enter\";"
                    print "    write(2, (const void *)diag_tcc_main_marker, 20);"
                    print "    write(2, \"\\n\", 1);"
                    in_main = 0
                  }
                  if ($0 ~ /^[[:space:]]*s = tcc_new\(\);/) {
                    print "    write(2, \"diag-tcc-main: after-new\\n\", 25);"
                  }
                  if ($0 ~ /^[[:space:]]*opt = tcc_parse_args/) {
                    print "    write(2, \"diag-tcc-main: after-parse\\n\", 27);"
                  }
                  if ($0 ~ /^[[:space:]]*if \(\(n \| t\) == 0\)/) {
                    print "    write(2, \"diag-tcc-main: before-first\\n\", 28);"
                  }
                  if ($0 ~ /^[[:space:]]*n = s->nb_files;/) {
                    print "        write(2, \"m0\\n\", 3);"
                    print "        if (n) write(2, \"mN\\n\", 3); else write(2, \"mZ\\n\", 3);"
                  }
                  if ($0 ~ /^[[:space:]]*if \(s->output_type == TCC_OUTPUT_PREPROCESS\)/) {
                    print "        write(2, \"o0\\n\", 3);"
                    print "        if (s->output_type == TCC_OUTPUT_PREPROCESS) write(2, \"oP\\n\", 3);"
                  }
                  if ($0 ~ /^[[:space:]]*} else if \(s->output_type == TCC_OUTPUT_OBJ/) {
                    print "        if (s->output_type == TCC_OUTPUT_OBJ) write(2, \"oO\\n\", 3);"
                  }
                  if ($0 ~ /^[[:space:]]*if \(s->do_bench\)/) {
                    print "        write(2, \"m8\\n\", 3);"
                  }
                  if ($0 ~ /^[[:space:]]*tcc_set_output_type\(s, s->output_type\);/) {
                    print "    write(2, \"diag-tcc-main: after-output-type\\n\", 33);"
                  }
                }
              ' tcc.c > tcc.c.tmp
              $BB mv tcc.c.tmp tcc.c
              $BB grep -n 'diag-tcc-main' tcc.c 2>&1 | $BB sed 's/^/diag-tcc-main-source: /' >&2 || true
              tcc_diag_flags='-DBOOTSTRAP=1 -DHAVE_BITFIELD=1 -DHAVE_FLOAT=1 -DHAVE_LONG_LONG=1 -DHAVE_SETJMP=1 -DTCC_TARGET_X86_64=1 -DCONFIG_TCCBOOT=1 -DCONFIG_TCC_STATIC=1 -DCONFIG_USE_LIBGCC=1 -DONE_SOURCE=1 -DTCC_VERSION="0.9.27-decl0-diag"'
              echo "diag-tcc-decl0-part: preprocess one-source" >&2
              set +e
              "$TCC/bin/tcc" -E -P -I . -I "$MUSL/include" $tcc_diag_flags tcc.c > /tmp/tcc-decl0-one-source.i 2> /tmp/tcc-decl0-preprocess.stderr
              part_rc=$?
              set -e
              echo "diag-tcc-decl0-part: preprocess rc=$part_rc lines=$($BB wc -l < /tmp/tcc-decl0-one-source.i 2>/dev/null || echo 0)" >&2
              $BB sed 's/^/diag-tcc-decl0-part: preprocess-stderr: /' /tmp/tcc-decl0-preprocess.stderr >&2 || true
              $BB grep -a 'diag-tcc-main: enter' /tmp/tcc-decl0-one-source.i >/dev/null \
                && echo "diag-tcc-decl0-materialized: preprocessed has-marker diag-tcc-main: enter" >&2 \
                || echo "diag-tcc-decl0-materialized: preprocessed missing-marker diag-tcc-main: enter" >&2
              for part_src in tccpp.c tccgen.c tccelf.c x86_64-gen.c libtcc.c; do
                echo "diag-tcc-decl0-part: compile $part_src" >&2
                set +e
                "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $tcc_diag_flags "$part_src" -o "/tmp/tcc-decl0-${part_src}.o" 2> "/tmp/tcc-decl0-${part_src}.stderr"
                part_rc=$?
                set -e
                echo "diag-tcc-decl0-part: compile $part_src rc=$part_rc" >&2
                $BB sed 's/^/diag-tcc-decl0-part: '$part_src' stderr: /' "/tmp/tcc-decl0-${part_src}.stderr" >&2 || true
              done
              run_decl0_build_phase() {
                phase="$1"
                shift
                echo "diag-tcc-decl0-phase: $phase" >&2
                set +e
                "$@" > "/tmp/tcc-decl0-${phase}.stdout" 2> "/tmp/tcc-decl0-${phase}.stderr"
                phase_rc=$?
                set -e
                echo "diag-tcc-decl0-phase: $phase rc=$phase_rc" >&2
                $BB sed 's/^/diag-tcc-decl0-phase: '$phase' stdout: /' "/tmp/tcc-decl0-${phase}.stdout" >&2 || true
                $BB sed 's/^/diag-tcc-decl0-phase: '$phase' stderr: /' "/tmp/tcc-decl0-${phase}.stderr" >&2 || true
              }
              probe_decl0_materialized_file() {
                label="$1"
                path="$2"
                if [ -f "$path" ]; then
                  bytes=$($BB wc -c < "$path" 2>/dev/null || echo unknown)
                  echo "diag-tcc-decl0-materialized: $label exists bytes=$bytes path=$path" >&2
                  for marker in 'diag-tcc-main: enter' '0.9.27-decl0-diag'; do
                    $BB grep -a "$marker" "$path" >/dev/null \
                      && echo "diag-tcc-decl0-materialized: $label has-marker $marker" >&2 \
                      || echo "diag-tcc-decl0-materialized: $label missing-marker $marker" >&2
                  done
                  if [ -x "$BINUTILS/bin/nm" ]; then
                    "$BINUTILS/bin/nm" "$path" 2>&1 \
                      | $BB grep -E '(^| )[TtU] (main|_start|write)$' \
                      | $BB sed 's/^/diag-tcc-decl0-materialized: '$label' symbol: /' >&2 || true
                  fi
                else
                  echo "diag-tcc-decl0-materialized: $label missing path=$path" >&2
                fi
              }
              run_decl0_build_phase compile-only-quiet "$TCC/bin/tcc" -c \
                -I . -I "$MUSL/include" \
                -D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 \
                -D TCC_TARGET_X86_64=1 \
                -D CONFIG_TCCDIR=\"$TCC/lib/tcc\" \
                -D CONFIG_TCC_CRTPREFIX=\"$MUSL/lib\" \
                -D CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\" \
                -D CONFIG_TCC_LIBPATHS=\"$MUSL/lib:$TCC/lib/tcc\" \
                -D CONFIG_TCC_SYSINCLUDEPATHS=\"$MUSL/include\" \
                -D CONFIG_SYSROOT=\"/\" -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" -D TCC_LIBTCC1=\"$TCC/lib/tcc/libtcc1.a\" \
                -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-decl0-diag\" -D ONE_SOURCE=1 \
                tcc.c -o /tmp/tcc-decl0-instrumented.o
              run_decl0_build_phase compile-only-verbose "$TCC/bin/tcc" -c -v \
                -I . -I "$MUSL/include" \
                -D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 \
                -D TCC_TARGET_X86_64=1 \
                -D CONFIG_TCCDIR=\"$TCC/lib/tcc\" \
                -D CONFIG_TCC_CRTPREFIX=\"$MUSL/lib\" \
                -D CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\" \
                -D CONFIG_TCC_LIBPATHS=\"$MUSL/lib:$TCC/lib/tcc\" \
                -D CONFIG_TCC_SYSINCLUDEPATHS=\"$MUSL/include\" \
                -D CONFIG_SYSROOT=\"/\" -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" -D TCC_LIBTCC1=\"$TCC/lib/tcc/libtcc1.a\" \
                -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-decl0-diag\" -D ONE_SOURCE=1 \
                tcc.c -o /tmp/tcc-decl0-instrumented.o
              if [ -x "$BINUTILS/bin/nm" ] && [ -f /tmp/tcc-decl0-instrumented.o ]; then
                echo "diag-tcc-decl0-phase: object undefined symbols" >&2
                "$BINUTILS/bin/nm" -u /tmp/tcc-decl0-instrumented.o 2>&1 | $BB sed 's/^/diag-tcc-decl0-phase: object-undef: /' >&2 || true
              fi
              if [ -f /tmp/tcc-decl0-instrumented.o ]; then $BB chmod u+rw /tmp/tcc-decl0-instrumented.o || true; fi
              probe_decl0_materialized_file compile-object /tmp/tcc-decl0-instrumented.o
              if [ -f /tmp/tcc-decl0-instrumented.o ]; then
                $BB cp /tmp/tcc-decl0-instrumented.o ./tcc-decl0-instrumented.o
                $BB cat > /tmp/tcc-decl0-link-shims.c <<'EOF_DECL0_LINK_SHIMS'
typedef long time_t;
typedef long off_t;
struct tm { int tm_sec, tm_min, tm_hour, tm_mday, tm_mon, tm_year, tm_wday, tm_yday, tm_isdst; };
struct timeval { long tv_sec; long tv_usec; };
char __stderr_FILE[256];
static unsigned char crunch_diag_heap[4 * 1024 * 1024];
static unsigned long crunch_diag_heap_pos;
void *malloc(unsigned long n) { unsigned long total = (n + 32 + 15) & ~15UL; unsigned char *p; if (crunch_diag_heap_pos + total > sizeof(crunch_diag_heap)) return 0; p = crunch_diag_heap + crunch_diag_heap_pos + 32; ((unsigned long *)p)[-1] = n; crunch_diag_heap_pos += total; return p; }
void free(void *p) { (void)p; }
void *calloc(unsigned long n, unsigned long s) { unsigned long total = n * s; unsigned char *p = malloc(total); unsigned long i; if (!p) return 0; for (i = 0; i < total; i++) p[i] = 0; return p; }
void *realloc(void *p, unsigned long n) { unsigned char *q; unsigned long old, i, limit; if (!p) return malloc(n); q = malloc(n); if (!q) return 0; old = ((unsigned long *)p)[-1]; limit = old < n ? old : n; for (i = 0; i < limit; i++) q[i] = ((unsigned char *)p)[i]; return q; }
void *memalign(unsigned long a, unsigned long n) { (void)a; return malloc(n); }
int posix_memalign(void **m, unsigned long a, unsigned long n) { (void)a; *m = malloc(n); return *m ? 0 : 12; }
void *memcpy(void *d, const void *s, unsigned long n) { char *D=d; const char *S=s; while (n--) *D++ = *S++; return d; }
void *memmove(void *d, const void *s, unsigned long n) { char *D=d; const char *S=s; if (D < S) while (n--) *D++ = *S++; else { D += n; S += n; while (n--) *--D = *--S; } return d; }
double ldexp(double x, int exp) { while (exp > 0) { x = x * 2.0; exp = exp - 1; } while (exp < 0) { x = x / 2.0; exp = exp + 1; } return x; }
time_t time(time_t *t) { if (t) *t = 0; return 0; }
struct tm *localtime(const time_t *t) { static struct tm tm; (void)t; return &tm; }
off_t lseek(int fd, off_t offset, int whence) { (void)fd; (void)offset; (void)whence; return -1; }
int mprotect(void *addr, unsigned long len, int prot) { (void)addr; (void)len; (void)prot; return 0; }
int execvp(const char *file, char *const argv[]) { (void)file; (void)argv; return -1; }
int gettimeofday(struct timeval *tv, void *tz) { (void)tz; if (tv) { tv->tv_sec = 0; tv->tv_usec = 0; } return 0; }
EOF_DECL0_LINK_SHIMS
                run_decl0_build_phase compile-link-shims "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/tcc-decl0-link-shims.c -o /tmp/tcc-decl0-link-shims.o
                if [ -f /tmp/tcc-decl0-link-shims.o ]; then $BB chmod u+rw /tmp/tcc-decl0-link-shims.o || true; fi
                $BB cat > /tmp/tcc-decl0-startup-smoke.c <<'EOF_DECL0_STARTUP_SMOKE'
int write(int fd, const void *buf, unsigned long count);
int main(int argc, char **argv) { (void)argv; write(2, "diag-startup-smoke: main\n", 25); return argc == 123; }
EOF_DECL0_STARTUP_SMOKE
                $BB cat > /tmp/tcc-decl0-malloc-smoke.c <<'EOF_DECL0_MALLOC_SMOKE'
int write(int fd, const void *buf, unsigned long count);
void *malloc(unsigned long n);
int main(void) { void *p; write(2, "diag-malloc-smoke: before\n", 26); p = malloc(32); write(2, "diag-malloc-smoke: after\n", 25); return p ? 0 : 1; }
EOF_DECL0_MALLOC_SMOKE
                run_decl0_build_phase startup-smoke-compile "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/tcc-decl0-startup-smoke.c -o /tmp/tcc-decl0-startup-smoke.o
                run_decl0_build_phase malloc-smoke-compile "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/tcc-decl0-malloc-smoke.c -o /tmp/tcc-decl0-malloc-smoke.o
                if [ -f /tmp/tcc-decl0-startup-smoke.o ]; then $BB chmod u+rw /tmp/tcc-decl0-startup-smoke.o || true; fi
                if [ -f /tmp/tcc-decl0-malloc-smoke.o ]; then $BB chmod u+rw /tmp/tcc-decl0-malloc-smoke.o || true; fi
                run_decl0_build_phase startup-smoke-link-manual "$TCC/bin/tcc" -nostdlib -static \
                  -o /tmp/tcc-decl0-startup-smoke-manual \
                  "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/tcc-decl0-startup-smoke.o \
                  "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
                run_decl0_build_phase malloc-smoke-link-libc "$TCC/bin/tcc" -nostdlib -static \
                  -o /tmp/tcc-decl0-malloc-smoke-libc \
                  "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/tcc-decl0-malloc-smoke.o \
                  "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
                run_decl0_build_phase malloc-smoke-link-shim "$TCC/bin/tcc" -nostdlib -static \
                  -o /tmp/tcc-decl0-malloc-smoke-shim \
                  "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/tcc-decl0-malloc-smoke.o /tmp/tcc-decl0-link-shims.o \
                  "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
                for startup_smoke in manual; do
                  startup_path="/tmp/tcc-decl0-startup-smoke-$startup_smoke"
                  if [ -f "$startup_path" ]; then $BB chmod 755 "$startup_path" || true; fi
                  echo "diag-tcc-decl0-startup-smoke: run $startup_smoke" >&2
                  set +e
                  $BB timeout 5 "$startup_path" > "/tmp/tcc-decl0-startup-smoke-$startup_smoke.stdout" 2> "/tmp/tcc-decl0-startup-smoke-$startup_smoke.stderr"
                  startup_rc=$?
                  set -e
                  echo "diag-tcc-decl0-startup-smoke: $startup_smoke rc=$startup_rc" >&2
                  $BB sed 's/^/diag-tcc-decl0-startup-smoke: '$startup_smoke' stdout: /' "/tmp/tcc-decl0-startup-smoke-$startup_smoke.stdout" >&2 || true
                  $BB sed 's/^/diag-tcc-decl0-startup-smoke: '$startup_smoke' stderr: /' "/tmp/tcc-decl0-startup-smoke-$startup_smoke.stderr" >&2 || true
                done
                for malloc_smoke in libc shim; do
                  malloc_path="/tmp/tcc-decl0-malloc-smoke-$malloc_smoke"
                  if [ -f "$malloc_path" ]; then $BB chmod 755 "$malloc_path" || true; fi
                  echo "diag-tcc-decl0-malloc-smoke: run $malloc_smoke" >&2
                  set +e
                  $BB timeout 5 "$malloc_path" > "/tmp/tcc-decl0-malloc-smoke-$malloc_smoke.stdout" 2> "/tmp/tcc-decl0-malloc-smoke-$malloc_smoke.stderr"
                  malloc_rc=$?
                  set -e
                  echo "diag-tcc-decl0-malloc-smoke: $malloc_smoke rc=$malloc_rc" >&2
                  $BB sed 's/^/diag-tcc-decl0-malloc-smoke: '$malloc_smoke' stdout: /' "/tmp/tcc-decl0-malloc-smoke-$malloc_smoke.stdout" >&2 || true
                  $BB sed 's/^/diag-tcc-decl0-malloc-smoke: '$malloc_smoke' stderr: /' "/tmp/tcc-decl0-malloc-smoke-$malloc_smoke.stderr" >&2 || true
                done
                if [ -f /tmp/tcc-decl0-link-shims.o ]; then $BB chmod u+rw /tmp/tcc-decl0-link-shims.o || true; fi
                run_decl0_build_phase link-object-musl-shims "$TCC/bin/tcc" -v -nostdlib -static \
                  -o /tmp/tcc-decl0-instrumented-musl-shims \
                  "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" ./tcc-decl0-instrumented.o /tmp/tcc-decl0-link-shims.o \
                  "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
                if [ -f /tmp/tcc-decl0-instrumented-musl-shims ]; then
                  $BB chmod u+rw /tmp/tcc-decl0-instrumented-musl-shims || true
                  probe_decl0_materialized_file musl-shim-binary /tmp/tcc-decl0-instrumented-musl-shims
                  $BB grep -a 'diag-tcc-main: enter' /tmp/tcc-decl0-instrumented-musl-shims >/dev/null \
                    && echo "diag-tcc-decl0-phase: binary-has-main-marker" >&2 \
                    || echo "diag-tcc-decl0-phase: binary-missing-main-marker" >&2
                  "$BINUTILS/bin/nm" /tmp/tcc-decl0-instrumented-musl-shims 2>&1 \
                    | $BB grep -E '(^| )[Tt] (main|_start)$' \
                    | $BB sed 's/^/diag-tcc-decl0-phase: linked-symbol: /' >&2 || true
                fi
              fi
              run_decl0_build_phase link-object-default "$TCC/bin/tcc" -v -static -o /tmp/tcc-decl0-instrumented-from-object /tmp/tcc-decl0-instrumented.o
              run_decl0_build_phase link-object-explicit-archives "$TCC/bin/tcc" -v -static -o /tmp/tcc-decl0-instrumented-from-object-explicit /tmp/tcc-decl0-instrumented.o "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a"
              for input in tcc.c /tmp/tcc-decl0-instrumented.o "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a" "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" "$MUSL/lib/crtn.o"; do
                if [ -e "$input" ]; then
                  echo "diag-tcc-decl0-phase: input-exists $input" >&2
                else
                  echo "diag-tcc-decl0-phase: input-missing $input" >&2
                fi
              done
              echo "diag-tcc-decl0: build instrumented compiler" >&2
              set +e
              "$TCC/bin/tcc" -v -static -o "$diag_tcc" \
                -I . -I "$MUSL/include" \
                -D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 \
                -D TCC_TARGET_X86_64=1 \
                -D CONFIG_TCCDIR=\"$TCC/lib/tcc\" \
                -D CONFIG_TCC_CRTPREFIX=\"$MUSL/lib\" \
                -D CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\" \
                -D CONFIG_TCC_LIBPATHS=\"$MUSL/lib:$TCC/lib/tcc\" \
                -D CONFIG_TCC_SYSINCLUDEPATHS=\"$MUSL/include\" \
                -D CONFIG_SYSROOT=\"/\" -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" -D TCC_LIBTCC1=\"$TCC/lib/tcc/libtcc1.a\" \
                -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-decl0-diag\" -D ONE_SOURCE=1 \
                tcc.c 2> /tmp/tcc-decl0-build.stderr
              build_rc=$?
              set -e
              echo "diag-tcc-decl0: build rc=$build_rc" >&2
              $BB sed 's/^/diag-tcc-decl0: build-stderr: /' /tmp/tcc-decl0-build.stderr >&2 || true
              if [ "$build_rc" = 0 ]; then
                probe_decl0_materialized_file direct-build-binary "$diag_tcc"
                $BB chmod 755 "$diag_tcc"
                echo "$diag_tcc"
              elif [ -x /tmp/tcc-decl0-instrumented-musl-shims ] || [ -f /tmp/tcc-decl0-instrumented-musl-shims ]; then
                echo "diag-tcc-decl0: using musl-shim linked instrumented compiler" >&2
                diag_tcc=/tmp/tcc-decl0-instrumented-musl-shims
                $BB chmod 755 "$diag_tcc" || true
                echo "$diag_tcc"
              else
                echo ""
              fi
              cd "$WORK/gcc-4.0.4/gcc"
            }

            make_cparse_plain_exact_with_compiler() {
              compiler="$1"
              label="$2"
              body="$3"
              probe="/tmp/$label.c"
              printf '%s\\n' "$body" > "$probe"
              try_cparse_variant_with_compiler "$compiler" "$label" "$probe"
            }

            run_decl0_runtime_probe() {
              label="$1"
              shift
              echo "diag-tcc-decl0-runtime-probe: $label" >&2
              set +e
              $BB timeout 5 "$@" > "/tmp/tcc-decl0-runtime-$label.stdout" 2> "/tmp/tcc-decl0-runtime-$label.stderr"
              rc=$?
              set -e
              echo "diag-tcc-decl0-runtime-probe: $label rc=$rc" >&2
              $BB sed 's/^/diag-tcc-decl0-runtime-probe: '$label' stdout: /' "/tmp/tcc-decl0-runtime-$label.stdout" >&2 || true
              $BB sed 's/^/diag-tcc-decl0-runtime-probe: '$label' stderr: /' "/tmp/tcc-decl0-runtime-$label.stderr" >&2 || true
            }

            make_cparse_exact_after_includes_config_undef6() {
              label="$1"
              includes="$2"
              body="$3"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in $includes; do
                echo "#include \"$header\"" >> "$probe"
              done
              printf '%s\n' "$body" >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_cparse_body_after_includes_config_undef6() {
              label="$1"
              includes="$2"
              body="$3"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in $includes; do
                echo "#include \"$header\"" >> "$probe"
              done
              printf '%s\n' "$body" >> "$probe"
              echo 'int cparse_body_after_includes_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_cparse_function_after_includes_config_undef6() {
              label="$1"
              includes="$2"
              make_cparse_body_after_includes_config_undef6 "$label" "$includes" 'void cparse_function_probe (void) { }'
            }

            make_cparse_preamble_before_includes_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              printf '%s\n' "$body" > "$probe"
              echo '#include "config-undef6.h"' >> "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h output.h toplev.h ggc.h c-common.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              echo 'int cparse_preamble_before_includes_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_cparse_source_range_before_includes_config_undef6() {
              label="$1"
              first="$2"
              last="$3"
              probe="/tmp/$label.c"
              $BB awk -v first="$first" -v last="$last" 'NR >= first && NR <= last { print }' "$WORK/gcc-4.0.4/gcc/c-parse.c" > "$probe"
              echo '#include "config-undef6.h"' >> "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h output.h toplev.h ggc.h c-common.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              echo 'int cparse_source_range_before_includes_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            make_cparse_frontmatter_source_ranges_config_undef6() {
              label="$1"
              ranges="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              for header in system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h output.h toplev.h ggc.h c-common.h; do
                echo "#include \"$header\"" >> "$probe"
              done
              for range in $ranges; do
                first=${range%:*}
                last=${range#*:}
                $BB awk -v first="$first" -v last="$last" 'NR >= first && NR <= last { print }' "$WORK/gcc-4.0.4/gcc/c-parse.c" >> "$probe"
              done
              echo 'int cparse_frontmatter_source_ranges_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }

            print_tm_include_window() {
              $BB awk '/^#include / { seen++; printf("diag-tm-include: %03d %s\n", seen, $0) > "/dev/stderr" }' tm.h
            }
            print_tm_head_window() {
              $BB awk 'NR <= 60 { printf("diag-tm-head: %03d %s\n", NR, $0) > "/dev/stderr" }' tm.h
            }
            print_insn_head_window() {
              for f in insn-constants.h insn-flags.h build/insn-constants.h build/insn-flags.h; do
                if [ -f "$f" ]; then $BB awk -v name="$f" 'NR <= 40 { printf("diag-insn-head: %s:%03d %s\n", name, NR, $0) > "/dev/stderr" }' "$f"; fi
              done
              $BB find . -name 'insn-constants.h' -o -name 'insn-flags.h' | while read f; do
                echo "diag-insn-path: $f" >&2
                $BB awk -v name="$f" 'NR <= 40 { printf("diag-insn-head: %s:%03d %s\n", name, NR, $0) > "/dev/stderr" }' "$f"
              done
            }
            make_tm_include_prefix_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              $BB awk -v limit="$count" '/^#include / { print; seen++; if (seen >= limit) exit }' tm.h >> "$probe"
              echo 'int tm_include_prefix_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_tm_header_prefix_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' tm.h >> "$probe"
              echo 'int tm_header_prefix_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_tm_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' tm.h >> "$probe"
              echo '#endif /* diag balance GCC_TM_H */' >> "$probe"
              echo 'int tm_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_tm_manual_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              printf '%s\n' "$body" >> "$probe"
              echo 'int tm_manual_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            print_tree_head_window() {
              $BB awk 'NR <= 220 { printf("diag-tree-head: %03d %s\n", NR, $0) > "/dev/stderr" }' "$WORK/gcc-4.0.4/gcc/tree.h"
            }
            print_machmode_head_window() {
              $BB awk 'NR <= 140 { printf("diag-machmode-head: %03d %s\n", NR, $0) > "/dev/stderr" }' "$WORK/gcc-4.0.4/gcc/machmode.h"
            }
            make_machmode_manual_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              printf '%s\n' "$body" >> "$probe"
              echo 'int machmode_manual_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_machmode_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' "$WORK/gcc-4.0.4/gcc/machmode.h" >> "$probe"
              echo '#endif /* diag balance HAVE_MACHINE_MODES */' >> "$probe"
              echo 'int machmode_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            print_insn_modes_head_window() {
              for f in insn-modes.h build/insn-modes.h; do
                if [ -f "$f" ]; then $BB awk -v name="$f" 'NR <= 120 { printf("diag-insn-modes-head: %s:%03d %s\n", name, NR, $0) > "/dev/stderr" }' "$f"; fi
              done
              $BB find . -name 'insn-modes.h' | while read f; do
                echo "diag-insn-modes-path: $f" >&2
                $BB awk -v name="$f" 'NR <= 120 { printf("diag-insn-modes-head: %s:%03d %s\n", name, NR, $0) > "/dev/stderr" }' "$f"
              done
            }
            make_insn_modes_manual_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              printf '%s\n' "$body" >> "$probe"
              echo 'int insn_modes_manual_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_insn_modes_header_prefix_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' insn-modes.h >> "$probe"
              echo 'int insn_modes_header_prefix_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_insn_modes_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' insn-modes.h >> "$probe"
              echo '  diag_LASTmode' >> "$probe"
              echo '};' >> "$probe"
              echo '#define MAX_MACHINE_MODE diag_LASTmode' >> "$probe"
              echo '#define NUM_MACHINE_MODES ((int) MAX_MACHINE_MODE)' >> "$probe"
              echo 'int insn_modes_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_tree_manual_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              printf '%s\n' "$body" >> "$probe"
              echo 'int tree_manual_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_tree_header_balanced_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' "$WORK/gcc-4.0.4/gcc/tree.h" >> "$probe"
              echo '#endif /* diag balance GCC_TREE_H */' >> "$probe"
              echo 'int tree_header_balanced_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            print_builtins_head_window() {
              $BB awk 'NR <= 120 { printf("diag-builtins-head: %03d %s\n", NR, $0) > "/dev/stderr" }' "$WORK/gcc-4.0.4/gcc/builtins.def"
            }
            make_tree_post180_manual_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              $BB awk 'NR <= 180 { print }' "$WORK/gcc-4.0.4/gcc/tree.h" >> "$probe"
              printf '%s\n' "$body" >> "$probe"
              echo '#endif /* diag balance GCC_TREE_H */' >> "$probe"
              echo 'int tree_post180_manual_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_tree_post180_no_machmode_manual_config_undef6() {
              label="$1"
              body="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              $BB awk 'NR <= 180 && $0 !~ /#include "machmode.h"/ { print }' "$WORK/gcc-4.0.4/gcc/tree.h" >> "$probe"
              printf '%s\n' "$body" >> "$probe"
              echo '#endif /* diag balance GCC_TREE_H */' >> "$probe"
              echo 'int tree_post180_no_machmode_manual_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_tree_builtins_def_prefix_config_undef6() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              echo '#include "config-undef6.h"' > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo '#include "coretypes.h"' >> "$probe"
              echo '#include "tm.h"' >> "$probe"
              $BB awk 'NR <= 180 { print }' "$WORK/gcc-4.0.4/gcc/tree.h" >> "$probe"
              echo '#define DEF_BUILTIN(ENUM, N, C, T, LT, B, F, NA, AT, IM, COND) ENUM,' >> "$probe"
              echo 'enum built_in_function {' >> "$probe"
              $BB awk -v limit="$count" 'NR <= limit { print }' "$WORK/gcc-4.0.4/gcc/builtins.def" >> "$probe"
              echo 'END_BUILTINS' >> "$probe"
              echo '};' >> "$probe"
              echo '#undef DEF_BUILTIN' >> "$probe"
              echo '#endif /* diag balance GCC_TREE_H */' >> "$probe"
              echo 'int tree_builtins_def_prefix_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            print_mode_bounds_window() {
              $BB awk '/MIN_MODE_COMPLEX_FLOAT|MAX_MODE_COMPLEX_FLOAT|COMPLEX_FLOAT|SFmode|TFmode/ { printf("diag-mode-bound: %03d %s\n", NR, $0) > "/dev/stderr" }' insn-modes.h
            }

            echo 'int cparse_empty_probe;' > /tmp/cparse_empty.c
            try_cparse_variant cparse_empty /tmp/cparse_empty.c

            make_include_probe cparse_inc_ansidecl 'ansidecl.h'
            make_include_probe cparse_inc_autohost 'auto-host.h'
            cat > /tmp/cparse_manual_config.c <<'EOF'
#include "auto-host.h"
#ifdef IN_GCC
# include "ansidecl.h"
#endif
int manual_config_probe;
EOF
            try_cparse_variant cparse_manual_config /tmp/cparse_manual_config.c
            cat > /tmp/cparse_manual_config_nospace.c <<'EOF'
#include "auto-host.h"
#ifdef IN_GCC
#include "ansidecl.h"
#endif
int manual_config_nospace_probe;
EOF
            try_cparse_variant cparse_manual_config_nospace /tmp/cparse_manual_config_nospace.c
            make_include_probe cparse_inc_config 'config.h'
            print_autohost_define_window() {
              $BB awk '/^#define / { seen++; if (seen >= 80 && seen <= 140) printf("diag-autohost-define: %03d %s\n", seen, $0) > "/dev/stderr" }' auto-host.h
            }
            make_autohost_define_prefix_probe() {
              label="$1"
              count="$2"
              probe="/tmp/$label.c"
              $BB awk -v limit="$count" '/^#define / { print; seen++; if (seen >= limit) exit }' auto-host.h > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_prefix_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_define_skip_probe() {
              label="$1"
              count="$2"
              skip="$3"
              probe="/tmp/$label.c"
              $BB awk -v limit="$count" -v skip="$skip" '/^#define / { seen++; if (seen != skip) print; if (seen >= limit) exit }' auto-host.h > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_skip_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_define_range_probe() {
              label="$1"
              start="$2"
              end="$3"
              probe="/tmp/$label.c"
              $BB awk -v start="$start" -v end="$end" '/^#define / { seen++; if (seen >= start && seen <= end) print; if (seen >= end) exit }' auto-host.h > "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_range_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_define_undef_probe() {
              label="$1"
              count="$2"
              macro="$3"
              probe="/tmp/$label.c"
              $BB awk -v limit="$count" '/^#define / { print; seen++; if (seen >= limit) exit }' auto-host.h > "$probe"
              echo "#undef $macro" >> "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_undef_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_define_undef2_probe() {
              label="$1"
              count="$2"
              macro1="$3"
              macro2="$4"
              probe="/tmp/$label.c"
              $BB awk -v limit="$count" '/^#define / { print; seen++; if (seen >= limit) exit }' auto-host.h > "$probe"
              echo "#undef $macro1" >> "$probe"
              echo "#undef $macro2" >> "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_undef2_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_define_undef3_probe() {
              label="$1"
              count="$2"
              macro1="$3"
              macro2="$4"
              macro3="$5"
              probe="/tmp/$label.c"
              $BB awk -v limit="$count" '/^#define / { print; seen++; if (seen >= limit) exit }' auto-host.h > "$probe"
              echo "#undef $macro1" >> "$probe"
              echo "#undef $macro2" >> "$probe"
              echo "#undef $macro3" >> "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_undef3_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_define_undef4_probe() {
              label="$1"
              count="$2"
              macro1="$3"
              macro2="$4"
              macro3="$5"
              macro4="$6"
              probe="/tmp/$label.c"
              $BB awk -v limit="$count" '/^#define / { print; seen++; if (seen >= limit) exit }' auto-host.h > "$probe"
              echo "#undef $macro1" >> "$probe"
              echo "#undef $macro2" >> "$probe"
              echo "#undef $macro3" >> "$probe"
              echo "#undef $macro4" >> "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_undef4_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_define_undef5_probe() {
              label="$1"
              count="$2"
              macro1="$3"
              macro2="$4"
              macro3="$5"
              macro4="$6"
              macro5="$7"
              probe="/tmp/$label.c"
              $BB awk -v limit="$count" '/^#define / { print; seen++; if (seen >= limit) exit }' auto-host.h > "$probe"
              echo "#undef $macro1" >> "$probe"
              echo "#undef $macro2" >> "$probe"
              echo "#undef $macro3" >> "$probe"
              echo "#undef $macro4" >> "$probe"
              echo "#undef $macro5" >> "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_undef5_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_define_undef6_probe() {
              label="$1"
              count="$2"
              macro1="$3"
              macro2="$4"
              macro3="$5"
              macro4="$6"
              macro5="$7"
              macro6="$8"
              probe="/tmp/$label.c"
              $BB awk -v limit="$count" '/^#define / { print; seen++; if (seen >= limit) exit }' auto-host.h > "$probe"
              echo "#undef $macro1" >> "$probe"
              echo "#undef $macro2" >> "$probe"
              echo "#undef $macro3" >> "$probe"
              echo "#undef $macro4" >> "$probe"
              echo "#undef $macro5" >> "$probe"
              echo "#undef $macro6" >> "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_define_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_full_undef6_probe() {
              label="$1"
              macro1="$2"
              macro2="$3"
              macro3="$4"
              macro4="$5"
              macro5="$6"
              macro6="$7"
              probe="/tmp/$label.c"
              echo '#include "auto-host.h"' > "$probe"
              echo "#undef $macro1" >> "$probe"
              echo "#undef $macro2" >> "$probe"
              echo "#undef $macro3" >> "$probe"
              echo "#undef $macro4" >> "$probe"
              echo "#undef $macro5" >> "$probe"
              echo "#undef $macro6" >> "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_full_undef6_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            make_autohost_full_undef_probe() {
              label="$1"
              macro="$2"
              probe="/tmp/$label.c"
              echo '#include "auto-host.h"' > "$probe"
              echo "#undef $macro" >> "$probe"
              echo '#include "system.h"' >> "$probe"
              echo 'int autohost_full_undef_probe;' >> "$probe"
              try_cparse_variant "$label" "$probe"
            }
            # The broad autohost/generated-header/c-parse bisection matrix is
            # archived in checked v5-v7 source-frontier evidence. Keep the
            # active diagnostic compact so the builder script stays below
            # Linux MAX_ARGSTRLEN and reaches the focused make-error boundary.
            echo "diag-cparse-frontier: compact_archived_matrix_v8" >&2
          ) || true

          cp gcc/config.h gcc/config.h.generated
          cat > gcc/config.h <<'EOF'
#include "auto-host.h"
#ifdef IN_GCC
# include "ansidecl.h"
#endif
EOF
          echo "diag: normalized generated config.h to manual include form"

          cparse_source_includes=/tmp/gcc40-cparse-source-includes.txt
          $BB awk '/^#include / { gsub(/^[#]include[ \t]+\"/, ""); gsub(/\".*/, ""); print }' \
            "$WORK/gcc-4.0.4/gcc/c-parse.c" > "$cparse_source_includes"
          cparse_source_include_count=$($BB wc -l < "$cparse_source_includes")
          cparse_source_include_order=$($BB tr '\n' ',' < "$cparse_source_includes" | $BB sed 's/,$//')
          echo "diag-cparse-frontier: cparse_source_include_count=$cparse_source_include_count" >&2
          echo "diag-cparse-frontier: cparse_source_include_order=$cparse_source_include_order" >&2
          run_cparse_source_include_prefix() {
            label="$1"
            count="$2"
            probe="/tmp/$label.c"
            : > "$probe"
            i=0
            while IFS= read -r header; do
              i=$((i + 1))
              [ "$i" -le "$count" ] || break
              echo "#include \"$header\"" >> "$probe"
            done < "$cparse_source_includes"
            echo 'int cparse_source_include_prefix_probe;' >> "$probe"
            set +e
            (
              cd gcc
              gcc40-cc -c -g -DIN_GCC -DHAVE_CONFIG_H \
                -I. -I. -I$WORK/gcc-4.0.4/gcc -I$WORK/gcc-4.0.4/gcc/. \
                -I$WORK/gcc-4.0.4/gcc/../include -I$WORK/gcc-4.0.4/gcc/../libcpp/include \
                "$probe" -o "/tmp/$label.o" > "/tmp/$label.stdout" 2> "/tmp/$label.stderr"
            )
            rc=$?
            set -e
            echo "diag-cparse-frontier: ${label}_rc=$rc" >&2
            $BB sed 's/^/diag-cparse-frontier: '${label}' stderr: /' "/tmp/$label.stderr" >&2 || true
          }
          run_cparse_source_include_prefix cparse_source_include_prefix_1 1
          run_cparse_source_include_prefix cparse_source_include_prefix_2 2
          run_cparse_source_include_prefix cparse_source_include_prefix_3 3
          run_cparse_source_include_prefix cparse_source_include_prefix_4 4
          run_cparse_source_include_prefix cparse_source_include_prefix_8 8
          run_cparse_source_include_prefix cparse_source_include_prefix_11 11
          run_cparse_source_include_prefix cparse_source_include_prefix_12 12
          run_cparse_source_include_prefix cparse_source_include_prefix_18 18
          run_cparse_source_include_prefix cparse_source_include_prefix_19 19

          cparse_system_includes=/tmp/gcc40-cparse-system-direct-includes.txt
          $BB awk '/^# ?include[ \t]+[<\"]/{ print }' "$WORK/gcc-4.0.4/gcc/system.h" > "$cparse_system_includes"
          cparse_system_include_count=$($BB wc -l < "$cparse_system_includes")
          cparse_system_include_order=$($BB sed 's/^# *include[ \t]*//; s/[<>\"]//g' "$cparse_system_includes" | $BB tr '\n' ',' | $BB sed 's/,$//')
          echo "diag-cparse-frontier: cparse_system_include_count=$cparse_system_include_count" >&2
          echo "diag-cparse-frontier: cparse_system_include_order=$cparse_system_include_order" >&2
          run_cparse_system_include_prefix() {
            label="$1"
            count="$2"
            probe="/tmp/$label.c"
            echo '#include "config.h"' > "$probe"
            $BB awk -v limit="$count" 'NR <= limit { print }' "$cparse_system_includes" >> "$probe"
            echo 'int cparse_system_include_prefix_probe;' >> "$probe"
            set +e
            (
              cd gcc
              gcc40-cc -c -g -DIN_GCC -DHAVE_CONFIG_H \
                -I. -I. -I$WORK/gcc-4.0.4/gcc -I$WORK/gcc-4.0.4/gcc/. \
                -I$WORK/gcc-4.0.4/gcc/../include -I$WORK/gcc-4.0.4/gcc/../libcpp/include \
                "$probe" -o "/tmp/$label.o" > "/tmp/$label.stdout" 2> "/tmp/$label.stderr"
            )
            rc=$?
            set -e
            flood_lines=$($BB grep -c '^In file included from In file included from' "/tmp/$label.stderr" || true)
            truncated_lines=$($BB awk '/^In file included from In file included from/ { if (length($0) == 2047) c++ } END { print c + 0 }' "/tmp/$label.stderr")
            echo "diag-cparse-frontier: ${label}_rc=$rc" >&2
            echo "diag-cparse-frontier: ${label}_include_flood_lines=$flood_lines" >&2
            echo "diag-cparse-frontier: ${label}_truncated_lines=$truncated_lines" >&2
            $BB sed 's/^/diag-cparse-frontier: '${label}' stderr: /' "/tmp/$label.stderr" >&2 || true
          }
          run_cparse_stdio_probe() {
            label="$1"
            config_header="$2"
            probe="/tmp/$label.c"
            if [ -n "$config_header" ]; then
              echo "#include \"$config_header\"" > "$probe"
            else
              : > "$probe"
            fi
            echo '#include <stdio.h>' >> "$probe"
            echo 'int cparse_stdio_probe;' >> "$probe"
            set +e
            (
              cd gcc
              gcc40-cc -c -g -DIN_GCC -DHAVE_CONFIG_H \
                -I. -I. -I$WORK/gcc-4.0.4/gcc -I$WORK/gcc-4.0.4/gcc/. \
                -I$WORK/gcc-4.0.4/gcc/../include -I$WORK/gcc-4.0.4/gcc/../libcpp/include \
                "$probe" -o "/tmp/$label.o" > "/tmp/$label.stdout" 2> "/tmp/$label.stderr"
            )
            rc=$?
            set -e
            flood_lines=$($BB grep -c '^In file included from In file included from' "/tmp/$label.stderr" || true)
            truncated_lines=$($BB awk '/^In file included from In file included from/ { if (length($0) == 2047) c++ } END { print c + 0 }' "/tmp/$label.stderr")
            echo "diag-cparse-frontier: ${label}_rc=$rc" >&2
            echo "diag-cparse-frontier: ${label}_include_flood_lines=$flood_lines" >&2
            echo "diag-cparse-frontier: ${label}_truncated_lines=$truncated_lines" >&2
            $BB sed 's/^/diag-cparse-frontier: '${label}' stderr: /' "/tmp/$label.stderr" >&2 || true
          }
          run_cparse_system_include_prefix cparse_system_include_prefix_1 1
          run_cparse_system_include_prefix cparse_system_include_prefix_2 2
          run_cparse_system_include_prefix cparse_system_include_prefix_3 3
          run_cparse_system_include_prefix cparse_system_include_prefix_4 4
          run_cparse_system_include_prefix cparse_system_include_prefix_10 10
          run_cparse_system_include_prefix cparse_system_include_prefix_15 15
          run_cparse_system_include_prefix cparse_system_include_prefix_21 21
          run_cparse_stdio_probe cparse_stdio_no_config ""
          run_cparse_stdio_probe cparse_stdio_config config.h
          run_cparse_stdio_probe cparse_stdio_config_undef6 config-undef6.h

          echo "diag: reproduce focused c-parse boundary"
          m=/tmp/gcc40-cparse-make.log
          set +e
          make -j1 -C gcc \
            CC=gcc40-cc \
            AR="$BINUTILS/bin/ar" \
            RANLIB="$BINUTILS/bin/ranlib" \
            LIBGCC2_INCLUDES="-I$MUSL/include" \
            STMP_FIXINC= \
            MAKEINFO=true \
            c-parse.o \
            > "$m" 2>&1
          rc=$?
          set -e
          echo "diag-cparse-frontier: cparse_make_cparse_o_rc=$rc" >&2
          if $BB grep -q '/gcc/c-parse.c -o c-parse.o' "$m"; then
            echo "diag-cparse-frontier: cparse_make_cparse_o_compile_command=present" >&2
          else
            echo "diag-cparse-frontier: cparse_make_cparse_o_compile_command=missing" >&2
          fi
          echo "diag-cparse-frontier: cparse_make_cparse_o_log_lines=$($BB wc -l < "$m")" >&2
          echo "diag-cparse-frontier: cparse_make_cparse_o_log_bytes=$($BB wc -c < "$m")" >&2
          include_flood_lines=$($BB grep -c '^In file included from In file included from' "$m" || true)
          if [ "$include_flood_lines" -gt 0 ]; then
            echo "diag-cparse-frontier: cparse_make_include_flood=present" >&2
          else
            echo "diag-cparse-frontier: cparse_make_include_flood=missing" >&2
          fi
          echo "diag-cparse-frontier: cparse_make_include_flood_lines=$include_flood_lines" >&2
          include_flood_truncated_lines=$($BB awk '/^In file included from In file included from/ { if (length($0) == 2047) c++ } END { print c + 0 }' "$m")
          include_flood_filename_payload=$($BB awk '/^In file included from In file included from/ { line = $0; gsub(/In file included from /, "", line); if (index(line, "/") || index(line, ".") || line ~ /:[0-9]/) found = 1 } END { if (found) print "present"; else print "absent" }' "$m")
          echo "diag-cparse-frontier: cparse_make_include_flood_truncated_lines=$include_flood_truncated_lines" >&2
          echo "diag-cparse-frontier: cparse_make_include_flood_filename_payload=$include_flood_filename_payload" >&2
          echo "diag-cparse-frontier: cparse_make_cparse_o_tail" >&2
          $BB tail -n 20 "$m" >&2
          [ "$rc" = 0 ] && { echo "ERROR: c-parse.o unexpectedly compiled" >&2; exit 1; }
          exit "$rc"
        fi
        make -j1 -C "$dir" \
          CC=gcc40-cc \
          AR="$BINUTILS/bin/ar" \
          RANLIB="$BINUTILS/bin/ranlib" \
          LIBGCC2_INCLUDES="-I$MUSL/include" \
          STMP_FIXINC= \
          MAKEINFO=true \
          2>&1
      done
    "%,
  ],
  inputs = [
    stage0, tcc, musl, make, binutils, sed, m4, grep, diffutils,
    gcc_src, tcc_src,
  ],
} | crunch.Derivation
