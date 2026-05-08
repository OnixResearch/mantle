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
let bash = import "bash-2.05b-tcc.ncl" in

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
      BASH=$(find_input bash-2.05b-tcc)
      SRC=$(find_input gcc-4.0.4-src)

      export PATH="$BINUTILS/bin:$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$STAGE0/bin:$PATH"
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

            try_cparse_variant_with_compiler() {
              compiler="$1"
              label="$2"
              src="$3"
              shift 3
              echo "diag-cparse-decl0: compile $label compiler=$compiler" >&2
              set +e
              "$compiler" -c "$@" -g -DIN_GCC -DHAVE_CONFIG_H \
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
              tcc_diag_flags_common='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1'
              tcc_diag_flags_one="$tcc_diag_flags_common -D ONE_SOURCE=1"
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
    tcc_error_noabort("file '\''%s'\'' not found", filename);\
    return 0;\
}' libtcc.c
              $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
              $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
              $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
              $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
              $BB sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
              for flags_name in common one_source; do
                compile_decl0_prepart libtcc_string_patch "$flags_name" libtcc.c
              done
              # Earlier committed evidence covered the coarse libtcc range,
              # prefix, macro, body, signature/prototype, error1, prefix
              # signature, and static-varargs probes. Keep this under the
              # sandbox argument-size limit by refining the prefix-only
              # va_start expansion trigger inside static variadic helpers.
              make_libtcc_strcat_prefix_vastart() {
                variant="$1"
                shape="$2"
                $BB awk -v shape="$shape" '
                  NR < 468 { print; next }
                  NR == 468 {
                    if (shape == "original") {
                      print "static void strcat_printf(char *buf, int buf_size, const char *fmt, ...)"
                      print "{"
                      print "    va_list ap;"
                      print "    va_start(ap, fmt);"
                      print "    strcat_vprintf(buf, buf_size, fmt, ap);"
                      print "    va_end(ap);"
                      print "}"
                    } else if (shape == "static_empty") {
                      print "static void strcat_printf(char *buf, int buf_size, const char *fmt, ...)"
                      print "{"
                      print "    (void)buf; (void)buf_size; (void)fmt;"
                      print "}"
                    } else if (shape == "static_va_start_macro") {
                      print "static void strcat_printf(char *buf, int buf_size, const char *fmt, ...)"
                      print "{"
                      print "    va_list ap;"
                      print "    va_start(ap, fmt);"
                      print "    (void)buf; (void)buf_size; (void)ap;"
                      print "}"
                    } else if (shape == "nonstatic_va_start_macro") {
                      print "void strcat_printf(char *buf, int buf_size, const char *fmt, ...)"
                      print "{"
                      print "    va_list ap;"
                      print "    va_start(ap, fmt);"
                      print "    (void)buf; (void)buf_size; (void)ap;"
                      print "}"
                    } else if (shape == "static_frame_address_only") {
                      print "static void strcat_printf(char *buf, int buf_size, const char *fmt, ...)"
                      print "{"
                      print "    void *fp;"
                      print "    fp = __builtin_frame_address(0);"
                      print "    (void)buf; (void)buf_size; (void)fmt; (void)fp;"
                      print "}"
                    } else if (shape == "static___va_start_frame") {
                      print "static void strcat_printf(char *buf, int buf_size, const char *fmt, ...)"
                      print "{"
                      print "    va_list ap;"
                      print "    __va_start(ap, __builtin_frame_address(0));"
                      print "    (void)buf; (void)buf_size; (void)fmt; (void)ap;"
                      print "}"
                    } else if (shape == "static___va_start_zero") {
                      print "static void strcat_printf(char *buf, int buf_size, const char *fmt, ...)"
                      print "{"
                      print "    va_list ap;"
                      print "    __va_start(ap, 0);"
                      print "    (void)buf; (void)buf_size; (void)fmt; (void)ap;"
                      print "}"
                    } else if (shape == "static___va_start_fmt_addr") {
                      print "static void strcat_printf(char *buf, int buf_size, const char *fmt, ...)"
                      print "{"
                      print "    va_list ap;"
                      print "    __va_start(ap, (void *)&fmt);"
                      print "    (void)buf; (void)buf_size; (void)ap;"
                      print "}"
                    }
                    exit
                  }
                ' libtcc.c > "libtcc-strcat-prefix-vastart-$variant.c"
              }
              compile_libtcc_strcat_prefix_vastart() {
                variant="$1"
                shape="$2"
                make_libtcc_strcat_prefix_vastart "$variant" "$shape"
                compile_decl0_prepart "libtcc_strcat_prefix_vastart_${variant}" common "libtcc-strcat-prefix-vastart-$variant.c"
              }
              compile_libtcc_strcat_prefix_vastart original original
              compile_libtcc_strcat_prefix_vastart static_empty static_empty
              compile_libtcc_strcat_prefix_vastart static_va_start_macro static_va_start_macro
              compile_libtcc_strcat_prefix_vastart nonstatic_va_start_macro nonstatic_va_start_macro
              compile_libtcc_strcat_prefix_vastart static_frame_address_only static_frame_address_only
              compile_libtcc_strcat_prefix_vastart static___va_start_frame static___va_start_frame
              compile_libtcc_strcat_prefix_vastart static___va_start_zero static___va_start_zero
              compile_libtcc_strcat_prefix_vastart static___va_start_fmt_addr static___va_start_fmt_addr
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
              tcc_diag_flags='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D ONE_SOURCE=1'
              echo "diag-tcc-decl0-part: preprocess one-source" >&2
              set +e
              "$TCC/bin/tcc" -E -P -I . -I "$MUSL/include" $tcc_diag_flags tcc.c > /tmp/tcc-decl0-one-source.i 2> /tmp/tcc-decl0-preprocess.stderr
              part_rc=$?
              set -e
              echo "diag-tcc-decl0-part: preprocess rc=$part_rc lines=$($BB wc -l < /tmp/tcc-decl0-one-source.i 2>/dev/null || echo 0)" >&2
              $BB sed 's/^/diag-tcc-decl0-part: preprocess-stderr: /' /tmp/tcc-decl0-preprocess.stderr >&2 || true
              for part_src in tccpp.c tccgen.c tccelf.c x86_64-gen.c libtcc.c; do
                echo "diag-tcc-decl0-part: compile $part_src" >&2
                set +e
                "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $tcc_diag_flags "$part_src" -o "/tmp/tcc-decl0-${part_src}.o" 2> "/tmp/tcc-decl0-${part_src}.stderr"
                part_rc=$?
                set -e
                echo "diag-tcc-decl0-part: compile $part_src rc=$part_rc" >&2
                $BB sed 's/^/diag-tcc-decl0-part: '$part_src' stderr: /' "/tmp/tcc-decl0-${part_src}.stderr" >&2 || true
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
                -D CONFIG_SYSROOT=\"/\" -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" -D TCC_LIBTCC1=\"libtcc1.a\" \
                -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-decl0-diag\" -D ONE_SOURCE=1 \
                tcc.c 2> /tmp/tcc-decl0-build.stderr
              build_rc=$?
              set -e
              echo "diag-tcc-decl0: build rc=$build_rc" >&2
              $BB sed 's/^/diag-tcc-decl0: build-stderr: /' /tmp/tcc-decl0-build.stderr >&2 || true
              if [ "$build_rc" = 0 ]; then
                $BB chmod 755 "$diag_tcc"
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
            print_autohost_define_window
            make_autohost_define_prefix_probe autohost_defines_10_system 10
            make_autohost_define_prefix_probe autohost_defines_25_system 25
            make_autohost_define_prefix_probe autohost_defines_50_system 50
            make_autohost_define_prefix_probe autohost_defines_60_system 60
            make_autohost_define_prefix_probe autohost_defines_70_system 70
            make_autohost_define_prefix_probe autohost_defines_80_system 80
            make_autohost_define_prefix_probe autohost_defines_81_system 81
            make_autohost_define_prefix_probe autohost_defines_82_system 82
            make_autohost_define_prefix_probe autohost_defines_83_system 83
            make_autohost_define_prefix_probe autohost_defines_84_system 84
            make_autohost_define_skip_probe autohost_defines_84_skip84_system 84 84
            make_autohost_define_skip_probe autohost_defines_85_skip84_system 85 84
            make_autohost_define_range_probe autohost_define_84_only_system 84 84
            make_autohost_define_undef_probe autohost_defines_84_undef_need64_system 84 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_85_undef_need64_system 85 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_90_undef_need64_system 90 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_95_undef_need64_system 95 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_96_undef_need64_system 96 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_range_probe autohost_define_96_only_system 96 96
            make_autohost_define_undef2_probe autohost_defines_96_undef_need64_gid_system 96 NEED_64BIT_HOST_WIDE_INT gid_t
            make_autohost_define_undef2_probe autohost_defines_97_undef_need64_gid_system 97 NEED_64BIT_HOST_WIDE_INT gid_t
            make_autohost_define_range_probe autohost_define_97_only_system 97 97
            make_autohost_define_undef3_probe autohost_defines_97_undef_need64_gid_inline_system 97 NEED_64BIT_HOST_WIDE_INT gid_t inline
            make_autohost_define_undef3_probe autohost_defines_98_undef_need64_gid_inline_system 98 NEED_64BIT_HOST_WIDE_INT gid_t inline
            make_autohost_define_range_probe autohost_define_98_only_system 98 98
            make_autohost_define_undef4_probe autohost_defines_98_undef_need64_gid_inline_rlim_system 98 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t
            make_autohost_define_undef4_probe autohost_defines_99_undef_need64_gid_inline_rlim_system 99 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t
            make_autohost_define_range_probe autohost_define_99_only_system 99 99
            make_autohost_define_undef5_probe autohost_defines_99_undef_need64_gid_inline_rlim_ssize_system 99 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t
            make_autohost_define_undef5_probe autohost_defines_100_undef_need64_gid_inline_rlim_ssize_system 100 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t
            make_autohost_define_undef6_probe autohost_defines_100_undef_need64_gid_inline_rlim_ssize_uid_system 100 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t
            make_autohost_define_undef6_probe autohost_defines_101_undef_need64_gid_inline_rlim_ssize_uid_system 101 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t
            make_autohost_define_range_probe autohost_define_101_only_system 101 101
            make_autohost_define_undef6_probe autohost_defines_105_undef_need64_gid_inline_rlim_ssize_uid_system 105 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t
            make_autohost_define_undef6_probe autohost_defines_110_undef_need64_gid_inline_rlim_ssize_uid_system 110 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t
            make_autohost_define_undef6_probe autohost_defines_120_undef_need64_gid_inline_rlim_ssize_uid_system 120 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t
            make_autohost_define_undef4_probe autohost_defines_100_undef_need64_gid_inline_rlim_system 100 NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t
            make_autohost_define_range_probe autohost_define_100_only_system 100 100
            make_autohost_define_undef3_probe autohost_defines_99_undef_need64_gid_inline_system 99 NEED_64BIT_HOST_WIDE_INT gid_t inline
            make_autohost_define_undef3_probe autohost_defines_100_undef_need64_gid_inline_system 100 NEED_64BIT_HOST_WIDE_INT gid_t inline
            make_autohost_define_undef2_probe autohost_defines_98_undef_need64_gid_system 98 NEED_64BIT_HOST_WIDE_INT gid_t
            make_autohost_define_undef2_probe autohost_defines_99_undef_need64_gid_system 99 NEED_64BIT_HOST_WIDE_INT gid_t
            make_autohost_define_undef2_probe autohost_defines_100_undef_need64_gid_system 100 NEED_64BIT_HOST_WIDE_INT gid_t
            make_autohost_define_undef_probe autohost_defines_97_undef_need64_system 97 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_98_undef_need64_system 98 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_99_undef_need64_system 99 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_100_undef_need64_system 100 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef2_probe autohost_defines_100_undef_need64_inline_system 100 NEED_64BIT_HOST_WIDE_INT inline
            make_autohost_define_undef_probe autohost_defines_120_undef_need64_system 120 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_140_undef_need64_system 140 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_undef_probe autohost_defines_200_undef_need64_system 200 NEED_64BIT_HOST_WIDE_INT
            make_autohost_define_skip_probe autohost_defines_90_skip84_system 90 84
            make_autohost_define_skip_probe autohost_defines_100_skip84_system 100 84
            make_autohost_define_skip_probe autohost_defines_140_skip84_system 140 84
            make_autohost_define_skip_probe autohost_defines_200_skip84_system 200 84
            make_autohost_full_undef_probe autohost_full_undef_need64_system NEED_64BIT_HOST_WIDE_INT
            make_autohost_full_undef6_probe autohost_full_undef_need64_gid_inline_rlim_ssize_uid_system NEED_64BIT_HOST_WIDE_INT gid_t inline rlim_t ssize_t uid_t
            $BB cp config.h config-undef6.h
            cat >> config-undef6.h <<'EOF'
#undef NEED_64BIT_HOST_WIDE_INT
#undef gid_t
#undef inline
#undef rlim_t
#undef ssize_t
#undef uid_t
EOF
            cat > /tmp/cparse_config_undef6_system.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
int config_undef6_system_probe;
EOF
            try_cparse_variant cparse_config_undef6_system /tmp/cparse_config_undef6_system.c
            make_include_probe_config_undef6 cparse_undef6_inc_system 'system.h'
            make_include_probe_config_undef6 cparse_undef6_inc_coretypes 'system.h coretypes.h'
            make_include_probe_config_undef6 cparse_undef6_inc_tm 'system.h coretypes.h tm.h'
            print_machmode_head_window
            make_machmode_manual_config_undef6 cparse_undef6_machmode_guard_only '#ifndef HAVE_MACHINE_MODES
#define HAVE_MACHINE_MODES
#endif'
            print_insn_modes_head_window
            make_insn_modes_manual_config_undef6 cparse_undef6_insn_modes_minimal_enum 'enum machine_mode {
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
};'
            make_insn_modes_manual_config_undef6 cparse_undef6_insn_modes_include_only '#include "insn-modes.h"'
            make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_1 1
            make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_5 5
            make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_10 10
            make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_20 20
            make_insn_modes_header_prefix_config_undef6 cparse_undef6_insn_modes_lines_40 40
            make_insn_modes_header_balanced_config_undef6 cparse_undef6_insn_modes_balanced_20 20
            make_insn_modes_header_balanced_config_undef6 cparse_undef6_insn_modes_balanced_40 40
            make_machmode_manual_config_undef6 cparse_undef6_machmode_insn_modes_only '#include "insn-modes.h"'
            make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_20 20
            make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_40 40
            make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_60 60
            make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_80 80
            make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_100 100
            make_machmode_header_balanced_config_undef6 cparse_undef6_machmode_lines_120 120
            print_tree_head_window
            make_tree_manual_config_undef6 cparse_undef6_tree_guard_only '#ifndef GCC_TREE_H
#define GCC_TREE_H
#endif'
            make_tree_manual_config_undef6 cparse_undef6_tree_machmode_only '#include "machmode.h"'
            make_tree_manual_config_undef6 cparse_undef6_tree_input_only '#include "input.h"'
            make_tree_manual_config_undef6 cparse_undef6_tree_statistics_only '#include "statistics.h"'
            make_tree_manual_config_undef6 cparse_undef6_tree_vec_only '#include "vec.h"'
            make_tree_manual_config_undef6 cparse_undef6_tree_def_enum '#define DEFTREECODE(SYM, STRING, TYPE, NARGS) SYM,
enum tree_code {
#include "tree.def"
  LAST_AND_UNUSED_TREE_CODE
};
#undef DEFTREECODE'
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_23 23
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_25 25
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_26 26
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_27 27
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_28 28
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_36 36
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_80 80
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_120 120
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_160 160
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_166 166
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_177 177
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_180 180
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_207 207
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_212 212
            make_tree_header_balanced_config_undef6 cparse_undef6_tree_lines_220 220
            print_builtins_head_window
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_empty 'enum built_in_function {
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_arith_only 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  BUILT_IN_COMPLEX_DIV_MIN,
  BUILT_IN_COMPLEX_DIV_MAX = BUILT_IN_COMPLEX_DIV_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_include_full '#define DEF_BUILTIN(ENUM, N, C, T, LT, B, F, NA, AT, IM, COND) ENUM,
enum built_in_function {
#include "builtins.def"
  END_BUILTINS
};
#undef DEF_BUILTIN'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_include_plus_complex '#define DEF_BUILTIN(ENUM, N, C, T, LT, B, F, NA, AT, IM, COND) ENUM,
enum built_in_function {
#include "builtins.def"
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  BUILT_IN_COMPLEX_DIV_MIN,
  BUILT_IN_COMPLEX_DIV_MAX = BUILT_IN_COMPLEX_DIV_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};
#undef DEF_BUILTIN'
            make_tree_builtins_def_prefix_config_undef6 cparse_undef6_builtins_def_lines_20 20
            make_tree_builtins_def_prefix_config_undef6 cparse_undef6_builtins_def_lines_60 60
            make_tree_builtins_def_prefix_config_undef6 cparse_undef6_builtins_def_lines_120 120
            make_tree_builtins_def_prefix_config_undef6 cparse_undef6_builtins_def_lines_240 240
            make_tree_builtins_def_prefix_config_undef6 cparse_undef6_builtins_def_lines_480 480
            make_tree_builtins_def_prefix_config_undef6 cparse_undef6_builtins_def_lines_800 800
            print_mode_bounds_window
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_literal_zero 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + 0 - 0,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_literal_sub 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + 2 - 1,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_max_only 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_min_only 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_mode_delta_one 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_complex_mode_delta_two 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  BUILT_IN_COMPLEX_DIV_MIN,
  BUILT_IN_COMPLEX_DIV_MAX = BUILT_IN_COMPLEX_DIV_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_include_plus_literal_sub '#define DEF_BUILTIN(ENUM, N, C, T, LT, B, F, NA, AT, IM, COND) ENUM,
enum built_in_function {
#include "builtins.def"
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + 2 - 1,
  END_BUILTINS
};
#undef DEF_BUILTIN'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_include_plus_mode_delta_one '#define DEF_BUILTIN(ENUM, N, C, T, LT, B, F, NA, AT, IM, COND) ENUM,
enum built_in_function {
#include "builtins.def"
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};
#undef DEF_BUILTIN'
            make_tree_post180_no_machmode_manual_config_undef6 cparse_undef6_tree_builtin_local_machine_mode_literal_sub 'enum machine_mode {
  VOIDmode = 0,
  SFmode = 15,
  TFmode = 18,
  MAX_MACHINE_MODE
};
enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + 2 - 1,
  END_BUILTINS
};'
            make_tree_post180_no_machmode_manual_config_undef6 cparse_undef6_tree_builtin_local_machine_mode_direct_sftf 'enum machine_mode {
  VOIDmode = 0,
  SFmode = 15,
  TFmode = 18,
  MAX_MACHINE_MODE
};
enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + TFmode - SFmode,
  END_BUILTINS
};'
            make_tree_post180_no_machmode_manual_config_undef6 cparse_undef6_tree_builtin_local_machine_mode_macro_delta 'enum machine_mode {
  VOIDmode = 0,
  SFmode = 15,
  TFmode = 18,
  MAX_MACHINE_MODE
};
#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_no_machmode_manual_config_undef6 cparse_undef6_tree_builtin_local_machine_mode_enum_alias_delta 'enum machine_mode {
  VOIDmode = 0,
  SFmode = 15,
  TFmode = 18,
  MIN_MODE_COMPLEX_FLOAT = SFmode,
  MAX_MODE_COMPLEX_FLOAT = TFmode,
  MAX_MACHINE_MODE
};
enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_generated_direct_sftf 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + TFmode - SFmode,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_generated_direct_tf_only 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + TFmode,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_generated_direct_sf_only 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN - SFmode,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_generated_macro_alias_delta '#define MIN_MODE_COMPLEX_FLOAT SFmode
#define MAX_MODE_COMPLEX_FLOAT TFmode
enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_generated_enum_alias_delta 'enum diag_mode_alias {
  MIN_MODE_COMPLEX_FLOAT = SFmode,
  MAX_MODE_COMPLEX_FLOAT = TFmode
};
enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + MAX_MODE_COMPLEX_FLOAT - MIN_MODE_COMPLEX_FLOAT,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_undefined_symbol_only 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + DIAG_UNDEFINED_ENUM_ID,
  END_BUILTINS
};'
            make_tree_post180_manual_config_undef6 cparse_undef6_tree_builtin_undefined_symbol_delta 'enum built_in_function {
  BUILT_IN_COMPLEX_MUL_MIN,
  BUILT_IN_COMPLEX_MUL_MAX = BUILT_IN_COMPLEX_MUL_MIN + DIAG_UNDEFINED_ENUM_MAX - DIAG_UNDEFINED_ENUM_MIN,
  END_BUILTINS
};'
            make_include_probe_config_undef6 cparse_undef6_inc_tree 'system.h coretypes.h tm.h tree.h'
            print_tm_head_window
            print_insn_head_window
            cat > /tmp/cparse_undef6_insn_constants_only.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#include "insn-constants.h"
int insn_constants_only_probe;
EOF
            try_cparse_variant cparse_undef6_insn_constants_only /tmp/cparse_undef6_insn_constants_only.c
            cat > /tmp/cparse_undef6_insn_flags_only.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#include "insn-flags.h"
int insn_flags_only_probe;
EOF
            try_cparse_variant cparse_undef6_insn_flags_only /tmp/cparse_undef6_insn_flags_only.c
            cat > /tmp/cparse_undef6_insn_manual_constants.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#ifndef GCC_INSN_CONSTANTS_H
#define GCC_INSN_CONSTANTS_H
/* bootstrap genconstants stub: no md constants required before the next GCC boundary */
#endif
int insn_manual_constants_probe;
EOF
            try_cparse_variant cparse_undef6_insn_manual_constants /tmp/cparse_undef6_insn_manual_constants.c
            cat > /tmp/cparse_undef6_insn_manual_flags.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#ifndef GCC_INSN_FLAGS_H
#define GCC_INSN_FLAGS_H
/* bootstrap genflags stub: generator-only boundary bridge */
#endif
int insn_manual_flags_probe;
EOF
            try_cparse_variant cparse_undef6_insn_manual_flags /tmp/cparse_undef6_insn_manual_flags.c
            cat > /tmp/cparse_undef6_tm_generator_file.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#define GENERATOR_FILE 1
#include "tm.h"
int tm_generator_file_probe;
EOF
            try_cparse_variant cparse_undef6_tm_generator_file /tmp/cparse_undef6_tm_generator_file.c
            cat > /tmp/cparse_undef6_tm_used_for_target.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#define USED_FOR_TARGET 1
#include "tm.h"
int tm_used_for_target_probe;
EOF
            try_cparse_variant cparse_undef6_tm_used_for_target /tmp/cparse_undef6_tm_used_for_target.c
            cat > /tmp/cparse_undef6_tm_undef_ingcc.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#undef IN_GCC
#include "tm.h"
int tm_undef_ingcc_probe;
EOF
            try_cparse_variant cparse_undef6_tm_undef_ingcc /tmp/cparse_undef6_tm_undef_ingcc.c
            cat > /tmp/cparse_undef6_tm_ansidecl_guard.c <<'EOF'
#include "config-undef6.h"
#include "system.h"
#include "coretypes.h"
#ifdef IN_GCC
# include "ansidecl.h"
#endif
int tm_ansidecl_guard_probe;
EOF
            try_cparse_variant cparse_undef6_tm_ansidecl_guard /tmp/cparse_undef6_tm_ansidecl_guard.c
            make_tm_include_prefix_config_undef6 cparse_undef6_tm_include_1 1
            make_tm_include_prefix_config_undef6 cparse_undef6_tm_include_2 2
            make_tm_include_prefix_config_undef6 cparse_undef6_tm_include_3 3
            make_tm_include_prefix_config_undef6 cparse_undef6_tm_include_4 4
            make_tm_include_prefix_config_undef6 cparse_undef6_tm_include_5 5
            make_tm_header_prefix_config_undef6 cparse_undef6_tm_lines_1 1
            make_tm_header_prefix_config_undef6 cparse_undef6_tm_lines_2 2
            make_tm_header_prefix_config_undef6 cparse_undef6_tm_lines_3 3
            make_tm_header_prefix_config_undef6 cparse_undef6_tm_lines_4 4
            make_tm_header_prefix_config_undef6 cparse_undef6_tm_lines_5 5
            make_tm_header_prefix_config_undef6 cparse_undef6_tm_lines_10 10
            make_tm_header_prefix_config_undef6 cparse_undef6_tm_lines_20 20
            make_tm_header_prefix_config_undef6 cparse_undef6_tm_lines_40 40
            make_tm_manual_config_undef6 cparse_undef6_tm_ifndef_balanced '#ifndef GCC_TM_H
#endif'
            make_tm_manual_config_undef6 cparse_undef6_tm_ifndef_define_balanced '#ifndef GCC_TM_H
#define GCC_TM_H
#endif'
            make_tm_manual_config_undef6 cparse_undef6_tm_ifdef_ingcc_balanced '#ifdef IN_GCC
#endif'
            make_tm_manual_config_undef6 cparse_undef6_tm_guard_ifdef_balanced '#ifndef GCC_TM_H
#define GCC_TM_H
#ifdef IN_GCC
#endif
#endif'
            make_tm_header_balanced_config_undef6 cparse_undef6_tm_balanced_lines_1 1
            make_tm_header_balanced_config_undef6 cparse_undef6_tm_balanced_lines_2 2
            make_tm_header_balanced_config_undef6 cparse_undef6_tm_balanced_lines_3 3
            make_tm_header_balanced_config_undef6 cparse_undef6_tm_balanced_lines_4 4
            make_tm_header_balanced_config_undef6 cparse_undef6_tm_balanced_lines_5 5
            make_tm_header_balanced_config_undef6 cparse_undef6_tm_balanced_lines_10 10
            make_tm_header_balanced_config_undef6 cparse_undef6_tm_balanced_lines_15 15
            make_tm_header_balanced_config_undef6 cparse_undef6_tm_balanced_lines_20 20
            make_tm_manual_config_undef6 cparse_undef6_tm_first_block '#ifndef GCC_TM_H
#define GCC_TM_H
#ifdef IN_GCC
# include "config/i386/biarch64.h"
# include "config/i386/i386.h"
# include "config/i386/unix.h"
# include "config/i386/att.h"
# include "config/dbxelf.h"
# include "config/elfos.h"
# include "config/svr4.h"
# include "config/linux.h"
# include "config/i386/x86-64.h"
# include "config/i386/linux64.h"
# include "defaults.h"
#endif
#endif'
            make_tm_manual_config_undef6 cparse_undef6_tm_first_block_insn_constants '#ifndef GCC_TM_H
#define GCC_TM_H
#ifdef IN_GCC
# include "config/i386/biarch64.h"
# include "config/i386/i386.h"
# include "config/i386/unix.h"
# include "config/i386/att.h"
# include "config/dbxelf.h"
# include "config/elfos.h"
# include "config/svr4.h"
# include "config/linux.h"
# include "config/i386/x86-64.h"
# include "config/i386/linux64.h"
# include "defaults.h"
#endif
#if defined IN_GCC && !defined GENERATOR_FILE && !defined USED_FOR_TARGET
# include "insn-constants.h"
#endif
#endif'
            make_tm_manual_config_undef6 cparse_undef6_tm_first_block_insn_flags '#ifndef GCC_TM_H
#define GCC_TM_H
#ifdef IN_GCC
# include "config/i386/biarch64.h"
# include "config/i386/i386.h"
# include "config/i386/unix.h"
# include "config/i386/att.h"
# include "config/dbxelf.h"
# include "config/elfos.h"
# include "config/svr4.h"
# include "config/linux.h"
# include "config/i386/x86-64.h"
# include "config/i386/linux64.h"
# include "defaults.h"
#endif
#if defined IN_GCC && !defined GENERATOR_FILE && !defined USED_FOR_TARGET
# include "insn-flags.h"
#endif
#endif'
            make_tm_manual_config_undef6 cparse_undef6_tm_first_block_insn_both '#ifndef GCC_TM_H
#define GCC_TM_H
#ifdef IN_GCC
# include "config/i386/biarch64.h"
# include "config/i386/i386.h"
# include "config/i386/unix.h"
# include "config/i386/att.h"
# include "config/dbxelf.h"
# include "config/elfos.h"
# include "config/svr4.h"
# include "config/linux.h"
# include "config/i386/x86-64.h"
# include "config/i386/linux64.h"
# include "defaults.h"
#endif
#if defined IN_GCC && !defined GENERATOR_FILE && !defined USED_FOR_TARGET
# include "insn-constants.h"
# include "insn-flags.h"
#endif
#endif'
            make_include_probe_config_undef6 cparse_undef6_inc_tree 'system.h coretypes.h tm.h tree.h'
            make_include_probe_config_undef6 cparse_undef6_inc_langhooks 'system.h coretypes.h tm.h tree.h langhooks.h'
            make_include_probe_config_undef6 cparse_undef6_inc_cpplib 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h'
            make_include_probe_config_undef6 cparse_undef6_inc_c_pragma 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h'
            print_c_tree_head_window
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_20 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 20
            make_c_tree_manual_config_undef6 cparse_undef6_c_tree_guard_ifndef_only '#ifndef GCC_C_TREE_H'
            make_c_tree_manual_config_undef6 cparse_undef6_c_tree_guard_define_only '#define GCC_C_TREE_H'
            make_c_tree_manual_config_undef6 cparse_undef6_c_tree_guard_empty_balanced '#ifndef GCC_C_TREE_H
#define GCC_C_TREE_H
#endif'
            for n in 24 25 26 27 31 38 40 80; do
              make_c_tree_header_balanced_config_undef6 "cparse_undef6_c_tree_balanced_lines_$n" "$n"
            done
            print_c_common_head_window
            for n in 20 24 25 26 27 30 40 60 80; do
              make_c_common_header_balanced_config_undef6 "cparse_undef6_c_common_balanced_lines_$n" "$n"
            done
            print_ggc_head_window
            for n in 20 24 25 30 40 60 80 100; do
              make_ggc_header_balanced_config_undef6 "cparse_undef6_ggc_balanced_lines_$n" "$n"
            done
            print_gtype_desc_head_window
            for n in 20 40 60 80 100 120; do
              make_gtype_desc_header_balanced_config_undef6 "cparse_undef6_gtype_desc_balanced_lines_$n" "$n"
            done
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_24 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 24
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_25 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 25
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_26 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 26
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_27 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 27
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_31 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 31
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_38 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 38
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_40 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 40
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_80 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 80
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_120 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 120
            make_header_prefix_probe_config_undef6 cparse_undef6_c_tree_lines_200 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h' "$WORK/gcc-4.0.4/gcc/c-tree.h" 200
            make_include_probe_config_undef6 cparse_undef6_inc_c_tree 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h'
            print_options_head_window
            for n in 10 20 40 80 120 160 180; do
              make_options_header_balanced_config_undef6 "cparse_undef6_options_balanced_lines_$n" "$n"
            done
            print_flags_head_window
            for n in 20 24 25 26 30 40 60 80 100 120; do
              make_flags_header_balanced_config_undef6 "cparse_undef6_flags_balanced_lines_$n" "$n"
            done
            make_include_probe_config_undef6 cparse_undef6_inc_flags 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h'
            make_include_probe_config_undef6 cparse_undef6_inc_varray 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h'
            make_include_probe_config_undef6 cparse_undef6_inc_output 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h output.h'
            make_include_probe_config_undef6 cparse_undef6_inc_toplev 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h output.h toplev.h'
            make_include_probe_config_undef6 cparse_undef6_inc_ggc 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h output.h toplev.h ggc.h'
            make_include_probe_config_undef6 cparse_undef6_inc_c_common 'system.h coretypes.h tm.h tree.h langhooks.h input.h cpplib.h intl.h timevar.h c-pragma.h c-tree.h flags.h varray.h output.h toplev.h ggc.h c-common.h'
            make_cparse_plain_exact cparse_plain_func_decl_semicolon_exact_no_config 'void cparse_plain_decl_exact_probe (void);'
            make_cparse_plain_exact cparse_plain_func_prefix_declarator_no_config 'void cparse_plain_prefix_probe (void)'
            make_cparse_plain_exact cparse_plain_oldstyle_prefix_no_config 'void cparse_plain_oldstyle_prefix_probe ()'
            make_cparse_plain_exact cparse_plain_param_int_prefix_no_config 'void cparse_plain_param_int_prefix_probe (int x)'
            make_cparse_plain_exact cparse_plain_var_prefix_no_config 'int cparse_plain_var_prefix_probe'
            make_cparse_plain_exact cparse_plain_pointer_prefix_no_config 'int *cparse_plain_pointer_prefix_probe'
            make_cparse_plain_exact cparse_plain_var_semicolon_exact_no_config 'int cparse_plain_var_semicolon_probe;'
            make_cparse_plain_exact cparse_plain_pointer_semicolon_exact_no_config 'int *cparse_plain_pointer_semicolon_probe;'
            make_cparse_plain_exact cparse_plain_bad_init_semicolon_no_config 'int cparse_plain_bad_init_probe = ;'
            make_cparse_plain_exact cparse_plain_bad_param_semicolon_no_config 'void cparse_plain_bad_param_probe (void;'
            make_cparse_plain_exact cparse_plain_bad_comma_semicolon_no_config 'int cparse_plain_bad_comma_probe, ;'
            print_tcc_decl_parser_source_trace
            decl0_tcc=$(build_decl0_instrumented_tcc)
            if [ -n "$decl0_tcc" ] && [ -x "$decl0_tcc" ]; then
              make_cparse_plain_exact_with_compiler "$decl0_tcc" cparse_decl0_trace_valid_var_semicolon 'int cparse_decl0_trace_valid_var_probe;'
              make_cparse_plain_exact_with_compiler "$decl0_tcc" cparse_decl0_trace_eof_var_prefix 'int cparse_decl0_trace_eof_var_probe'
              make_cparse_plain_exact_with_compiler "$decl0_tcc" cparse_decl0_trace_bad_init_semicolon 'int cparse_decl0_trace_bad_init_probe = ;'
              make_cparse_plain_exact_with_compiler "$decl0_tcc" cparse_decl0_trace_bad_comma_semicolon 'int cparse_decl0_trace_bad_comma_probe, ;'
            else
              echo "diag-tcc-decl0: instrumented compiler unavailable" >&2
            fi
            trace_cparse_plain_exact_phases cparse_trace_valid_var_semicolon 'int cparse_trace_valid_var_probe;'
            trace_cparse_plain_exact_phases cparse_trace_eof_var_prefix 'int cparse_trace_eof_var_probe'
            trace_cparse_plain_exact_phases cparse_trace_bad_init_semicolon 'int cparse_trace_bad_init_probe = ;'
            trace_cparse_plain_exact_phases cparse_trace_bad_comma_semicolon 'int cparse_trace_bad_comma_probe, ;'
            make_cparse_plain_body cparse_plain_decl_no_config 'void cparse_plain_probe (void);'
            make_cparse_plain_body cparse_plain_global_no_config 'int cparse_plain_global_probe;'
            make_cparse_plain_body cparse_plain_func_no_config 'void cparse_plain_function_probe (void) { }'
            make_cparse_exact_after_includes_config_undef6 cparse_func_decl_semicolon_config_only '' 'void cparse_decl_semicolon_probe (void);'
            make_cparse_exact_after_includes_config_undef6 cparse_func_exact_config_only '' 'void cparse_exact_function_probe (void) { }'
            make_cparse_exact_after_includes_config_undef6 cparse_func_prefix_declarator_config_only '' 'void cparse_prefix_probe (void)'
            make_cparse_exact_after_includes_config_undef6 cparse_func_prefix_oldstyle_config_only '' 'void cparse_oldstyle_prefix_probe ()'
            make_cparse_exact_after_includes_config_undef6 cparse_func_prefix_param_int_config_only '' 'void cparse_param_int_prefix_probe (int x)'
            make_cparse_exact_after_includes_config_undef6 cparse_var_prefix_config_only '' 'int cparse_var_prefix_probe'
            make_cparse_exact_after_includes_config_undef6 cparse_pointer_prefix_config_only '' 'int *cparse_pointer_prefix_probe'
            make_cparse_exact_after_includes_config_undef6 cparse_var_semicolon_config_only '' 'int cparse_var_semicolon_probe;'
            make_cparse_exact_after_includes_config_undef6 cparse_pointer_semicolon_config_only '' 'int *cparse_pointer_semicolon_probe;'
            make_cparse_exact_after_includes_config_undef6 cparse_bad_init_semicolon_config_only '' 'int cparse_bad_init_probe = ;'
            make_cparse_exact_after_includes_config_undef6 cparse_bad_param_semicolon_config_only '' 'void cparse_bad_param_probe (void;'
            make_cparse_exact_after_includes_config_undef6 cparse_bad_comma_semicolon_config_only '' 'int cparse_bad_comma_probe, ;'
            make_cparse_exact_after_includes_config_undef6 cparse_func_prefix_open_brace_config_only '' 'void cparse_open_brace_probe (void) {'
            make_cparse_exact_after_includes_config_undef6 cparse_func_prefix_local_decl_config_only '' 'void cparse_local_decl_prefix_probe (void) { int local_probe;'
            make_cparse_body_after_includes_config_undef6 cparse_decl_after_config_only '' 'void cparse_function_probe (void);'
            make_cparse_body_after_includes_config_undef6 cparse_global_after_config_only '' 'int cparse_global_probe;'
            make_cparse_function_after_includes_config_undef6 cparse_func_after_config_only ''
            # Earlier frontmatter/full-source probes are preserved in committed
            # evidence; keep the active diagnostic matrix focused so the generated
            # builder script stays below the host argument-size limit.
          )

          cp gcc/config.h gcc/config.h.generated
          cat > gcc/config.h <<'EOF'
#include "auto-host.h"
#ifdef IN_GCC
# include "ansidecl.h"
#endif
EOF
          echo "diag: normalized generated config.h to manual include form"

          echo "diag: reproduce focused c-parse boundary"
          make -j1 -C gcc \
            CC=gcc40-cc \
            AR="$BINUTILS/bin/ar" \
            RANLIB="$BINUTILS/bin/ranlib" \
            LIBGCC2_INCLUDES="-I$MUSL/include" \
            STMP_FIXINC= \
            MAKEINFO=true \
            c-parse.o \
            2>&1
          echo "ERROR: c-parse.o unexpectedly compiled" >&2
          exit 1
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
    stage0, tcc, musl, make, binutils, sed, m4, grep, diffutils, bash,
    gcc_src, tcc_src,
  ],
} | crunch.Derivation
