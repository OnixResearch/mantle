set -eu
REAL_TCC='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/tcc-musl-v2-stage/runtime/output/bin/tcc-0.9.27-musl-v2'
MUSL_INCLUDE='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/musl-native-stage/runtime/output/include'
MUSL_LIB='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/musl-native-stage/runtime/output/lib'
RUNTIME_OBJECT='/home/brittonr/.cargo-target/stagex-binutils-configure-blocker-v2/binutils-runtime.o'
RUNTIME_ASM_OBJECT='/home/brittonr/.cargo-target/stagex-binutils-configure-blocker-v2/binutils-runtime-asm.o'
CHMOD='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/coreutils-stage/runtime/output/bin/chmod'
CAT='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/coreutils-stage/runtime/output/bin/cat'
RM='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/coreutils-stage/runtime/output/bin/rm'
WC='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/coreutils-stage/runtime/output/bin/wc'
GREP='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/grep-stage/runtime/output/bin/grep'
SED='/home/brittonr/.cargo-target/stagex-protected-transition-v77-bash-full-final-20260729/sed-stage/runtime/output/bin/sed'
SOURCE_BYTES_MAX=65536
INVOCATION_COUNT_MAX=4096
mode=link
source_file=
output_file=
previous=
for argument in "$@"; do
  if test "$previous" = -o; then output_file="$argument"; fi
  case "$argument" in
    -c) mode=compile ;;
    -E) mode=preprocess ;;
    --version|-V|-qversion) exec "$REAL_TCC" -v ;;
    *.c) source_file="$argument" ;;
  esac
  previous="$argument"
done
if test "$mode" = preprocess; then
  reject_probe() {
    printf '%s\n' "binutils-tcc-wrapper: configure preprocess rejected: $1" >&2
    exit 1
  }
  probe_dir=${MANTLE_BINUTILS_CONFIGURE_PROBE_DIR:-}
  probe_class=${MANTLE_BINUTILS_CONFIGURE_PROBE_CLASS:-}
  probe_audit=${MANTLE_BINUTILS_CONFIGURE_PROBE_AUDIT:-}
  probe_count_file=${MANTLE_BINUTILS_CONFIGURE_PROBE_COUNT:-}
  test -n "$probe_dir" || reject_probe missing-authority-directory
  test -n "$probe_audit" || reject_probe missing-audit-path
  test -n "$probe_count_file" || reject_probe missing-count-path
  case "$probe_class" in
    intl|libiberty|zlib|bfd|opcodes|binutils|gas|gprof|ld) ;;
    *) reject_probe unknown-configure-class ;;
  esac
  current_dir=$PWD
  test "$current_dir" = "$probe_dir" || reject_probe current-directory-mismatch
  case "$source_file" in conftest.c|./conftest.c) ;; *) reject_probe non-conftest-source ;; esac
  test -z "$output_file" || reject_probe explicit-output-forbidden
  source_canonical="$probe_dir/conftest.c"
  test -f "$source_canonical" || reject_probe source-unavailable
  source_bytes=$("$WC" -c < "$source_canonical") || reject_probe source-size-unavailable
  source_bytes=${source_bytes//[[:space:]]/}
  case "$source_bytes" in ''|*[!0-9]*) reject_probe invalid-source-size ;; esac
  test "$source_bytes" -le "$SOURCE_BYTES_MAX" || reject_probe source-too-large
  probe_count=0
  if test -f "$probe_count_file"; then probe_count=$("$CAT" "$probe_count_file") || reject_probe count-read-failed; fi
  case "$probe_count" in ''|*[!0-9]*) reject_probe invalid-count ;; esac
  test "$probe_count" -lt "$INVOCATION_COUNT_MAX" || reject_probe invocation-budget-exhausted
  next_probe_count=$((probe_count + 1))
  printf '%s\n' "$next_probe_count" > "$probe_count_file" || reject_probe count-write-failed
  printf '%s\t%s\t%s\n' "$next_probe_count" "$probe_class" "$source_canonical" >> "$probe_audit" || reject_probe audit-write-failed
  if "$GREP" -q '^#ifdef CHAR_BIT' "$source_canonical"; then
    "$GREP" -Eq '^#define[[:space:]]+CHAR_BIT[[:space:]]+8([[:space:]]|$)' "$MUSL_INCLUDE/limits.h" || reject_probe unexpected-char-bit
    printf 'found\n'
    exit 0
  fi
  scratch_source="$probe_dir/.mantle-cpp-source.c"
  scratch_object="$probe_dir/.mantle-cpp-object.o"
  "$RM" -f "$scratch_source" "$scratch_object"
  "$SED" '/^[[:space:]]*Syntax error[[:space:]]*$/d' "$source_canonical" > "$scratch_source"
  printf '\nint mantle_configure_cpp_probe(void) { return 0; }\n' >> "$scratch_source"
  "$REAL_TCC" -c -I"$MUSL_INCLUDE" "$scratch_source" -o "$scratch_object" || reject_probe compile-validation-failed
  test -s "$scratch_object" || reject_probe compile-validation-output-missing
  "$CAT" "$source_canonical"
  "$RM" -f "$scratch_source" "$scratch_object"
  exit 0
fi
if test "$mode" = compile; then
  "$REAL_TCC" "$@"
  compile_status=$?
  test "$compile_status" -eq 0 || exit "$compile_status"
  if test -z "$output_file"; then
    test -n "$source_file" || exit 1
    source_name=${source_file##*/}
    output_file=${source_name%.c}.o
  fi
  test -s "$output_file" || exit 1
  "$CHMOD" 644 "$output_file"
  exit 0
fi
if test -z "$output_file"; then output_file=a.out; fi
test -s "$RUNTIME_OBJECT"
test -s "$RUNTIME_ASM_OBJECT"
"$REAL_TCC" -nostdlib -static "$MUSL_LIB/crt1.o" "$@" "$RUNTIME_OBJECT" "$RUNTIME_ASM_OBJECT" "$MUSL_LIB/libc.a"
link_status=$?
test "$link_status" -eq 0 || exit "$link_status"
test -s "$output_file"
"$CHMOD" 755 "$output_file"
