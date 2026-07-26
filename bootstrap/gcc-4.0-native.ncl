# Diagnose a two-pass GCC 4.0.4 build using the declared TinyCC/musl closure.
#
# This isolated candidate follows GCC's subdirectory build shape, but currently
# relies on a visible pass1 TinyCC wrapper, shipped generated files, bounded
# source rewrites, and predecessor fallbacks.  It is frontier evidence only and
# cannot satisfy source-built provider admission.

let crunch = import "lib.ncl" in

# Keep every diagnostic tool edge explicit.  The shipped GCC release already
# contains generated configure/parser sources, so this root does not depend on
# the currently incomplete Perl/autoconf/automake regeneration chain.
let binutils = crunch.derivationFile "binutils-tcc.ncl" in
let tcc = crunch.derivationFile "tcc-musl-native.ncl" in
let tcc_selfhost = crunch.derivationFile "tcc-musl-selfhost.ncl" in
let tcc26 = crunch.derivationFile "tcc-musl-v2.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-native.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let sed = crunch.derivationFile "sed-4.0.9-musl.ncl" in
let m4 = crunch.derivationFile "m4-1.4.7-musl.ncl" in
let grep = crunch.derivationFile "grep-2.4-musl.ncl" in
let diffutils = crunch.derivationFile "diffutils-2.7-musl.ncl" in
let gawk = crunch.derivationFile "gawk-3.0.4-musl.ncl" in

# Upstream uses a modern automake source only for config.sub. Keeping it as a
# source input avoids constructing an unrelated late autotools output.
let automake115_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/automake/automake-1.15.1.tar.xz",
  hash = "sha256-/KHwBUWoEgUxjLuLFOpqM4c0UXUDWiyiBV4HBTwOcZU=",
  name = "automake-1.15.1-config-sub-src",
} in

let gcc_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap commit 9a268c4c39cae952b268bc86da342be2175f03d4
  # mechanism: steps/gcc-4.0.4/pass1.sh + pass2.sh
  url = "https://ftpmirror.gnu.org/gcc/gcc-4.0.4/gcc-4.0.4.tar.bz2",
  hash = "sha256-QNHCUqWDcOtqVLokzAd3MfsWWR/jN3HdAld/Pm+GSM0=",
  name = "gcc-4.0.4-native-src",
} in

{
  name = "gcc-4.0.4-native-gas-v45",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=4.0.4
      JOB_COUNT=1
      WORK=/tmp/gcc-4.0.4-native
      PASS1=/tmp/gcc-4.0.4-pass1
      COMPILER_TEARDOWN_STATUS=139
      GCC_ASSEMBLER_COMPAT_FLAGS="-fno-asynchronous-unwind-tables -fno-unwind-tables -fno-common -D__STDC__=1"

      find_input() {
        input_name="$1"
        for input_path in $NIX_STORE/*-"$input_name"; do
          if [ -d "$input_path" ]; then
            echo "$input_path"
            return
          fi
        done
        echo "ERROR: input $input_name not found" >&2
        exit 1
      }

      TCC=$(find_input tcc-0.9.27-musl-native-runtime)
      TCC_SELFHOST=$(find_input tcc-0.9.27-musl-selfhost)
      TCC26=$(find_input tcc-0.9.27-musl-v2)
      MUSL=$(find_input musl-1.1.24-native-candidate)
      MAKE=$(find_input make-3.82-tcc)
      BINUTILS=$(find_input binutils-2.30-tcc-source-v1)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      M4=$(find_input m4-1.4.7-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      GAWK=$(find_input gawk-3.0.4-musl)
      AUTOMAKE115_SRC=$(find_input automake-1.15.1-config-sub-src)
      SRC=$(find_input gcc-4.0.4-native-src)
      TCC_RUNTIME="$TCC/lib/tcc/libtcc1.a"
      test -s "$TCC_RUNTIME"

      # Every PATH utility is a declared source-built derivation output. The
      # sandbox shell remains the explicit builder boundary, not a PATH fallback.
      TOOLBIN=/tmp/gcc40-tools
      $BB rm -rf "$TOOLBIN"
      $BB mkdir -p "$TOOLBIN"
      for applet in basename cat chmod cmp cp cut date dirname echo env expr false find head install ln ls mkdir mktemp mv od printf pwd readlink rm rmdir sh sha256sum sleep sort tail tee test timeout touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      export PATH="$TOOLBIN:$TCC/bin:$BINUTILS/bin:$MAKE/bin:$SED/bin:$M4/bin:$GREP/bin:$DIFF/bin:$GAWK/bin:$STAGE0/bin"
      export AWK="$GAWK/bin/gawk"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh

      CONFIGURE_PROBE_SOURCE_BYTES_MAX=65536
      CONFIGURE_PROBE_INVOCATION_COUNT_MAX=4096
      CONFIGURE_PROBE_AUDIT=/tmp/gcc40-configure-preprocess.audit
      CONFIGURE_PROBE_COUNT=/tmp/gcc40-configure-preprocess.count
      $BB rm -f "$CONFIGURE_PROBE_AUDIT" "$CONFIGURE_PROBE_COUNT"

      TCC_WRAPPER=/tmp/gcc40-tcc-wrapper
      $BB cat > "$TCC_WRAPPER" <<EOF
#!/bin/sh
REAL_TCC="$TCC/bin/tcc"
REAL_CPP_COMPILER="$TCC_SELFHOST/bin/tcc"
REAL_OLD_COMPILER="$TCC26/bin/tcc"
MUSL_INCLUDE="$MUSL/include"
MUSL_LIB="$MUSL/lib"
TCC_RUNTIME="$TCC/lib/tcc/libtcc1.a"
LINK_COMPAT_OBJECT=/tmp/gcc40-link-compat.o
PROGNAME_COMPAT_OBJECT=/tmp/gcc40-progname-compat.o
TEARDOWN_STATUS=$COMPILER_TEARDOWN_STATUS
mode=link
output_file=
source_file=
previous=
for argument in "\$@"; do
  if [ "\$previous" = -o ]; then output_file="\$argument"; fi
  case "\$argument" in
    -c) mode=compile ;;
    -E) mode=preprocess ;;
    --version|-V) exec "\$REAL_TCC" -v ;;
    *.c) source_file="\$argument" ;;
  esac
  previous="\$argument"
done
if [ "\$mode" = preprocess ]; then
  # Configure-only diagnostic bridge: TinyCC's -E parses Autoconf's deliberate
  # syntax-error sentinel and can finish without a preprocessed artifact. This
  # branch is admitted only for bounded conftest.c probes under authority supplied
  # by the active configure child. It is not a provider preprocessor.
  CONFIGURE_PROBE_SOURCE_BYTES_MAX=$CONFIGURE_PROBE_SOURCE_BYTES_MAX
  CONFIGURE_PROBE_INVOCATION_COUNT_MAX=$CONFIGURE_PROBE_INVOCATION_COUNT_MAX
  probe_dir="\${MANTLE_GCC40_CONFIGURE_PROBE_DIR:-}"
  probe_class="\${MANTLE_GCC40_CONFIGURE_PROBE_CLASS:-}"
  probe_audit="\${MANTLE_GCC40_CONFIGURE_PROBE_AUDIT:-}"
  probe_count_file="\${MANTLE_GCC40_CONFIGURE_PROBE_COUNT:-}"
  reject_probe() {
    echo "gcc40-tcc-wrapper: configure preprocess probe rejected: \$1" >&2
    exit 1
  }
  test -n "\$probe_dir" || reject_probe missing-authority-directory
  test -n "\$probe_audit" || reject_probe missing-audit-path
  test -n "\$probe_count_file" || reject_probe missing-count-path
  case "\$probe_class" in
    libiberty|libcpp|gcc) ;;
    *) reject_probe unknown-configure-class ;;
  esac
  current_dir=\$(/bin/busybox pwd -P) || reject_probe current-directory-unavailable
  test "\$current_dir" = "\$probe_dir" || reject_probe current-directory-mismatch
  case "\$source_file" in
    conftest.c|./conftest.c) ;;
    *) reject_probe non-conftest-source ;;
  esac
  test -z "\$output_file" || reject_probe explicit-output-forbidden
  source_canonical=\$(/bin/busybox readlink -f "\$source_file") || reject_probe source-canonicalization-failed
  source_parent=\$(/bin/busybox dirname "\$source_canonical") || reject_probe source-parent-unavailable
  test "\$source_parent" = "\$probe_dir" || reject_probe source-escape
  source_bytes=\$(/bin/busybox wc -c < "\$source_canonical") || reject_probe source-size-unavailable
  case "\$source_bytes" in
    ''|*[!0-9]*) reject_probe invalid-source-size ;;
  esac
  test "\$source_bytes" -le "\$CONFIGURE_PROBE_SOURCE_BYTES_MAX" || reject_probe source-too-large
  probe_count=0
  if [ -f "\$probe_count_file" ]; then
    probe_count=\$(/bin/busybox cat "\$probe_count_file") || reject_probe count-read-failed
  fi
  case "\$probe_count" in
    ''|*[!0-9]*) reject_probe invalid-count ;;
  esac
  test "\$probe_count" -lt "\$CONFIGURE_PROBE_INVOCATION_COUNT_MAX" || reject_probe invocation-budget-exhausted
  next_probe_count=\$((probe_count + 1))
  test "\$next_probe_count" -le "\$CONFIGURE_PROBE_INVOCATION_COUNT_MAX" || reject_probe invocation-budget-exhausted
  /bin/busybox printf '%s\n' "\$next_probe_count" > "\$probe_count_file" || reject_probe count-write-failed
  /bin/busybox printf '%s\t%s\t%s\t%s\n' \
    "\$next_probe_count" "\$probe_class" "\$probe_dir" "\$source_canonical" >> "\$probe_audit" || reject_probe audit-write-failed

  # The one macro-output probe is CHAR_BIT; bind it to the declared musl header
  # before emitting the token Autoconf expects.
  CHAR_BIT_EXPECTED=8
  if /bin/busybox grep -q '^#ifdef CHAR_BIT' "\$source_canonical"; then
    /bin/busybox grep -Eq "^#define[[:space:]]+CHAR_BIT[[:space:]]+\$CHAR_BIT_EXPECTED([[:space:]]|$)" "\$MUSL_INCLUDE/limits.h" || exit 1
    /bin/busybox printf 'found\n'
    exit 0
  fi
  scratch_source="/tmp/gcc40-cpp-source-\$\$.c"
  scratch_object="/tmp/gcc40-cpp-object-\$\$.o"
  /bin/busybox sed '/^[[:space:]]*Syntax error[[:space:]]*$/d' "\$source_canonical" > "\$scratch_source"
  /bin/busybox printf '\nint mantle_configure_cpp_probe(void) { return 0; }\n' >> "\$scratch_source"
  "\$REAL_TCC" -c -I"\$MUSL_INCLUDE" "\$scratch_source" -o "\$scratch_object"
  compiler_status=\$?
  if ! { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } || [ ! -s "\$scratch_object" ]; then
    /bin/busybox rm -f "\$scratch_object"
    echo "gcc40-tcc-wrapper: retry self-host compiler for preprocess probe \$source_canonical" >&2
    "\$REAL_CPP_COMPILER" -c -I"\$MUSL_INCLUDE" "\$scratch_source" -o "\$scratch_object"
    compiler_status=\$?
  fi
  if ! { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } || [ ! -s "\$scratch_object" ]; then
    /bin/busybox rm -f "\$scratch_object"
    echo "gcc40-tcc-wrapper: retry older compiler for preprocess probe \$source_canonical" >&2
    "\$REAL_OLD_COMPILER" -c -I"\$MUSL_INCLUDE" "\$scratch_source" -o "\$scratch_object"
    compiler_status=\$?
  fi
  if { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } && [ -s "\$scratch_object" ]; then
    /bin/busybox cat "\$source_canonical"
    /bin/busybox rm -f "\$scratch_source" "\$scratch_object"
    exit 0
  fi
  echo "gcc40-tcc-wrapper: preprocess probe failed status=\$compiler_status source=\$source_canonical" >&2
  /bin/busybox cat "\$scratch_source" >&2
  /bin/busybox ls -l "\$scratch_object" >&2 || true
  /bin/busybox rm -f "\$scratch_source" "\$scratch_object"
  exit "\$compiler_status"
fi
if [ "\$mode" = link ]; then
  if [ -z "\$output_file" ]; then output_file=a.out; fi
  link_inputs=
  for argument in "\$@"; do
    case "\$argument" in
      *.o|*.a|-l*|-L*) link_inputs="\$link_inputs \$argument" ;;
    esac
  done
  if [ -n "\$source_file" ]; then
    scratch_link_object="/tmp/gcc40-link-object-\$\$.o"
    "\$REAL_TCC" -c -I"\$MUSL_INCLUDE" "\$source_file" -o "\$scratch_link_object"
    compiler_status=\$?
    if ! { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } || [ ! -s "\$scratch_link_object" ]; then
      /bin/busybox rm -f "\$scratch_link_object"
      echo "gcc40-tcc-wrapper: retry self-host compiler for link source \$source_file" >&2
      "\$REAL_CPP_COMPILER" -c -I"\$MUSL_INCLUDE" "\$source_file" -o "\$scratch_link_object"
      compiler_status=\$?
      if ! { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } || [ ! -s "\$scratch_link_object" ]; then
        /bin/busybox rm -f "\$scratch_link_object"
        echo "gcc40-tcc-wrapper: retry older compiler for link source \$source_file" >&2
        "\$REAL_OLD_COMPILER" -c -I"\$MUSL_INCLUDE" "\$source_file" -o "\$scratch_link_object"
        compiler_status=\$?
        if ! { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } || [ ! -s "\$scratch_link_object" ]; then
          exit "\$compiler_status"
        fi
      fi
    fi
    /bin/busybox chmod u+rw "\$scratch_link_object"
    link_inputs="\$scratch_link_object \$link_inputs"
  fi
  progname_compat_input=
  case "\$output_file" in
    Tcollect2|xgcc|cpp) progname_compat_input="\$PROGNAME_COMPAT_OBJECT" ;;
  esac
  echo "gcc40-tcc-wrapper: link output=\$output_file inputs=\$link_inputs" >&2
  "\$REAL_TCC" -nostdlib -static -o "\$output_file" "\$MUSL_LIB/crt1.o" \
    \$link_inputs \$progname_compat_input "\$LINK_COMPAT_OBJECT" "\$MUSL_LIB/libc.a" "\$TCC_RUNTIME"
  compiler_status=\$?
  echo "gcc40-tcc-wrapper: link status=\$compiler_status" >&2
  if ! { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } || [ ! -s "\$output_file" ]; then
    if [ -n "\${scratch_link_object:-}" ] && [ -n "\$source_file" ]; then
      /bin/busybox rm -f "\$output_file" "\$scratch_link_object"
      echo "gcc40-tcc-wrapper: retry older compiler after rejected link object" >&2
      "\$REAL_OLD_COMPILER" -c -I"\$MUSL_INCLUDE" "\$source_file" -o "\$scratch_link_object"
      compiler_status=\$?
      echo "gcc40-tcc-wrapper: older link-source compile status=\$compiler_status" >&2
      if { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } && [ -s "\$scratch_link_object" ]; then
        /bin/busybox chmod u+rw "\$scratch_link_object"
        "\$REAL_OLD_COMPILER" -nostdlib -static -o "\$output_file" "\$MUSL_LIB/crt1.o" \
          \$link_inputs \$progname_compat_input "\$LINK_COMPAT_OBJECT" "\$MUSL_LIB/libc.a" "\$TCC_RUNTIME"
        compiler_status=\$?
        echo "gcc40-tcc-wrapper: older-object link status=\$compiler_status" >&2
      fi
    fi
  fi
  link_output_accepted=no
  if [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; then
    if [ -s "\$output_file" ]; then link_output_accepted=yes; fi
  fi
  if [ "\$link_output_accepted" = no ]; then
    /bin/busybox rm -f "\$output_file"
    echo "gcc40-tcc-wrapper: retry older compiler for rejected link output=\$output_file" >&2
    "\$REAL_OLD_COMPILER" -nostdlib -static -o "\$output_file" "\$MUSL_LIB/crt1.o" \
      \$link_inputs \$progname_compat_input "\$LINK_COMPAT_OBJECT" "\$MUSL_LIB/libc.a" "\$TCC_RUNTIME"
    compiler_status=\$?
    echo "gcc40-tcc-wrapper: older link status=\$compiler_status" >&2
  fi
  if [ -n "\${scratch_link_object:-}" ]; then /bin/busybox rm -f "\$scratch_link_object"; fi
  if { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } && [ -s "\$output_file" ]; then
    exit 0
  fi
  exit "\$compiler_status"
fi
if [ -z "\$output_file" ] && [ "\$mode" = compile ] && [ -n "\$source_file" ]; then
  source_name=\$(/bin/busybox basename "\$source_file")
  output_file="\${source_name%.c}.o"
fi
if [ "\$mode" = compile ] && [ "\${source_file##*/}" = charset.c ]; then
  # The source-built frontends crash while retaining the deeply nested GCC
  # include stack for this translation unit.  Preserve the exact preprocessed
  # C as a bounded diagnostic handoff; admission still forbids this bridge.
  preprocessed_source="/tmp/gcc40-charset-\$\$.i"
  preprocessor_args=
  for argument in "\$@"; do
    case "\$argument" in
      -c|-o|"\$output_file"|"\$source_file"|-W|-Wall|-Wwrite-strings|-Wstrict-prototypes|-Wmissing-prototypes|-Wold-style-definition) ;;
      *) preprocessor_args="\$preprocessor_args \$argument" ;;
    esac
  done
  echo "gcc40-tcc-wrapper: diagnostic preprocess handoff for \$source_file" >&2
  # Paths and defines in this generated command are sandbox-owned and contain
  # no whitespace; keep expansion visible instead of hiding it behind eval.
  "\$REAL_OLD_COMPILER" -E \$preprocessor_args "\$source_file" > "\$preprocessed_source"
  preprocessor_status=\$?
  if { [ "\$preprocessor_status" -eq 0 ] || [ "\$preprocessor_status" -eq "\$TEARDOWN_STATUS" ]; } && \
    /bin/busybox grep -q 'cpp_interpret_charconst' "\$preprocessed_source" && \
    /bin/busybox grep -q '_cpp_default_encoding' "\$preprocessed_source"; then
    "\$REAL_TCC" -c "\$preprocessed_source" -o "\$output_file"
    compiler_status=\$?
    if { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } && [ -s "\$output_file" ]; then
      /bin/busybox chmod u+rw "\$output_file"
      /bin/busybox rm -f "\$preprocessed_source"
      exit 0
    fi
  fi
  echo "gcc40-tcc-wrapper: diagnostic preprocess handoff failed preprocess=\$preprocessor_status compile=\${compiler_status:-not-run}" >&2
  /bin/busybox rm -f "\$preprocessed_source" "\$output_file"
fi
"\$REAL_TCC" "\$@"
compiler_status=\$?
if { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } && [ -n "\$output_file" ] && [ -s "\$output_file" ]; then
  /bin/busybox chmod u+rw "\$output_file"
  exit 0
fi
if [ "\$mode" = compile ]; then
  /bin/busybox rm -f "\$output_file"
  echo "gcc40-tcc-wrapper: retry self-host compiler for \$source_file" >&2
  "\$REAL_CPP_COMPILER" "\$@"
  compiler_status=\$?
  if { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } && [ -s "\$output_file" ]; then
    /bin/busybox chmod u+rw "\$output_file"
    exit 0
  fi
  /bin/busybox rm -f "\$output_file"
  echo "gcc40-tcc-wrapper: retry older compiler for \$source_file" >&2
  "\$REAL_OLD_COMPILER" "\$@"
  compiler_status=\$?
  if { [ "\$compiler_status" -eq 0 ] || [ "\$compiler_status" -eq "\$TEARDOWN_STATUS" ]; } && [ -s "\$output_file" ]; then
    /bin/busybox chmod u+rw "\$output_file"
    exit 0
  fi
fi
exit "\$compiler_status"
EOF
      $BB chmod 755 "$TCC_WRAPPER"

      # The inherited reduced libtcc1 archive omits the x86_64 unsigned
      # double-word-to-float helper that TinyCC emits for GCC objects.  Compile
      # the upstream TinyCC algorithm as an explicit diagnostic link object.
      # A complete compiler runtime must provide this itself before admission.
      $BB cat > /tmp/gcc40-link-compat.c <<'EOF'
typedef union {
  struct { unsigned int low; int high; } words;
  long long signed_value;
  unsigned long long unsigned_value;
} mantle_double_word;

float __floatundisf(unsigned long long value)
{
  mantle_double_word bits;
  long double converted;
  bits.unsigned_value = value;
  if (bits.words.high >= 0)
    return (float) bits.signed_value;
  converted = (long double) bits.signed_value;
  converted += 18446744073709551616.0;
  return (float) converted;
}

/* Diagnostic-only reduced-libc helpers required while cc1 runs. */
enum { MANTLE_PAGE_SIZE_BYTES = 4096 };
int getpagesize(void) { return MANTLE_PAGE_SIZE_BYTES; }
int __lsysinfo(void *information) { (void)information; return -1; }
int sem_init(void *semaphore, int shared, unsigned int value) { (void)semaphore; (void)shared; (void)value; return -1; }
int sem_wait(void *semaphore) { (void)semaphore; return -1; }
int sem_post(void *semaphore) { (void)semaphore; return -1; }
int sem_destroy(void *semaphore) { (void)semaphore; return 0; }
EOF
      "$TCC/bin/tcc" -c /tmp/gcc40-link-compat.c -o /tmp/gcc40-link-compat.o
      test -s /tmp/gcc40-link-compat.o || { echo "ERROR: GCC link compatibility object missing" >&2; exit 1; }
      $BB chmod u+rw /tmp/gcc40-link-compat.o
      $BB cp "$MUSL/lib/libc.a" /tmp/gcc40-indexed-libc.a
      $BB chmod u+rw /tmp/gcc40-indexed-libc.a
      "$BINUTILS/bin/ar" s /tmp/gcc40-indexed-libc.a
      "$BINUTILS/bin/ar" t /tmp/gcc40-indexed-libc.a > /tmp/gcc40-indexed-libc-members.txt
      test -s /tmp/gcc40-indexed-libc-members.txt || { echo "ERROR: indexed diagnostic libc has no members" >&2; exit 1; }
      $BB printf 'const char *progname = "gcc-bootstrap";\n' > /tmp/gcc40-progname-compat.c
      "$TCC/bin/tcc" -c /tmp/gcc40-progname-compat.c -o /tmp/gcc40-progname-compat.o
      test -s /tmp/gcc40-progname-compat.o || { echo "ERROR: GCC progname compatibility object missing" >&2; exit 1; }
      $BB chmod u+rw /tmp/gcc40-progname-compat.o

      prepare_source() {
        source_dir="$1"

        $BB sed -i 's/ix86_attribute_table\[\]/ix86_attribute_table[10]/' "$source_dir/gcc/config/i386/i386.c"
        $BB sed -i 's/struct siginfo/siginfo_t/' "$source_dir/gcc/config/i386/linux-unwind.h"
        $BB cat >> "$source_dir/gcc/system.h" <<'EOF'
#undef fprintf_unlocked
#define fprintf_unlocked fprintf
EOF
        # TinyCC's assembler accepts .space but rejects the equivalent ELF
        # .zero pseudo-op.  Select the supported spelling for this diagnostic
        # compiler handoff; admitted binutils must remove this compatibility.
        $BB sed -i 's/^#define SKIP_ASM_OP[[:space:]]*"\\t\.zero\\t"$/#define SKIP_ASM_OP "\\t.space\\t"/' "$source_dir/gcc/config/elfos.h"
        $BB grep -q '^#define SKIP_ASM_OP "\\t.space\\t"$' "$source_dir/gcc/config/elfos.h"
        $BB sed -i 's/movabs{/mov{/g; s/sal{/shl{/g; s/   {cltq|cdqe}/   movs{lq|x}\\t{%1,%0|%0, %1}/' "$source_dir/gcc/config/i386/i386.md"
        if $BB grep -q 'movabs{\|sal{\|{cltq|cdqe}' "$source_dir/gcc/config/i386/i386.md"; then
          echo "ERROR: unsupported TinyCC assembler spelling remained" >&2
          exit 1
        fi
        $BB sed -i 's/^EXTRA_MULTILIB_PARTS=crtbegin.o crtend.o crtbeginS.o crtendS.o crtbeginT.o$/EXTRA_MULTILIB_PARTS=crtbegin.o crtend.o crtbeginT.o/' "$source_dir/gcc/config/i386/t-linux64"
        $BB grep -q '^EXTRA_MULTILIB_PARTS=crtbegin.o crtend.o crtbeginT.o$' "$source_dir/gcc/config/i386/t-linux64"
        $BB cat >> "$source_dir/gcc/config/elfos.h" <<'EOF'

/* Diagnostic TinyCC assembler compatibility: avoid .local plus .comm.  */
#undef ASM_OUTPUT_ALIGNED_LOCAL
#define ASM_OUTPUT_ALIGNED_LOCAL(FILE, NAME, SIZE, ALIGN)             \
  do                                                                  \
    {                                                                 \
      bss_section ();                                                 \
      ASM_OUTPUT_ALIGN ((FILE), floor_log2 ((ALIGN) / BITS_PER_UNIT)); \
      ASM_OUTPUT_LABEL ((FILE), (NAME));                              \
      ASM_OUTPUT_SKIP ((FILE), (SIZE));                               \
    }                                                                 \
  while (0)
EOF
        $BB awk '
          /^#define FIBHEAPKEY_MIN/ {
            print "#ifndef LONG_MAX"
            print "#define LONG_MAX ((long)(~0UL >> 1))"
            print "#endif"
            print "#ifndef LONG_MIN"
            print "#define LONG_MIN (-LONG_MAX - 1L)"
            print "#endif"
          }
          { print }
        ' "$source_dir/libiberty/fibheap.c" > "$source_dir/libiberty/fibheap.c.tmp"
        $BB mv "$source_dir/libiberty/fibheap.c.tmp" "$source_dir/libiberty/fibheap.c"
        $BB printf '#include <stddef.h>\nvoid *malloc(size_t);\nvoid *calloc(size_t, size_t);\nvoid *realloc(void *, size_t);\nvoid free(void *);\nvoid abort(void);\n' > "$source_dir/libiberty/hashtab.c.tmp"
        $BB cat "$source_dir/libiberty/hashtab.c" >> "$source_dir/libiberty/hashtab.c.tmp"
        $BB mv "$source_dir/libiberty/hashtab.c.tmp" "$source_dir/libiberty/hashtab.c"
        # The predecessor cannot size the old cached stat-based getpwd source.
        # Keep a bounded getcwd implementation for frontier diagnosis only.
        $BB cat > "$source_dir/libiberty/getpwd.c" <<'EOF'
#include <unistd.h>
enum { mantle_getpwd_capacity_bytes = 4096 };
char *getpwd(void)
{
  static char path[mantle_getpwd_capacity_bytes];
  if (!getcwd(path, sizeof(path))) return 0;
  return path;
}
EOF

        # Rehome the exact generated UCN table into its own translation unit.
        # TinyCC currently crashes when charset.c combines this large static
        # initializer with the conversion implementation.  This preserves the
        # table and binary-search semantics, but remains a diagnostic source
        # normalization that must disappear before provider admission.
        $BB cp "$source_dir/libcpp/ucnid.h" "$source_dir/libcpp/ucnid-table.c"
        $BB sed -i 's/static const struct ucnrange ucnranges\[\]/const struct ucnrange ucnranges[]/' "$source_dir/libcpp/ucnid-table.c"
        $BB sed -i '/#endif \/\* LIBCPP_UCNID_H \*\//i const unsigned int ucnranges_count = sizeof (ucnranges) / sizeof (ucnranges[0]);' "$source_dir/libcpp/ucnid-table.c"
        $BB cat > "$source_dir/libcpp/ucnid.h" <<'EOF'
#ifndef LIBCPP_UCNID_H
#define LIBCPP_UCNID_H
#define C99 1
#define CXX 2
#define DIG 4
struct ucnrange
{
  unsigned short lo, hi;
  unsigned short flags;
};
extern const struct ucnrange ucnranges[];
extern const unsigned int ucnranges_count;
#endif
EOF
        $BB sed -i 's/ARRAY_SIZE (ucnranges)/ucnranges_count/' "$source_dir/libcpp/charset.c"
        # Normalize optimizer-only forms that exercise known predecessor
        # frontend faults.  These changes preserve callable linkage, branch
        # conditions, table values, and conversion behavior.
        $BB sed -i 's/^static inline /static /' "$source_dir/libcpp/charset.c"
        $BB sed -i 's/int (\*const one_conversion)/int (*one_conversion)/' "$source_dir/libcpp/charset.c"
        $BB sed -i 's/if (__builtin_expect (inbytesleft == 0, 1))/if (inbytesleft == 0)/' "$source_dir/libcpp/charset.c"
        $BB sed -i 's/static const struct conversion conversion_tab\[\]/static struct conversion conversion_tab[]/' "$source_dir/libcpp/charset.c"
        $BB sed -i 's/static const uchar /static uchar /' "$source_dir/libcpp/charset.c"
        $BB sed -i 's/libcpp_a_OBJS = charset.o /libcpp_a_OBJS = charset.o ucnid-table.o /' "$source_dir/libcpp/Makefile.in"
        $BB sed -i 's/libcpp_a_SOURCES = charset.c /libcpp_a_SOURCES = charset.c ucnid-table.c /' "$source_dir/libcpp/Makefile.in"
        $BB printf '\nucnid-table.o: $(srcdir)/ucnid-table.c\n' >> "$source_dir/libcpp/Makefile.in"

        # TinyCC eagerly loads archive members instead of selecting only those
        # needed to satisfy unresolved symbols.  Remove one exact duplicate
        # backend member, the mutually exclusive no-mudflap member, and the
        # repeated libcpp archive argument for this diagnostic link.  A real
        # source-built linker must restore ordinary archive semantics.
        $BB sed -i 's/ insn-preds.o pointer-set.o postreload.o/ insn-preds.o pointer-set.o/' "$source_dir/gcc/Makefile.in"
        $BB sed -i 's/  cgraph.o cgraphunit.o tree-nomudflap.o/  cgraph.o cgraphunit.o/' "$source_dir/gcc/Makefile.in"
        $BB sed -i 's/BACKEND = main.o @TREEBROWSER@ libbackend.a $(CPPLIB)/BACKEND = main.o @TREEBROWSER@ libbackend.a/' "$source_dir/gcc/Makefile.in"
        $BB grep -q '^BACKEND = main.o @TREEBROWSER@ libbackend.a$' "$source_dir/gcc/Makefile.in"
        $BB grep -q '^COLLECT2_OBJS = collect2.o tlink.o intl.o version.o$' "$source_dir/gcc/Makefile.in"

        # Native TinyCC miscompiles libiberty's relocatable-prefix walker and
        # the resulting driver crashes before argument processing.  Disable
        # that optional discovery in this diagnostic pass; every pass1 driver
        # invocation remains bound explicitly with -B and this bridge is not
        # eligible for provider admission.
        $BB sed -i 's/make_relative_prefix (/bootstrap_make_relative_prefix (/g' "$source_dir/gcc/gcc.c"
        $BB awk '
          /\/\* Create the vector `switches/ && !inserted {
            print "static char *";
            print "bootstrap_make_relative_prefix (const char *progname, const char *bin_prefix, const char *prefix)";
            print "{";
            print "  (void) progname;";
            print "  (void) bin_prefix;";
            print "  (void) prefix;";
            print "  return 0;";
            print "}";
            print "";
            inserted = 1;
          }
          { print }
          END { if (!inserted) exit 1 }
        ' "$source_dir/gcc/gcc.c" > "$source_dir/gcc/gcc.c.bootstrap"
        $BB mv "$source_dir/gcc/gcc.c.bootstrap" "$source_dir/gcc/gcc.c"
        [ "$($BB grep -c 'bootstrap_make_relative_prefix (' "$source_dir/gcc/gcc.c")" -eq 5 ]

        # Use the release tarball's generated configure, parser, and Unicode
        # tables.  Regenerating them belongs to the separate Perl/autotools
        # repair and is not a prerequisite for diagnosing GCC's C build.
        cd "$source_dir"
        $BB cp "$AUTOMAKE115_SRC/lib/config.sub" config.sub
        test -f gcc/c-parse.c || { echo "ERROR: shipped c-parse.c missing" >&2; exit 1; }
        test -f gcc/gengtype-yacc.c || { echo "ERROR: shipped gengtype parser missing" >&2; exit 1; }
        test -f gcc/gengtype-yacc.h || { echo "ERROR: shipped gengtype parser header missing" >&2; exit 1; }
        test -f gcc/gengtype-lex.c || { echo "ERROR: shipped gengtype lexer missing" >&2; exit 1; }
        test -f libcpp/ucnid-table.c || { echo "ERROR: shipped Unicode table missing" >&2; exit 1; }
        # Source staging does not preserve the release archive's generated-file
        # ordering.  Keep shipped parser/lexer artifacts newer than their
        # grammars for this no-Bison/no-Flex diagnostic pass.
        $BB touch gcc/c-parse.c gcc/gengtype-yacc.c gcc/gengtype-yacc.h gcc/gengtype-lex.c
        # The pass1 cc1 runtime reaches all integer built-ins but crashes in
        # the first floating-constant definition.  The current TinyCC/musl
        # predecessor is already known to miscompute floating arithmetic and
        # formatting, so omit only these three generated macro groups for this
        # diagnostic pass.  They are mandatory before provider admission.
        $BB sed -i 's/^  builtin_define_float_constants ("FLT", "F", float_type_node);$/  \/\* diagnostic omission: FLT built-ins \*\//' gcc/c-cppbuiltin.c
        $BB sed -i 's/^  builtin_define_float_constants ("DBL", "", double_type_node);$/  \/\* diagnostic omission: DBL built-ins \*\//' gcc/c-cppbuiltin.c
        $BB sed -i 's/^  builtin_define_float_constants ("LDBL", "L", long_double_type_node);$/  \/\* diagnostic omission: LDBL built-ins \*\//' gcc/c-cppbuiltin.c
        [ "$($BB grep -c 'diagnostic omission: .* built-ins' gcc/c-cppbuiltin.c)" -eq 3 ]
        $BB find . -name '*.gmo' -delete
        $BB find gcc/doc -name '*.info' -delete
        $BB find gcc/doc -name '*.1' -delete
        $BB find gcc/doc -name '*.7' -delete
      }

      build_pass() {
        source_dir="$1"
        build_dir="$2"
        prefix_dir="$3"
        compiler="$4"

        $BB mkdir -p "$build_dir/gcc"
        cd "$build_dir"
        compiler_include_flags=
        if [ "$compiler" != "$TCC_WRAPPER" ]; then
          $BB mkdir -p "$build_dir/gcc-compat-include"
          $BB cat > "$build_dir/gcc-compat-include/stdarg.h" <<'EOF'
#ifndef MANTLE_GCC40_STDARG_H
#define MANTLE_GCC40_STDARG_H
#ifndef __GNUC_VA_LIST
#define __GNUC_VA_LIST 1
typedef __builtin_va_list __gnuc_va_list;
#endif
#ifndef __DEFINED_va_list
typedef __builtin_va_list va_list;
#define __DEFINED_va_list 1
#endif
#ifndef __DEFINED___isoc_va_list
typedef __builtin_va_list __isoc_va_list;
#define __DEFINED___isoc_va_list 1
#endif
#define va_start(value, last) __builtin_va_start(value, last)
#define va_end(value) __builtin_va_end(value)
#define va_arg(value, type) __builtin_va_arg(value, type)
#define va_copy(destination, source) __builtin_va_copy(destination, source)
#define __va_copy(destination, source) __builtin_va_copy(destination, source)
#endif
EOF
          compiler_include_flags="-I$build_dir/gcc-compat-include"
        fi
        # Use the isolated source-built GNU assembler candidate. Its own
        # derivation requires version, SSE, and malformed-input runtime checks.
        $BB cat > gcc/as <<EOF
#!/bin/sh
assembler_arguments=
for assembler_argument in "\$@"; do
  case "\$assembler_argument" in
    -Qy) ;;
    *) assembler_arguments="\$assembler_arguments \$assembler_argument" ;;
  esac
done
exec "$BINUTILS/bin/as" \$assembler_arguments
EOF
        $BB chmod 755 gcc/as
        # Pair the source-built assembler with the isolated source-built GNU
        # linker candidate for GCC-produced objects. The source BFD archive
        # parser and source armap writer are checked by the binutils derivation,
        # so archives stay archives at this boundary.
        $BB cat > gcc/ld <<EOF
#!/bin/sh
link_arguments=
for link_argument in "\$@"; do
  case "\$link_argument" in
    -lgcc|-lgcc_eh|*/libgcc.a|*/libtcc1.a|*/crtbegin*.o|*/crtend*.o|crtbegin*.o|crtend*.o) ;;
    crt1.o|crti.o|crtn.o) link_arguments="\$link_arguments $MUSL/lib/\$link_argument" ;;
    -lc) link_arguments="\$link_arguments /tmp/gcc40-indexed-libc.a" ;;
    -lm) ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
echo "gcc40-ld: $BINUTILS/bin/ld \$link_arguments /tmp/gcc40-link-compat.o" >&2
exec "$BINUTILS/bin/ld" \$link_arguments /tmp/gcc40-link-compat.o
EOF
        $BB chmod 755 gcc/ld

        for dir in libiberty libcpp gcc; do
          $BB mkdir -p "$dir"
          cd "$dir"
          configure_probe_dir=$($BB pwd -P)
          set +e
          ac_cv_c_bigendian=no \
            MANTLE_GCC40_CONFIGURE_PROBE_DIR="$configure_probe_dir" \
            MANTLE_GCC40_CONFIGURE_PROBE_CLASS="$dir" \
            MANTLE_GCC40_CONFIGURE_PROBE_AUDIT="$CONFIGURE_PROBE_AUDIT" \
            MANTLE_GCC40_CONFIGURE_PROBE_COUNT="$CONFIGURE_PROBE_COUNT" \
            CC="$compiler" \
            CPP="$compiler -E $compiler_include_flags -I$MUSL/include" \
            CPPFLAGS="$compiler_include_flags -I$MUSL/include" \
            AWK="$BB awk" \
            AR="$BINUTILS/bin/ar" \
            RANLIB="$BINUTILS/bin/ar s" \
            CFLAGS="-D HAVE_ALLOCA_H $compiler_include_flags -I$MUSL/include $GCC_ASSEMBLER_COMPAT_FLAGS" \
            LDFLAGS="-static -L$MUSL/lib" \
            "$source_dir/$dir/configure" \
              --prefix="$prefix_dir" \
              --libdir="$prefix_dir/lib" \
              --build="$TARGET" \
              --target="$TARGET" \
              --host="$TARGET" \
              --disable-shared \
              --program-transform-name=
          configure_status=$?
          set -e
          if [ "$configure_status" -ne 0 ]; then
            echo "ERROR: $dir configure failed with status $configure_status" >&2
            $BB cat config.log >&2
            exit "$configure_status"
          fi
          cd ..
        done

        if [ "$compiler" = "$TCC_WRAPPER" ]; then
          test -s "$CONFIGURE_PROBE_AUDIT" || {
            echo "ERROR: GCC configure preprocess probe produced no audit" >&2
            exit 1
          }
          configure_probe_audit_count=$($BB wc -l < "$CONFIGURE_PROBE_AUDIT")
          test "$configure_probe_audit_count" -le "$CONFIGURE_PROBE_INVOCATION_COUNT_MAX" || {
            echo "ERROR: GCC configure preprocess probe audit exceeds invocation bound" >&2
            exit 1
          }
          while IFS="$($BB printf '\t')" read -r audit_index audit_class audit_dir audit_source; do
            case "$audit_class" in
              libiberty|libcpp|gcc) ;;
              *) echo "ERROR: unknown GCC configure probe audit class: $audit_class" >&2; exit 1 ;;
            esac
            expected_audit_dir="$build_dir/$audit_class"
            test "$audit_dir" = "$expected_audit_dir" || {
              echo "ERROR: GCC configure probe audit directory mismatch: $audit_dir" >&2
              exit 1
            }
            test "$audit_source" = "$expected_audit_dir/conftest.c" || {
              echo "ERROR: GCC configure probe audit source mismatch: $audit_source" >&2
              exit 1
            }
            test -n "$audit_index" || { echo "ERROR: GCC configure probe audit index missing" >&2; exit 1; }
          done < "$CONFIGURE_PROBE_AUDIT"
        fi

        # The compile-backed configure preprocessor cannot expand config.status
        # templates faithfully.  Bind the target-derived 64-bit host integer
        # type, then restore clock/times declarations and remove fallback
        # typedef macros for types that musl already supplies.  The upstream
        # preprocessor path must produce these results without correction
        # before admission.
        if [ "$compiler" != "$TCC_WRAPPER" ]; then
          for libiberty_host_fact in \
            HAVE_MEMCPY HAVE_STDLIB_H HAVE_STRING_H HAVE_STRINGS_H \
            HAVE_PSIGNAL HAVE_STRSIGNAL HAVE_SYS_STAT_H HAVE_SYS_TYPES_H \
            HAVE_UINTPTR_T HAVE_UNISTD_H STDC_HEADERS; do
            $BB sed -i "s/^#undef $libiberty_host_fact$/#define $libiberty_host_fact 1/" libiberty/config.h
            $BB grep -q "^#define $libiberty_host_fact 1$" libiberty/config.h
          done
        fi
        HOST_INT_BYTES=4
        HOST_LONG_BYTES=8
        $BB sed -i \
          -e '/^#define SIZEOF_INT/d' \
          -e '/^#define SIZEOF_LONG/d' \
          -e 's/^#undef HOST_WIDE_INT$/#define HOST_WIDE_INT long/' \
          libcpp/config.h
        $BB printf '#define SIZEOF_INT %s\n#define SIZEOF_LONG %s\n' \
          "$HOST_INT_BYTES" "$HOST_LONG_BYTES" >> libcpp/config.h
        $BB grep -q '^#define HOST_WIDE_INT long$' libcpp/config.h || {
          echo "ERROR: libcpp HOST_WIDE_INT correction missing" >&2
          return 1
        }
        $BB sed -i '/HAVE_CLOCK_T/d; /HAVE_STRUCT_TMS/d' gcc/auto-host.h
        $BB printf '%s\n' '#define HAVE_CLOCK_T 1' '#define HAVE_STRUCT_TMS 1' >> gcc/auto-host.h
        $BB grep -q '^#define HAVE_CLOCK_T 1$' gcc/auto-host.h || {
          echo "ERROR: gcc clock_t correction missing" >&2
          return 1
        }
        $BB grep -q '^#define HAVE_STRUCT_TMS 1$' gcc/auto-host.h || {
          echo "ERROR: gcc struct tms correction missing" >&2
          return 1
        }
        $BB sed -i "s|^SYSTEM_HEADER_DIR =.*$|SYSTEM_HEADER_DIR = $MUSL/include|" gcc/Makefile
        $BB grep -q "^SYSTEM_HEADER_DIR = $MUSL/include$" gcc/Makefile || {
          echo "ERROR: GCC system header correction missing" >&2
          return 1
        }
        for system_type in gid_t pid_t rlim_t uid_t; do
          $BB sed -i "/^#define $system_type /d" gcc/auto-host.h
          if $BB grep -q "^#define $system_type " gcc/auto-host.h; then
            echo "ERROR: configure fallback type remains defined: $system_type" >&2
            exit 1
          fi
        done

        $BB ln -s . "build-$TARGET"
        # The source-built fixincludes chain is not available yet.  Provide a
        # fail-bounded no-op only for this direct-musl diagnostic because every
        # compiler invocation receives the declared musl include directory.
        $BB mkdir -p fixincludes
        $BB printf '#!/bin/sh\nexit 0\n' > fixincludes/fixinc.sh
        $BB chmod 755 fixincludes/fixinc.sh
        $BB mkdir -p gcc/include
        $BB ln -s "$source_dir/gcc/gsyslimits.h" gcc/include/syslimits.h
        $BB rm -f gcc/c-parse.c
        $BB ln -s "$source_dir/gcc/c-parse.c" gcc/c-parse.c
        make -j"$JOB_COUNT" -C gcc gengtype-yacc.c \
          CC="$compiler" \
          AR="$BINUTILS/bin/ar" \
          RANLIB="$BINUTILS/bin/ar s" || { make_status=$?; set +e; return "$make_status"; }

        for dir in libiberty libcpp gcc; do
          make -j"$JOB_COUNT" -C "$dir" \
            CC="$compiler" \
            AR="$BINUTILS/bin/ar" \
            RANLIB="$BINUTILS/bin/ar s" \
            T_CFLAGS="$compiler_include_flags -I$MUSL/include $GCC_ASSEMBLER_COMPAT_FLAGS" \
            LIBGCC2_INCLUDES="-I$MUSL/include" \
            EXTRA_PARTS="crtbegin.o crtend.o crtbeginT.o" \
            EXTRA_MULTILIB_PARTS="crtbegin.o crtend.o crtbeginT.o" \
            AWK="$BB awk" \
            LIBGCC= \
            INSTALL_LIBGCC= \
            STMP_FIXINC= \
            MAKEINFO=true || { make_status=$?; set +e; return "$make_status"; }
        done

        $BB mkdir -p "$prefix_dir/lib/gcc/$TARGET/$GCC_VERSION/install-tools/include"
        make -j"$JOB_COUNT" -C gcc install \
          CC="$compiler" \
          AR="$BINUTILS/bin/ar" \
          RANLIB="$BINUTILS/bin/ar s" \
          T_CFLAGS="$compiler_include_flags -I$MUSL/include $GCC_ASSEMBLER_COMPAT_FLAGS" \
          EXTRA_PARTS="crtbegin.o crtend.o crtbeginT.o" \
          EXTRA_MULTILIB_PARTS="crtbegin.o crtend.o crtbeginT.o" \
          LIBGCC= \
          INSTALL_LIBGCC= \
          INSTALL_HEADERS_DIR=install-headers-cp \
          STMP_FIXINC= \
          MAKEINFO=true \
          DESTDIR= || { make_status=$?; set +e; return "$make_status"; }
        $BB mkdir -p "$prefix_dir/lib/gcc/$TARGET/$GCC_VERSION"
        $BB cp "$TCC_RUNTIME" "$prefix_dir/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
        test -s "$prefix_dir/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
        $BB mkdir -p "$prefix_dir/lib/gcc/$TARGET/$GCC_VERSION/include"
        $BB rm -f "$prefix_dir/lib/gcc/$TARGET/$GCC_VERSION/include/syslimits.h"
        $BB cp "$source_dir/gcc/gsyslimits.h" "$prefix_dir/lib/gcc/$TARGET/$GCC_VERSION/include/syslimits.h"
      }

      $BB rm -rf "$WORK" "$PASS1"
      $BB mkdir -p "$WORK"

      $BB cp -r "$SRC" "$WORK/pass1-src"
      $BB chmod -R u+w "$WORK/pass1-src"
      prepare_source "$WORK/pass1-src"
      build_pass "$WORK/pass1-src" "$WORK/pass1-build" "$PASS1" "$TCC_WRAPPER"

      test -x "$PASS1/bin/gcc" || { echo "ERROR: pass1 gcc missing" >&2; exit 1; }
      test -x "$PASS1/libexec/gcc/$TARGET/$GCC_VERSION/cc1" || { echo "ERROR: pass1 cc1 missing" >&2; exit 1; }

      export PATH="$PASS1/bin:$PATH"
      $BB cp -r "$SRC" "$WORK/pass2-src"
      $BB chmod -R u+w "$WORK/pass2-src"
      prepare_source "$WORK/pass2-src"
      set +e
      build_pass "$WORK/pass2-src" "$WORK/pass2-build" "$out" "$PASS1/bin/gcc -B$WORK/pass2-build/gcc/"
      pass2_status=$?
      set -e
      if [ "$pass2_status" -ne 0 ]; then
        $BB rm -rf "$out"
        $BB mkdir -p "$out/share/pass2-frontier"
        $BB cp -r "$PASS1" "$out/share/pass2-frontier/pass1"
        $BB cp "$WORK/pass2-src/libiberty/regex.c" "$out/share/pass2-frontier/regex.c"
        $BB cp "$WORK/pass2-build/libiberty/config.h" "$out/share/pass2-frontier/config.h"
        if [ -s "$WORK/pass2-build/gcc/assembler-failure.s" ]; then
          $BB cp "$WORK/pass2-build/gcc/assembler-failure.s" "$out/share/pass2-frontier/rejected.s"
          $BB cp "$WORK/pass2-build/gcc/assembler-failure.status" "$out/share/pass2-frontier/assembler-status"
        fi
        $BB printf '%s\n' "$pass2_status" > "$out/share/pass2-frontier/pass2-status"
        $BB printf '%s\n' 'NON_ADMISSION: pass2 compiler diagnostic frontier' > "$out/share/pass2-frontier/NON_ADMISSION"
        echo "gcc-4.0.4 diagnostic pass2 compiler frontier exported; not provider admission"
        exit 0
      fi
      [ "$pass2_status" -eq 0 ] || exit "$pass2_status"

      test -x "$out/bin/gcc" || { echo "ERROR: native gcc missing" >&2; exit 1; }
      test -x "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1" || { echo "ERROR: native cc1 missing" >&2; exit 1; }
      ELF_MAGIC_HEX=7f454c46
      gcc_magic=$($BB od -An -tx1 -N4 "$out/bin/gcc" | $BB tr -d ' ')
      cc1_magic=$($BB od -An -tx1 -N4 "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1" | $BB tr -d ' ')
      test "$gcc_magic" = "$ELF_MAGIC_HEX" || { echo "ERROR: native gcc is not ELF" >&2; exit 1; }
      test "$cc1_magic" = "$ELF_MAGIC_HEX" || { echo "ERROR: native cc1 is not ELF" >&2; exit 1; }
      "$BINUTILS/bin/readelf" -h "$out/bin/gcc" > /tmp/gcc40-gcc-header.txt
      "$BINUTILS/bin/readelf" -h "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1" > /tmp/gcc40-cc1-header.txt
      $BB grep -q 'ELF' /tmp/gcc40-gcc-header.txt
      $BB grep -q 'ELF' /tmp/gcc40-cc1-header.txt
      predecessor_delegation_marker="Mantle pass1 b""ridge"
      if "$BB" grep -a -q "$predecessor_delegation_marker" "$out/bin/gcc" "$out/libexec/gcc/$TARGET/$GCC_VERSION/cc1"; then
        echo "ERROR: predecessor delegation marker present in native provider" >&2
        exit 1
      fi

      GCC_TOOL_PREFIX="$out/libexec/gcc/$TARGET/$GCC_VERSION/"
      "$out/bin/gcc" -B"$GCC_TOOL_PREFIX" -v >/tmp/gcc40-version.out 2>/tmp/gcc40-version.err
      "$out/bin/gcc" -B"$GCC_TOOL_PREFIX" --help >/tmp/gcc40-help.out
      "$out/bin/gcc" -B"$GCC_TOOL_PREFIX" -dumpspecs >/tmp/gcc40-specs.out
      test -s /tmp/gcc40-help.out
      test -s /tmp/gcc40-specs.out

      $BB rm -rf /tmp/gcc40-final-include
      $BB mkdir -p /tmp/gcc40-final-include
      $BB cp "$out/lib/gcc/$TARGET/$GCC_VERSION/include/stdarg.h" /tmp/gcc40-final-include/stdarg.h
      $BB cat > /tmp/gcc40-native-smoke.c << 'EOF'
#include <stdarg.h>

enum {
  NEGATIVE_VALUE = -7,
  VARARG_COUNT = 3,
  VARARG_FIRST = 5,
  VARARG_SECOND = 8,
  VARARG_THIRD = 13,
  VARARG_EXPECTED = 26
};
#define MAX_UNSIGNED_WORD 18446744073709551615ULL
#define ROUNDED_MAX_FLOAT 18446744073709551616.0f

int diagnostic_common_global;
extern float __floatundisf(unsigned long long value);

static long widen_signed(int value) { return value; }
static int sum_values(int count, ...)
{
  va_list arguments;
  int index;
  int result = 0;
  va_start(arguments, count);
  for (index = 0; index < count; ++index) result += va_arg(arguments, int);
  va_end(arguments);
  return result;
}

int main(void)
{
  if (widen_signed(NEGATIVE_VALUE) != NEGATIVE_VALUE) return 1;
  if (diagnostic_common_global != 0) return 1;
  if (sum_values(VARARG_COUNT, VARARG_FIRST, VARARG_SECOND, VARARG_THIRD) != VARARG_EXPECTED) return 1;
  if (__floatundisf(0) != 0.0f) return 1;
  if (__floatundisf(1) != 1.0f) return 1;
  if (__floatundisf(MAX_UNSIGNED_WORD) != ROUNDED_MAX_FLOAT) return 1;
  return 0;
}
EOF
      "$out/bin/gcc" -B"$GCC_TOOL_PREFIX" -B"$WORK/pass2-build/gcc/" \
        -I/tmp/gcc40-final-include -I"$MUSL/include" \
        $GCC_ASSEMBLER_COMPAT_FLAGS -static \
        /tmp/gcc40-native-smoke.c -o /tmp/gcc40-native-smoke
      test -s /tmp/gcc40-native-smoke || { echo "ERROR: native gcc executable smoke missing" >&2; exit 1; }
      smoke_magic=$($BB od -An -tx1 -N4 /tmp/gcc40-native-smoke | $BB tr -d ' ')
      test "$smoke_magic" = "$ELF_MAGIC_HEX" || { echo "ERROR: native gcc smoke is not ELF" >&2; exit 1; }
      /tmp/gcc40-native-smoke

      $BB printf '%s\n' 'int main(' > /tmp/gcc40-native-rejected.c
      $BB rm -f /tmp/gcc40-native-rejected.o
      if "$out/bin/gcc" -B"$GCC_TOOL_PREFIX" -B"$WORK/pass2-build/gcc/" \
        -I"$MUSL/include" $GCC_ASSEMBLER_COMPAT_FLAGS \
        -c /tmp/gcc40-native-rejected.c -o /tmp/gcc40-native-rejected.o; then
        echo "ERROR: native gcc accepted malformed C" >&2
        exit 1
      fi
      test ! -e /tmp/gcc40-native-rejected.o

      $BB mkdir -p "$out/share/gcc-frontier"
      $BB printf '%s\n' 'NON_ADMISSION: GCC pass2 runs only with reduced-libc stubs and diagnostic linker shell' > "$out/share/gcc-frontier/NON_ADMISSION"
      echo "gcc-4.0.4 diagnostic two-pass build complete; not provider admission"
    "%
  ],
  inputs = [
    binutils, tcc, tcc_selfhost, tcc26, musl, make, stage0, sed, m4, grep, diffutils,
    gawk, automake115_src, gcc_src,
  ],
} | crunch.Derivation
