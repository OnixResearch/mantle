# Diagnostic clean binutils 2.30 rebuild using pass4 GCC and musl.
#
# This stage replaces the TinyCC-era reduced BFD/libiberty compatibility build
# with the upstream configure/make/install flow. It remains isolated from
# bootstrap/seed.ncl until the complete runtime suite and provider admission
# checks pass.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let gcc = crunch.derivationFile "gcc-4.0-musl-pass4.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-gcc-pass4.ncl" in
let stagex_provider = import "stagex-provider-proof-input.ncl" in
let stagex_transition = import "stagex-transition-proof-input.ncl" in
let binutils_src = crunch.fetchTarball {
  url = "https://ftpmirror.gnu.org/binutils/binutils-2.30.tar.xz",
  hash = "sha256-L8aaWezlL47cNdPIbSEAtryYUDXLe9htMSECT4xqpoc=",
  name = "binutils-2.30-src",
} in
let automake115_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/automake/automake-1.15.1.tar.xz",
  hash = "sha256-/KHwBUWoEgUxjLuLFOpqM4c0UXUDWiyiBV4HBTwOcZU=",
  name = "automake-1.15.1-config-sub-src",
} in

{
  name = "binutils-2.30-gcc-pass4-v16",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=4.0.4
      JOB_COUNT=1
      BUILD_CFLAGS="-O2 -fno-common"

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then echo "$candidate"; return; fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      GCC=$(find_input gcc-4.0.4-musl-pass4-v7)
      MUSL=$(find_input musl-1.1.24-gcc-pass4-v2)
      STAGEX=$(find_input mantle-stagex-intermediate-provider)
      TRANSITION=$(find_input mantle-stagex-transition)
      GREP="$TRANSITION/grep-stage/runtime/output"
      DIFFUTILS="$TRANSITION/diffutils-stage/runtime/output"
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SRC=$(find_input binutils-2.30-src)
      AUTOMAKE115_SRC=$(find_input automake-1.15.1-config-sub-src)

      WORK=/tmp/binutils-2.30-gcc-pass4
      TOOLBIN="$WORK/tools"
      PREDECESSOR_BINUTILS="$WORK/predecessor-binutils"
      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN" "$PREDECESSOR_BINUTILS/bin"
      for tool in ar as ld nm objcopy objdump ranlib readelf size strings strip; do
        $BB ln -s "$STAGEX/bin/x86_64-linux-musl-$tool" "$PREDECESSOR_BINUTILS/bin/$tool"
      done

      test -x "$GCC/bin/gcc"
      test -x "$GCC/bin/ld"
      test -x "$GCC/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -s "$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
      test -s "$MUSL/lib/libc.a"
      test -x "$PREDECESSOR_BINUTILS/bin/ar"
      test -x "$GREP/bin/grep"
      test -x "$DIFFUTILS/bin/diff"
      for applet in awk basename cat chmod cmp cp cut dirname echo expr find head ln ls mkdir mv od printf pwd readlink rm rmdir sed sh sleep sort tail tar test touch tr true uname uniq wc; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      $BB cat > "$TOOLBIN/file" <<EOF
#!/bin/sh
set -eu
BB=/bin/busybox
MAGIC_BYTES=4
ELF_MAGIC=7f454c46
ARCHIVE_MAGIC=213c6172
input_path=
for argument in "\$@"; do
  case "\$argument" in
    -*) ;;
    *) input_path="\$argument" ;;
  esac
done
if [ -z "\$input_path" ] || [ ! -f "\$input_path" ]; then
  exit 1
fi
magic="\$(\$BB od -An -tx1 -N\$MAGIC_BYTES "\$input_path" | \$BB tr -d ' ')"
case "\$magic" in
  \$ELF_MAGIC) echo "\$input_path: ELF 64-bit LSB" ;;
  \$ARCHIVE_MAGIC) echo "\$input_path: current ar archive" ;;
  *) echo "\$input_path: data" ;;
esac
EOF
      $BB chmod 755 "$TOOLBIN/file"
      $BB cat > "$TOOLBIN/gcc" <<EOF
#!/bin/sh
exec "$GCC/bin/gcc" -static "\$@"
EOF
      $BB chmod 755 "$TOOLBIN/gcc"
      BUILD_CC="$TOOLBIN/gcc"

      export PATH="$GREP/bin:$DIFFUTILS/bin:$TOOLBIN:$PREDECESSOR_BINUTILS/bin:$MAKE/bin:$STAGE0/bin:$PATH"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export MAKEINFO=true
      export CC="$BUILD_CC"
      export CXX="$BUILD_CC"
      export CPP="$BUILD_CC -E"
      export CXXCPP="$BUILD_CC -E -x c"
      export MAGIC_CMD="$TOOLBIN/file"
      export CFLAGS="$BUILD_CFLAGS"
      export CXXFLAGS="$BUILD_CFLAGS"
      export LDFLAGS="-static -Wl,-t"
      export AR="$PREDECESSOR_BINUTILS/bin/ar"
      export RANLIB="$PREDECESSOR_BINUTILS/bin/ar s"
      unset MAKEFLAGS MFLAGS MAKELEVEL

      $BB cp -r "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"
      test -f configure || { echo "ERROR: binutils configure missing" >&2; exit 1; }

      for config_sub in $($BB find . -name config.sub -type f); do
        $BB cp "$AUTOMAKE115_SRC/lib/config.sub" "$config_sub"
        $BB chmod 755 "$config_sub"
      done
      for configure_script in $($BB find . -name configure -type f); do
        $BB sed -i \
          -e 's/-Wstack-usage=262144//g' \
          -e "s#/usr/bin/file#$TOOLBIN/file#g" \
          "$configure_script"
        if $BB grep -q -- '-Wstack-usage=262144' "$configure_script"; then
          echo "ERROR: unsupported GCC warning flag remains in $configure_script" >&2
          exit 1
        fi
      done

      for generated_file in \
        binutils/arlex.c binutils/arparse.c binutils/arparse.h \
        binutils/deflex.c binutils/defparse.c binutils/defparse.h \
        gas/config/bfin-lex.c gas/config/bfin-parse.c gas/config/bfin-parse.h \
        gas/config/m68k-parse.c gas/config/m68k-parse.h \
        ld/deffilep.c ld/ldgram.c ld/ldgram.h ld/ldlex.c; do
        if [ -f "$generated_file" ]; then $BB touch "$generated_file"; fi
      done

      $BB mkdir -p "$WORK/build"
      cd "$WORK/build"
      CC="$BUILD_CC" \
      CXX="$BUILD_CC" \
      CXXCPP="$BUILD_CC -E -x c" \
      MAGIC_CMD="$TOOLBIN/file" \
      AR="$PREDECESSOR_BINUTILS/bin/ar" \
      RANLIB="$PREDECESSOR_BINUTILS/bin/ar s" \
      "$WORK/src/configure" \
        --prefix="$out" \
        --build="$TARGET" \
        --host="$TARGET" \
        --target="$TARGET" \
        --disable-gdb \
        --disable-gdbserver \
        --disable-sim \
        --disable-nls \
        --disable-shared \
        --disable-plugins \
        --disable-multilib \
        --disable-werror \
        --without-zlib || {
          configure_status=$?
          if [ -f config.log ]; then $BB cat config.log >&2; fi
          exit "$configure_status"
        }

      for generated_makefile in $($BB find . -name Makefile -type f); do
        $BB sed -i 's/-Wstack-usage=[0-9][0-9]*//g' "$generated_makefile"
        if $BB grep -q -- '-Wstack-usage=' "$generated_makefile"; then
          echo "ERROR: unsupported GCC warning flag remains in $generated_makefile" >&2
          exit 1
        fi
      done

      set +e
      make -j"$JOB_COUNT" all \
        CC="$BUILD_CC" \
        CXX="$BUILD_CC" \
        CXXCPP="$BUILD_CC -E -x c" \
        MAGIC_CMD="$TOOLBIN/file" \
        AR="$PREDECESSOR_BINUTILS/bin/ar" \
        RANLIB="$PREDECESSOR_BINUTILS/bin/ar s" \
        CFLAGS="$BUILD_CFLAGS" \
        CXXFLAGS="$BUILD_CFLAGS" \
        LDFLAGS="-static -Wl,-t" \
        WARN_CFLAGS= \
        MAKEINFO=true
      build_status=$?
      set -e
      if [ "$build_status" -ne 0 ]; then
        for diagnostic_archive in \
          "$WORK/build/opcodes/.libs/libopcodes.a" \
          "$WORK/build/bfd/.libs/libbfd.a" \
          "$WORK/build/zlib/libz.a" \
          "$WORK/build/libiberty/libiberty.a" \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a" \
          "$MUSL/lib/libc.a"; do
          diagnostic_name=${diagnostic_archive##*/}
          diagnostic_parent=${diagnostic_archive%/*}
          diagnostic_parent=${diagnostic_parent##*/}
          diagnostic_output="$WORK/diagnostic-$diagnostic_parent-$diagnostic_name.o"
          $BB rm -f "$diagnostic_output"
          set +e
          "$PREDECESSOR_BINUTILS/bin/ld" -r --whole-archive "$diagnostic_archive" \
            --no-whole-archive -o "$diagnostic_output"
          diagnostic_status=$?
          set -e
          echo "clean-binutils-archive-probe: $diagnostic_parent/$diagnostic_name status=$diagnostic_status" >&2
        done
        for diagnostic_object in \
          "$WORK/build/gas"/*.o \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION"/crtbeginT.o \
          "$GCC/lib/gcc/$TARGET/$GCC_VERSION"/crtend.o \
          "$MUSL/lib"/crt1.o "$MUSL/lib"/crti.o "$MUSL/lib"/crtn.o; do
          diagnostic_name=${diagnostic_object##*/}
          diagnostic_output="$WORK/diagnostic-object-$diagnostic_name.o"
          $BB rm -f "$diagnostic_output"
          set +e
          "$PREDECESSOR_BINUTILS/bin/ld" -r "$diagnostic_object" -o "$diagnostic_output"
          diagnostic_status=$?
          set -e
          if [ "$diagnostic_status" -ne 0 ]; then
            echo "clean-binutils-object-probe: $diagnostic_name status=$diagnostic_status" >&2
          fi
        done
        cumulative_inputs="$WORK/build/gas/*.o"
        for cumulative_archive in \
          none \
          "$WORK/build/opcodes/.libs/libopcodes.a" \
          "$WORK/build/bfd/.libs/libbfd.a" \
          "$WORK/build/zlib/libz.a" \
          "$WORK/build/libiberty/libiberty.a"; do
          if [ "$cumulative_archive" != none ]; then
            cumulative_inputs="$cumulative_inputs $cumulative_archive"
          fi
          cumulative_output="$WORK/diagnostic-cumulative.o"
          $BB rm -f "$cumulative_output"
          set +e
          "$PREDECESSOR_BINUTILS/bin/ld" -r $cumulative_inputs -o "$cumulative_output"
          cumulative_status=$?
          set -e
          echo "clean-binutils-cumulative-probe: ${cumulative_archive##*/} status=$cumulative_status" >&2
        done
        exit "$build_status"
      fi
      make -j"$JOB_COUNT" install \
        WARN_CFLAGS= \
        MAKEINFO=true

      for required_tool in as ld ar ranlib readelf nm objcopy objdump size strings strip; do
        test -x "$out/bin/$required_tool" || {
          echo "ERROR: clean binutils output missing $required_tool" >&2
          exit 1
        }
      done

      for required_tool in as ld ar ranlib readelf nm objcopy objdump size strings strip; do
        "$out/bin/$required_tool" --version > "$WORK/$required_tool-version.txt"
        $BB grep -q 'GNU' "$WORK/$required_tool-version.txt"
      done
      $BB grep -q 'GNU assembler' "$WORK/as-version.txt"
      $BB grep -q 'GNU ld' "$WORK/ld-version.txt"

      $BB cat > "$WORK/smoke.s" <<'ASM'
.section .text
.global _start
_start:
  xorpd %xmm0, %xmm0
  movsd %xmm0, %xmm1
  mov $60, %rax
  xor %rdi, %rdi
  syscall
ASM
      "$out/bin/as" -Qy -o "$WORK/smoke.o" "$WORK/smoke.s"
      test -s "$WORK/smoke.o"
      "$out/bin/readelf" -h "$WORK/smoke.o" > "$WORK/smoke-header.txt"
      $BB grep -q 'ELF' "$WORK/smoke-header.txt"
      $BB grep -q 'REL' "$WORK/smoke-header.txt"

      $BB rm -f "$WORK/libsmoke.a"
      "$out/bin/ar" rcs "$WORK/libsmoke.a" "$WORK/smoke.o"
      test -s "$WORK/libsmoke.a"
      "$out/bin/ranlib" "$WORK/libsmoke.a"
      "$out/bin/ar" t "$WORK/libsmoke.a" > "$WORK/archive-members.txt"
      $BB grep -q '^smoke.o$' "$WORK/archive-members.txt"
      "$out/bin/ld" -static -o "$WORK/archive-smoke" "$WORK/libsmoke.a"
      test -s "$WORK/archive-smoke"
      $BB chmod 755 "$WORK/archive-smoke"
      "$WORK/archive-smoke"

      "$out/bin/nm" "$WORK/smoke.o" > "$WORK/smoke.nm"
      $BB grep -q '_start' "$WORK/smoke.nm"
      "$out/bin/objdump" -h "$WORK/smoke.o" > "$WORK/smoke-objdump.txt"
      $BB grep -q '.text' "$WORK/smoke-objdump.txt"
      "$out/bin/objcopy" "$WORK/smoke.o" "$WORK/smoke-copy.o"
      test -s "$WORK/smoke-copy.o"
      "$out/bin/readelf" -h "$WORK/smoke-copy.o" > "$WORK/smoke-copy-header.txt"
      $BB grep -q 'ELF' "$WORK/smoke-copy-header.txt"
      "$out/bin/size" "$WORK/archive-smoke" > "$WORK/archive-smoke-size.txt"
      $BB grep -q 'archive-smoke' "$WORK/archive-smoke-size.txt"
      "$out/bin/strings" "$WORK/smoke.o" > "$WORK/smoke-strings.txt"
      $BB grep -q '_start' "$WORK/smoke-strings.txt"
      "$out/bin/strip" -o "$WORK/archive-smoke-stripped" "$WORK/archive-smoke"
      test -s "$WORK/archive-smoke-stripped"
      "$out/bin/readelf" -h "$WORK/archive-smoke-stripped" > "$WORK/archive-smoke-stripped-header.txt"
      $BB grep -q 'ELF' "$WORK/archive-smoke-stripped-header.txt"
      $BB chmod 755 "$WORK/archive-smoke-stripped"
      "$WORK/archive-smoke-stripped"

      $BB printf '.text\ninvalid_mantle_opcode %%rax\n' > "$WORK/rejected.s"
      $BB rm -f "$WORK/rejected.o"
      if "$out/bin/as" -o "$WORK/rejected.o" "$WORK/rejected.s"; then
        echo "ERROR: clean source assembler accepted malformed input" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB cat > "$WORK/undefined.s" <<'ASM'
.section .text
.global _start
_start:
  call mantle_missing_symbol
ASM
      "$out/bin/as" -o "$WORK/undefined.o" "$WORK/undefined.s"
      $BB rm -f "$WORK/undefined-output"
      if "$out/bin/ld" -static -o "$WORK/undefined-output" "$WORK/undefined.o"; then
        echo "ERROR: clean source linker accepted an undefined symbol" >&2
        exit 1
      fi
      test ! -e "$WORK/undefined-output"

      $BB printf '%s\n' 'not an object or archive' > "$WORK/rejected-input"
      if "$out/bin/readelf" -h "$WORK/rejected-input" > "$WORK/rejected-readelf.txt"; then
        echo "ERROR: clean source readelf accepted malformed input" >&2
        exit 1
      fi
      if "$out/bin/ar" t "$WORK/rejected-input" > "$WORK/rejected-ar.txt"; then
        echo "ERROR: clean source ar accepted malformed input" >&2
        exit 1
      fi
      if "$out/bin/ranlib" "$WORK/rejected-input"; then
        echo "ERROR: clean source ranlib accepted malformed input" >&2
        exit 1
      fi
      if "$out/bin/nm" "$WORK/rejected-input" > "$WORK/rejected-nm.txt"; then
        echo "ERROR: clean source nm accepted malformed input" >&2
        exit 1
      fi
      if "$out/bin/objdump" -h "$WORK/rejected-input" > "$WORK/rejected-objdump.txt"; then
        echo "ERROR: clean source objdump accepted malformed input" >&2
        exit 1
      fi
      $BB rm -f "$WORK/rejected-objcopy.o"
      if "$out/bin/objcopy" "$WORK/rejected-input" "$WORK/rejected-objcopy.o"; then
        echo "ERROR: clean source objcopy accepted malformed input" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-objcopy.o"
      if "$out/bin/size" "$WORK/rejected-input" > "$WORK/rejected-size.txt"; then
        echo "ERROR: clean source size accepted malformed input" >&2
        exit 1
      fi
      if "$out/bin/strings" "$WORK/missing-strings-input" > "$WORK/rejected-strings.txt"; then
        echo "ERROR: clean source strings accepted a missing input" >&2
        exit 1
      fi
      $BB rm -f "$WORK/rejected-strip-output"
      if "$out/bin/strip" -o "$WORK/rejected-strip-output" "$WORK/rejected-input"; then
        echo "ERROR: clean source strip accepted malformed input" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected-strip-output"

      $BB mkdir -p "$out/share/binutils-gcc-pass4-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: clean binutils still uses shipped generated sources pending repaired regeneration and complete provider normalization' \
        > "$out/share/binutils-gcc-pass4-frontier/NON_ADMISSION"
      echo "binutils-2.30 clean GCC pass4 runtime checks passed; not provider admission"
    "%
  ],
  inputs = [
    stage0,
    make,
    gcc,
    musl,
    stagex_provider,
    stagex_transition,
    binutils_src,
    automake115_src,
  ],
} | crunch.Derivation
