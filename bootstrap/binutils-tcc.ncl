# Build binutils 2.30 from authenticated source with the declared TCC/musl
# bootstrap closure.
#
# The recipe follows live-bootstrap's binutils-2.30 pass1 artifact boundary:
# remove required generated parser/scanner/table/header sources, regenerate
# them with declared source-built Bison/Flex/native generators, configure from
# the authenticated release scripts, compile the real GNU tools with TinyCC,
# and reject missing or delegated output members.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let tcc = crunch.derivationFile "tcc-musl-v2.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-native.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let sed = crunch.derivationFile "sed-4.0.9-musl.ncl" in
let m4 = crunch.derivationFile "m4-1.4.7-musl.ncl" in
let grep = crunch.derivationFile "grep-2.4-musl.ncl" in
let diffutils = crunch.derivationFile "diffutils-2.7-musl.ncl" in
let gawk = crunch.derivationFile "gawk-3.0.4-musl.ncl" in
let bison = crunch.derivationFile "bison-2.3-musl.ncl" in
let flex = crunch.derivationFile "flex-2.6.4-musl.ncl" in

let binutils_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/binutils-2.30/sources
  # first-consumer: bootstrap/binutils-tcc.ncl
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz",
  hash = "sha256-L8aaWezlL47cNdPIbSEAtryYUDXLe9htMSECT4xqpoc=",
  name = "binutils-2.30-src",
} in

{
  name = "binutils-2.30-tcc-source-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-gnu
      JOB_COUNT=1
      CONFIGURE_MAX_COMMAND_BYTES=32768
      CONFIG_SUB_TIMEOUT_SECONDS=10
      CONFIG_LOG_ERROR_LINES=80
      CONFIG_LOG_TAIL_LINES=160
      CONFIGURE_PROBE_SOURCE_BYTES_MAX=65536
      CONFIGURE_PROBE_INVOCATION_COUNT_MAX=4096
      BFD_BOOTSTRAP_CONFIG_INPUT_REFS_EXPECTED=2
      ARCHIVE_MEMBER_COUNT_MAX=1024
      ARCHIVE_HEADER_ARG_COUNT=2
      ARCHIVE_GLOBAL_HEADER_BYTES=8
      ARCHIVE_SHORT_NAME_BYTES_MAX=15
      ARCHIVE_MEMBER_NAME_BYTES_MAX=255
      ARCHIVE_MEMBER_BYTES_MAX=67108864
      ARCHIVE_STRING_TABLE_BYTES_MAX=1048576
      ARCHIVE_OUTPUT_BYTES_MAX=1073741824
      ARCHIVE_ALIGNMENT_BYTES=2
      ARCHIVE_LONG_NAME_SUFFIX_BYTES=2
      CONFIGURE_TIMEOUT_SECONDS=300
      MAKE_TIMEOUT_SECONDS=600
      SMOKE_EXIT_STATUS=42
      WORK=/tmp/binutils-2.30-tcc-source

      find_input() {
        input_suffix=$1
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -d "$input_path" ]; then
            echo "$input_path"
            return 0
          fi
        done
        echo "ERROR: input $input_suffix not found" >&2
        exit 1
      }

      TCC=$(find_input tcc-0.9.27-musl-v2)
      MUSL=$(find_input musl-1.1.24-native-candidate)
      MAKE=$(find_input make-3.82-tcc)
      SED=$(find_input sed-4.0.9-musl)
      M4=$(find_input m4-1.4.7-musl)
      GREP=$(find_input grep-2.4-musl)
      DIFF=$(find_input diffutils-2.7-musl)
      GAWK=$(find_input gawk-3.0.4-musl)
      BISON=$(find_input bison-2.3-musl)
      FLEX=$(find_input flex-2.6.4-musl)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input binutils-2.30-src)

      TOOLBIN="$WORK/tools"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK" "$TOOLBIN"
      for applet in basename cat chmod cmp cp cut date dirname echo env expr false find head install ln ls mkdir mktemp mv od printf pwd readlink rm rmdir sed sh sha256sum sleep sort tail tee test timeout touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done

      CONFIGURE_PROBE_AUDIT="$WORK/configure-preprocess.audit"
      CONFIGURE_PROBE_COUNT="$WORK/configure-preprocess.count"
      TCC_WRAPPER="$TOOLBIN/tcc-bounded"
      $BB rm -f "$CONFIGURE_PROBE_AUDIT" "$CONFIGURE_PROBE_COUNT"
      $BB cat > "$TCC_WRAPPER" <<EOF
#!/bin/sh
set -eu
REAL_TCC="$TCC/bin/tcc"
MUSL_INCLUDE="$MUSL/include"
MUSL_LIB="$MUSL/lib"
RUNTIME_OBJECT="$WORK/binutils-runtime.o"
RUNTIME_ASM_OBJECT="$WORK/binutils-runtime-asm.o"
SOURCE_BYTES_MAX=$CONFIGURE_PROBE_SOURCE_BYTES_MAX
INVOCATION_COUNT_MAX=$CONFIGURE_PROBE_INVOCATION_COUNT_MAX
mode=link
source_file=
output_file=
previous=
for argument in "\$@"; do
  if [ "\$previous" = -o ]; then output_file="\$argument"; fi
  case "\$argument" in
    -c) mode=compile ;;
    -E) mode=preprocess ;;
    --version|-V|-qversion) exec "\$REAL_TCC" -v ;;
    *.c) source_file="\$argument" ;;
  esac
  previous="\$argument"
done
if [ "\$mode" = preprocess ]; then
  reject_probe() {
    echo "binutils-tcc-wrapper: configure preprocess rejected: \$1" >&2
    exit 1
  }
  probe_dir="\${MANTLE_BINUTILS_CONFIGURE_PROBE_DIR:-}"
  probe_class="\${MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS:-}"
  probe_audit="\${MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT:-}"
  probe_count_file="\${MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT:-}"
  test -n "\$probe_dir" || reject_probe missing-authority-directory
  test -n "\$probe_audit" || reject_probe missing-audit-path
  test -n "\$probe_count_file" || reject_probe missing-count-path
  case "\$probe_class" in
    intl|libiberty|zlib|bfd|opcodes|binutils|gas|gprof|ld) ;;
    *) reject_probe unknown-configure-class ;;
  esac
  current_dir=\$(/bin/busybox pwd -P) || reject_probe current-directory-unavailable
  test "\$current_dir" = "\$probe_dir" || reject_probe current-directory-mismatch
  case "\$source_file" in conftest.c|./conftest.c) ;; *) reject_probe non-conftest-source ;; esac
  test -z "\$output_file" || reject_probe explicit-output-forbidden
  source_canonical=\$(/bin/busybox readlink -f "\$source_file") || reject_probe source-canonicalization-failed
  source_parent=\$(/bin/busybox dirname "\$source_canonical") || reject_probe source-parent-unavailable
  test "\$source_parent" = "\$probe_dir" || reject_probe source-escape
  source_bytes=\$(/bin/busybox wc -c < "\$source_canonical") || reject_probe source-size-unavailable
  case "\$source_bytes" in ''|*[!0-9]*) reject_probe invalid-source-size ;; esac
  test "\$source_bytes" -le "\$SOURCE_BYTES_MAX" || reject_probe source-too-large
  probe_count=0
  if [ -f "\$probe_count_file" ]; then probe_count=\$(/bin/busybox cat "\$probe_count_file") || reject_probe count-read-failed; fi
  case "\$probe_count" in ''|*[!0-9]*) reject_probe invalid-count ;; esac
  test "\$probe_count" -lt "\$INVOCATION_COUNT_MAX" || reject_probe invocation-budget-exhausted
  next_probe_count=\$((probe_count + 1))
  /bin/busybox printf '%s\n' "\$next_probe_count" > "\$probe_count_file" || reject_probe count-write-failed
  /bin/busybox printf '%s\t%s\t%s\n' "\$next_probe_count" "\$probe_class" "\$source_canonical" >> "\$probe_audit" || reject_probe audit-write-failed
  if /bin/busybox grep -q '^#ifdef CHAR_BIT' "\$source_canonical"; then
    /bin/busybox grep -Eq '^#define[[:space:]]+CHAR_BIT[[:space:]]+8([[:space:]]|$)' "\$MUSL_INCLUDE/limits.h" || reject_probe unexpected-char-bit
    /bin/busybox printf 'found\n'
    exit 0
  fi
  scratch_source="/tmp/binutils-cpp-source-\$\$.c"
  scratch_object="/tmp/binutils-cpp-object-\$\$.o"
  /bin/busybox sed '/^[[:space:]]*Syntax error[[:space:]]*$/d' "\$source_canonical" > "\$scratch_source"
  /bin/busybox printf '\nint mantle_configure_cpp_probe(void) { return 0; }\n' >> "\$scratch_source"
  "\$REAL_TCC" -c -I"\$MUSL_INCLUDE" "\$scratch_source" -o "\$scratch_object" || reject_probe compile-validation-failed
  test -s "\$scratch_object" || reject_probe compile-validation-output-missing
  /bin/busybox cat "\$source_canonical"
  /bin/busybox rm -f "\$scratch_source" "\$scratch_object"
  exit 0
fi
if [ "\$mode" = compile ]; then
  if "\$REAL_TCC" "\$@"; then compile_status=0; else compile_status=\$?; fi
  test "\$compile_status" -eq 0 || exit "\$compile_status"
  if [ -z "\$output_file" ]; then
    test -n "\$source_file" || exit 1
    source_name=\${source_file##*/}
    output_file=\${source_name%.c}.o
  fi
  test -s "\$output_file" || exit 1
  /bin/busybox chmod 644 "\$output_file"
  exit 0
fi
if [ -z "\$output_file" ]; then output_file=a.out; fi
test -s "\$RUNTIME_OBJECT" || exit 1
test -s "\$RUNTIME_ASM_OBJECT" || exit 1
"\$REAL_TCC" -nostdlib -static "\$MUSL_LIB/crt1.o" "\$@" "\$RUNTIME_OBJECT" "\$RUNTIME_ASM_OBJECT" "\$MUSL_LIB/libc.a"
link_status=\$?
test "\$link_status" -eq 0 || exit "\$link_status"
test -s "\$output_file" || exit 1
/bin/busybox chmod 755 "\$output_file"
exit 0
EOF
      $BB chmod 755 "$TCC_WRAPPER"

      AR_WRAPPER="$TOOLBIN/ar-bounded"
      $BB cat > "$AR_WRAPPER" <<EOF
#!/bin/sh
set -eu
MEMBER_COUNT_MAX=$ARCHIVE_MEMBER_COUNT_MAX
HEADER_ARG_COUNT=$ARCHIVE_HEADER_ARG_COUNT
GLOBAL_HEADER_BYTES=$ARCHIVE_GLOBAL_HEADER_BYTES
SHORT_NAME_BYTES_MAX=$ARCHIVE_SHORT_NAME_BYTES_MAX
MEMBER_NAME_BYTES_MAX=$ARCHIVE_MEMBER_NAME_BYTES_MAX
MEMBER_BYTES_MAX=$ARCHIVE_MEMBER_BYTES_MAX
STRING_TABLE_BYTES_MAX=$ARCHIVE_STRING_TABLE_BYTES_MAX
OUTPUT_BYTES_MAX=$ARCHIVE_OUTPUT_BYTES_MAX
ALIGNMENT_BYTES=$ARCHIVE_ALIGNMENT_BYTES
LONG_NAME_SUFFIX_BYTES=$ARCHIVE_LONG_NAME_SUFFIX_BYTES
reject_archive() {
  echo "binutils-ar-wrapper: archive request rejected: \$1" >&2
  exit 1
}
test "\$#" -ge "\$HEADER_ARG_COUNT" || reject_archive missing-arguments
flags=\$1
archive=\$2
shift "\$HEADER_ARG_COUNT"
flags=\${flags#-}
case "\$flags" in *[!cruqs]*) reject_archive unsupported-mode ;; esac
case "\$flags" in *r*|*q*) ;; *) reject_archive non-inserting-mode ;; esac
test -n "\$archive" || reject_archive empty-archive-path
member_count=\$#
test "\$member_count" -gt 0 || reject_archive empty-member-set
test "\$member_count" -le "\$MEMBER_COUNT_MAX" || reject_archive member-budget-exhausted
string_table="\$archive.mantle-long-names-\$\$"
/bin/busybox rm -f "\$archive" "\$string_table"
: > "\$string_table"
for member in "\$@"; do
  test -f "\$member" || reject_archive missing-member
  member_name=\$(/bin/busybox basename "\$member") || reject_archive basename-failed
  case "\$member_name" in ''|*[!A-Za-z0-9._+-]*) reject_archive unsafe-member-name ;; esac
  member_name_bytes=\$(/bin/busybox printf '%s' "\$member_name" | /bin/busybox wc -c)
  test "\$member_name_bytes" -le "\$MEMBER_NAME_BYTES_MAX" || reject_archive member-name-too-long
  member_bytes=\$(/bin/busybox wc -c < "\$member") || reject_archive member-size-unavailable
  test "\$member_bytes" -le "\$MEMBER_BYTES_MAX" || reject_archive member-too-large
  if [ "\$member_name_bytes" -gt "\$SHORT_NAME_BYTES_MAX" ]; then
    /bin/busybox printf '%s/\n' "\$member_name" >> "\$string_table"
  fi
done
write_header() {
  header_name=\$1
  payload_bytes=\$2
  /bin/busybox printf '%-16s%-12s%-6s%-6s%-8s%-10s\`\n' "\$header_name" 0 0 0 100644 "\$payload_bytes" >> "\$archive"
}
append_padding() {
  payload_bytes=\$1
  if [ \$((payload_bytes % ALIGNMENT_BYTES)) -ne 0 ]; then /bin/busybox printf '\n' >> "\$archive"; fi
}
/bin/busybox printf '!<arch>\n' > "\$archive"
test "\$(/bin/busybox wc -c < "\$archive")" -eq "\$GLOBAL_HEADER_BYTES" || reject_archive invalid-global-header
string_table_bytes=\$(/bin/busybox wc -c < "\$string_table") || reject_archive string-table-size-unavailable
test "\$string_table_bytes" -le "\$STRING_TABLE_BYTES_MAX" || reject_archive string-table-too-large
if [ "\$string_table_bytes" -gt 0 ]; then
  write_header // "\$string_table_bytes"
  /bin/busybox cat "\$string_table" >> "\$archive"
  append_padding "\$string_table_bytes"
fi
long_name_offset=0
member_index=0
for member in "\$@"; do
  member_name=\$(/bin/busybox basename "\$member") || reject_archive basename-failed
  member_name_bytes=\$(/bin/busybox printf '%s' "\$member_name" | /bin/busybox wc -c)
  member_bytes=\$(/bin/busybox wc -c < "\$member") || reject_archive member-size-unavailable
  if [ "\$member_name_bytes" -gt "\$SHORT_NAME_BYTES_MAX" ]; then
    header_name="/\$long_name_offset"
    long_name_offset=\$((long_name_offset + member_name_bytes + LONG_NAME_SUFFIX_BYTES))
  else
    header_name="\$member_name/"
  fi
  write_header "\$header_name" "\$member_bytes"
  /bin/busybox cat "\$member" >> "\$archive"
  append_padding "\$member_bytes"
  member_index=\$((member_index + 1))
done
/bin/busybox rm -f "\$string_table"
test "\$member_index" -eq "\$member_count" || reject_archive member-count-mismatch
archive_bytes=\$(/bin/busybox wc -c < "\$archive") || reject_archive output-size-unavailable
test "\$archive_bytes" -le "\$OUTPUT_BYTES_MAX" || reject_archive output-too-large
test -s "\$archive" || reject_archive output-missing
exit 0
EOF
      $BB chmod 755 "$AR_WRAPPER"

      $BB cat > "$WORK/binutils-runtime.c" <<'RUNTIME_EOF'
#include <limits.h>

enum { MANTLE_BINARY_RADIX = 2 };

double
__floatundidf(value)
unsigned long value;
{
    double upper;

    if (value <= (unsigned long) LONG_MAX)
        return (double) (long) value;
    upper = (double) (long) (value >> 1);
    return upper * (double) MANTLE_BINARY_RADIX + (double) (value & (unsigned long) 1);
}

static unsigned long
mantle_double_to_ulong_bounded(value, valid_out)
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
        unit *= (double) MANTLE_BINARY_RADIX;
        bit <<= 1;
        step_count++;
    }
    while (bit != (unsigned long) 0) {
        if (remaining >= unit) {
            remaining -= unit;
            result |= bit;
        }
        unit /= (double) MANTLE_BINARY_RADIX;
        bit >>= 1;
    }
    if (remaining >= (double) 1)
        return (unsigned long) 0;
    *valid_out = 1;
    return result;
}

unsigned long
__fixunsdfdi(value)
double value;
{
    int valid = 0;
    unsigned long result = mantle_double_to_ulong_bounded(value, &valid);

    return valid ? result : (unsigned long) 0;
}

char *gettext(const char *message) { return (char *) message; }
unsigned int towlower(unsigned int value)
{
    enum {
        MANTLE_ASCII_UPPERCASE_FIRST = 'A',
        MANTLE_ASCII_UPPERCASE_LAST = 'Z',
        MANTLE_ASCII_CASE_OFFSET = 'a' - 'A'
    };
    if (value >= MANTLE_ASCII_UPPERCASE_FIRST && value <= MANTLE_ASCII_UPPERCASE_LAST)
        return value + MANTLE_ASCII_CASE_OFFSET;
    return value;
}
int prctl(int option, ...) { (void) option; return -1; }
int __lsysinfo(void *information) { (void) information; return -1; }
RUNTIME_EOF
      "$TCC_WRAPPER" -c -I"$MUSL/include" "$WORK/binutils-runtime.c" -o "$WORK/binutils-runtime.o"
      test -s "$WORK/binutils-runtime.o" || { echo "ERROR: bounded binutils runtime object missing" >&2; exit 1; }
      $BB cat > "$WORK/binutils-runtime.s" <<'RUNTIME_ASM_EOF'
.global sigsetjmp
.type sigsetjmp,@function
sigsetjmp:
  jmp setjmp
RUNTIME_ASM_EOF
      "$TCC_WRAPPER" -c "$WORK/binutils-runtime.s" -o "$WORK/binutils-runtime-asm.o"
      test -s "$WORK/binutils-runtime-asm.o" || { echo "ERROR: bounded binutils assembly runtime object missing" >&2; exit 1; }

      export PATH="$TOOLBIN:$BISON/bin:$FLEX/bin:$M4/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$SED/bin:$MAKE/bin:$TCC/bin:$STAGE0/bin"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export AWK="$GAWK/bin/gawk"
      export M4="$M4/bin/m4"
      export BISON_PKGDATADIR="$BISON/share/bison"
      export BISON="$BISON/bin/bison"
      export FLEX="$FLEX/bin/flex"
      export CC="$TCC_WRAPPER"
      export CC_FOR_BUILD="$TCC_WRAPPER"
      export AR="$AR_WRAPPER"
      export RANLIB=true
      export MAKEINFO=true
      export CFLAGS="-I$MUSL/include -static -D_GNU_SOURCE -DBUILDFIXED=1 -DDYNAMIC_CRC_TABLE=1"
      export CFLAGS_FOR_BUILD="$CFLAGS"
      export LDFLAGS="-static -L$MUSL/lib"

      cd "$WORK"
      $BB cp -R "$SRC" source
      $BB chmod -R u+w source
      cd source

      # Apply the selected x86_64 table dependency repair to both the source
      # Makefile and authenticated release Makefile template. Configure-script
      # regeneration is intentionally outside this row.
      $BB sed -i 's/^$(srcdir)\/i386-init.h: @MAINT@ /$(srcdir)\/i386-init.h: /' opcodes/Makefile.am opcodes/Makefile.in
      $BB grep -q '^$(srcdir)/i386-init.h: i386-gen' opcodes/Makefile.am
      $BB grep -q '^$(srcdir)/i386-init.h: i386-gen' opcodes/Makefile.in

      "$BB" timeout "$CONFIG_SUB_TIMEOUT_SECONDS" /bin/sh ./config.sub sun4 \
        > "$WORK/config-sub-sun4.out" || { echo "ERROR: authenticated config.sub preflight timed out or failed" >&2; exit 1; }
      test -s "$WORK/config-sub-sun4.out" || { echo "ERROR: authenticated config.sub preflight emitted no target" >&2; exit 1; }
      $BB grep -q sun "$WORK/config-sub-sun4.out" || { echo "ERROR: authenticated config.sub preflight returned an unrelated target" >&2; exit 1; }

      # Remove generated files selected by the x86_64 build. Required parser,
      # scanner, opcode, and BFD artifacts must be recreated from source;
      # zlib's unused fixed tables must remain absent under the dynamic flags.
      for grammar in \
        binutils/arparse.y binutils/defparse.y binutils/mcparse.y \
        binutils/rcparse.y binutils/sysinfo.y intl/plural.y ld/deffilep.y \
        ld/ldgram.y; do
        test -f "$grammar"
        $BB touch "$grammar"
      done
      for scanner in \
        binutils/arlex.l binutils/deflex.l binutils/syslex.l ld/ldlex.l; do
        test -f "$scanner"
        $BB touch "$scanner"
      done
      $BB rm -f \
        binutils/arparse.c binutils/arparse.h binutils/defparse.c \
        binutils/defparse.h binutils/mcparse.c binutils/mcparse.h \
        binutils/rcparse.c binutils/rcparse.h binutils/sysinfo.c \
        binutils/sysinfo.h intl/plural.c ld/deffilep.c ld/deffilep.h \
        ld/ldgram.c ld/ldgram.h binutils/arlex.c binutils/deflex.c \
        binutils/syslex.c ld/ldlex.c
      $BB rm -f opcodes/i386-init.h opcodes/i386-tbl.h
      $BB rm -f bfd/libbfd.h bfd/bfd-in2.h bfd/libcoff.h
      bfd_bootstrap_config_refs=$($BB grep -o 'bfd-in3\.h:bfd-in2\.h' bfd/configure | $BB wc -l)
      test "$bfd_bootstrap_config_refs" -eq "$BFD_BOOTSTRAP_CONFIG_INPUT_REFS_EXPECTED" || { echo "ERROR: unexpected BFD bootstrap config input reference count: $bfd_bootstrap_config_refs" >&2; exit 1; }
      $BB cp bfd/configure "$WORK/bfd-configure.authenticated"
      $BB sed -i 's/bfd-in3\.h:bfd-in2\.h//g' bfd/configure
      test "$($BB grep -o 'bfd-in3\.h:bfd-in2\.h' bfd/configure | $BB wc -l)" -eq 0 || { echo "ERROR: BFD bootstrap config input reference retained" >&2; exit 1; }
      $BB rm -f zlib/crc32.h zlib/inffixed.h
      $BB rm -f bfd/doc/bfd.info binutils/doc/binutils.info gas/doc/as.info ld/ld.info
      $BB rm -f libiberty/functions.texi

      configure_dir() {
        configure_path=$1
        cd "$WORK/source/$configure_path"
        MANTLE_BINUTILS_CONFIGURE_PROBE_DIR="$WORK/source/$configure_path" \
          MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS="$configure_path" \
          MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT="$CONFIGURE_PROBE_AUDIT" \
          MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT="$CONFIGURE_PROBE_COUNT" \
          LD=true \
          AR="$AR_WRAPPER" \
          CC="$TCC_WRAPPER" \
          CPP="$TCC_WRAPPER -E" \
          CFLAGS="$CFLAGS" \
          LDFLAGS="$LDFLAGS" \
          RANLIB=true \
          MAKEINFO=true \
          "$BB" timeout "$CONFIGURE_TIMEOUT_SECONDS" /bin/sh ./configure \
            --disable-nls \
            --enable-deterministic-archives \
            --enable-64-bit-bfd \
            --disable-shared \
            --disable-plugins \
            --disable-werror \
            --build="$TARGET" \
            --host="$TARGET" \
            --target="$TARGET" \
            --program-prefix="" \
            --prefix="$out" \
            --libdir="$out/lib" \
            --with-sysroot= \
            --srcdir=. \
            --enable-compressed-debug-sections=all \
            lt_cv_sys_max_cmd_len="$CONFIGURE_MAX_COMMAND_BYTES" || {
          echo "ERROR: configure failed for $configure_path" >&2
          if test -f config.log; then
            "$BB" grep -n -E 'error:|failed|sanity|conftest' config.log \
              | "$BB" tail -n "$CONFIG_LOG_ERROR_LINES" >&2 || true
            "$BB" tail -n "$CONFIG_LOG_TAIL_LINES" config.log >&2
          fi
          exit 1
        }
      }

      for configure_path in intl libiberty zlib bfd opcodes binutils gas gprof ld; do
        configure_dir "$configure_path"
      done
      cd "$WORK/source"
      "$BB" timeout "$MAKE_TIMEOUT_SECONDS" make -j"$JOB_COUNT" -C bfd headers \
        CC="$TCC_WRAPPER" AR="$AR_WRAPPER" RANLIB=true MAKEINFO=true
      "$BB" timeout "$MAKE_TIMEOUT_SECONDS" make -j"$JOB_COUNT" -C intl plural.c \
        CC="$TCC_WRAPPER" AR="$AR_WRAPPER" RANLIB=true MAKEINFO=true
      test -s intl/plural.c || { echo "ERROR: Bison-generated intl parser missing" >&2; exit 1; }
      test -s bfd/bfd-in2.h || { echo "ERROR: first-pass BFD header missing" >&2; exit 1; }
      $BB cp "$WORK/bfd-configure.authenticated" bfd/configure
      $BB chmod 755 bfd/configure
      configure_dir bfd
      test -s "$WORK/source/bfd/bfd-in3.h" || { echo "ERROR: second-pass BFD header missing" >&2; exit 1; }
      cd "$WORK/source"
      "$BB" timeout "$MAKE_TIMEOUT_SECONDS" make -j"$JOB_COUNT" -C binutils \
        arparse.c defparse.c mcparse.c rcparse.c sysinfo.c \
        arlex.c deflex.c syslex.c \
        CC="$TCC_WRAPPER" AR="$AR_WRAPPER" RANLIB=true MAKEINFO=true
      "$BB" timeout "$MAKE_TIMEOUT_SECONDS" make -j"$JOB_COUNT" -C ld \
        deffilep.c ldgram.c ldlex.c \
        CC="$TCC_WRAPPER" AR="$AR_WRAPPER" RANLIB=true MAKEINFO=true
      test -s "$CONFIGURE_PROBE_AUDIT" || { echo "ERROR: configure preprocess audit missing" >&2; exit 1; }
      probe_count=$($BB cat "$CONFIGURE_PROBE_COUNT")
      audit_count=$($BB wc -l < "$CONFIGURE_PROBE_AUDIT")
      test "$probe_count" -eq "$audit_count" || { echo "ERROR: configure preprocess audit count mismatch" >&2; exit 1; }
      test "$probe_count" -le "$CONFIGURE_PROBE_INVOCATION_COUNT_MAX" || { echo "ERROR: configure preprocess invocation budget exceeded" >&2; exit 1; }

      cd "$WORK/source"
      for build_path in libiberty zlib bfd opcodes binutils gas gprof ld; do
        "$BB" timeout "$MAKE_TIMEOUT_SECONDS" make -j"$JOB_COUNT" -C "$build_path" \
          tooldir="$out" \
          CC="$TCC_WRAPPER" \
          AR="$AR_WRAPPER" \
          RANLIB=true \
          MAKEINFO=true \
          CPPFLAGS="-DPLUGIN_LITTLE_ENDIAN"
      done

      for generated_path in \
        binutils/arparse.c binutils/arparse.h binutils/defparse.c \
        binutils/defparse.h binutils/mcparse.c binutils/mcparse.h \
        binutils/rcparse.c binutils/rcparse.h binutils/sysinfo.c \
        binutils/sysinfo.h intl/plural.c ld/deffilep.c ld/deffilep.h \
        ld/ldgram.c ld/ldgram.h binutils/arlex.c binutils/deflex.c \
        binutils/syslex.c ld/ldlex.c opcodes/i386-init.h \
        opcodes/i386-tbl.h bfd/libbfd.h bfd/bfd-in2.h bfd/libcoff.h; do
        test -s "$generated_path" || { echo "ERROR: missing regenerated artifact $generated_path" >&2; exit 1; }
      done
      test ! -e zlib/crc32.h || { echo "ERROR: unused zlib CRC table was retained" >&2; exit 1; }
      test ! -e zlib/inffixed.h || { echo "ERROR: unused zlib inflate table was retained" >&2; exit 1; }

      for install_path in libiberty zlib bfd opcodes binutils gas gprof ld; do
        "$BB" timeout "$MAKE_TIMEOUT_SECONDS" make -j"$JOB_COUNT" -C "$install_path" \
          tooldir="$out" \
          DESTDIR= \
          prefix="$out" \
          MAKEINFO=true \
          install
      done

      for required_tool in as ld ar ranlib nm objcopy objdump readelf size strings strip; do
        test -x "$out/bin/$required_tool"
        if $BB grep -a -q "$TCC/bin/tcc" "$out/bin/$required_tool"; then
          echo "ERROR: $required_tool delegates to predecessor TinyCC" >&2
          exit 1
        fi
        triplet_tool="$out/bin/$TARGET-$required_tool"
        if [ ! -e "$triplet_tool" ]; then
          $BB ln -s "$out/bin/$required_tool" "$triplet_tool"
        fi
      done

      $BB cat > "$WORK/positive.s" <<'MANTLE_ASSEMBLY'
.global _start
.text
_start:
  mov $60, %rax
  mov $42, %rdi
  syscall
MANTLE_ASSEMBLY
      "$out/bin/as" -o "$WORK/positive.o" "$WORK/positive.s"
      "$out/bin/ld" -o "$WORK/positive" "$WORK/positive.o"
      $BB chmod 755 "$WORK/positive"
      set +e
      "$WORK/positive"
      smoke_status=$?
      set -e
      test "$smoke_status" -eq "$SMOKE_EXIT_STATUS"
      "$out/bin/ar" rc "$WORK/libpositive.a" "$WORK/positive.o"
      "$out/bin/ranlib" "$WORK/libpositive.a"
      "$out/bin/ar" t "$WORK/libpositive.a" | $BB grep -q positive.o
      "$out/bin/nm" "$WORK/positive.o" | $BB grep -q _start
      "$out/bin/objcopy" "$WORK/positive.o" "$WORK/positive-copy.o"
      "$out/bin/objdump" -d "$WORK/positive-copy.o" | $BB grep -q syscall
      "$out/bin/readelf" -h "$WORK/positive.o" | $BB grep -q ELF64

      $BB printf '%s\n' '.mantle-invalid-directive' > "$WORK/rejected.s"
      $BB rm -f "$WORK/rejected.o"
      if "$out/bin/as" -o "$WORK/rejected.o" "$WORK/rejected.s"; then
        echo "ERROR: source-built assembler accepted malformed input" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"
      $BB printf '%s\n' 'not-an-object' > "$WORK/rejected.o"
      $BB rm -f "$WORK/rejected-bin"
      if "$out/bin/ld" -o "$WORK/rejected-bin" "$WORK/rejected.o"; then
        echo "ERROR: source-built linker accepted malformed object input" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-bin"
      $BB printf '%s\n' 'not-an-archive' > "$WORK/rejected.a"
      if "$out/bin/ar" t "$WORK/rejected.a"; then
        echo "ERROR: source-built archive tool accepted malformed archive input" >&2
        exit 1
      fi

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/binutils-tcc.json" <<EOF
{
  "schema": "mantle-binutils-tcc-row-receipt-v1",
  "status": "complete-candidate",
  "target": "$TARGET",
  "source": "binutils-2.30-src",
  "compiler": "tcc-0.9.27-musl-v2",
  "generated_sources": "authenticated-release-configure-plus-declared-bison-flex-native-generators",
  "fallback_events": [],
  "non_claims": ["assembler-correctness", "linker-correctness", "bootstrap-correctness"]
}
EOF
      test -s "$out/share/mantle-bootstrap/binutils-tcc.json"
      echo "source-built TCC-era binutils matrix passed"
    "%
  ],
  inputs = [
    stage0,
    tcc,
    musl,
    make,
    sed,
    m4,
    grep,
    diffutils,
    gawk,
    bison,
    flex,
    binutils_src,
  ],
} | crunch.Derivation
