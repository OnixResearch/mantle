# Diagnostic binutils 2.41 frontier over final GCC 10 + musl 1.2.5.
#
# This root validates real assembler/linker/archive/object-tool behavior. It
# remains non-admission until source regeneration, pure derivational closure,
# normalized provider metadata, and fixed-point evidence are complete.

let crunch = import "lib.ncl" in
let gcc10 = crunch.derivationFile "gcc-10-final.ncl" in
let musl125 = crunch.derivationFile "musl-full.ncl" in

let binutils_src = crunch.derivationFile "binutils-2.41-regeneration-ready-source.ncl" in

{
  name = "binutils-2.41-gcc10-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=10.5.0
      JOB_COUNT=1
      BUILD_FLAGS='-O2 -fno-common'
      WORK=/tmp/binutils-2.41-gcc10
      TOOLBIN="$WORK/tools"

      find_input() {
        input_suffix=$1
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            echo "$input_path"
            return 0
          fi
        done
        echo "ERROR: input $input_suffix not found" >&2
        exit 1
      }

      GCC=$(find_input gcc-10.5.0-musl-final-v1)
      MUSL=$(find_input musl-1.2.5-gcc10-v1)
      BINUTILS_OLD=$(find_input binutils-2.30-gcc-pass4-v16)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      GREP=$(find_input grep-2.4-gcc-v10)
      GAWK=$(find_input gawk-3.0.4-gcc-v4)
      M4=$(find_input m4-1.4.7-gcc-v5)
      FLEX=$(find_input flex-2.6.4-gcc-v1)
      BISON=$(find_input bison-3.4.1-gcc-v2)
      SRC=$(find_input binutils-2.41-regeneration-ready-source-v1)

      test -x "$GCC/bin/gcc"
      test -x "$GCC/bin/g++"
      test -s "$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
      test -s "$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc_eh.a"
      LIBGCC_S=$($BB find "$GCC" -type f -name libgcc_s.so.1 | $BB head -n 1)
      test -n "$LIBGCC_S"
      test -s "$LIBGCC_S"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in basename cat chmod cmp cp cut date diff dirname echo egrep env expr false find grep head install ln ls mkdir mv od printf pwd readlink rm rmdir sed sh sleep sort tail tar test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
      $BB ln -sf "$GAWK/bin/awk" "$TOOLBIN/awk"
      $BB ln -sf "$GAWK/bin/awk" "$TOOLBIN/gawk"
      $BB ln -sf "$M4/bin/m4" "$TOOLBIN/m4"
      $BB ln -sf "$FLEX/bin/flex" "$TOOLBIN/flex"
      $BB ln -sf "$BISON/bin/bison" "$TOOLBIN/bison"
      $BB ln -sf "$MAKE/bin/make" "$TOOLBIN/make"
      export PATH="$TOOLBIN:$BINUTILS_OLD/bin:$STAGE0/bin"
      export M4="$M4/bin/m4"
      export BISON_PKGDATADIR="$BISON/share/bison"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh
      export MAKEFLAGS=

      $BB cat > "$TOOLBIN/gcc10" <<EOF
#!/bin/sh
export MANTLE_GCC_ASSEMBLER="$BINUTILS_OLD/bin/as"
export MANTLE_GCC_LINKER="$BINUTILS_OLD/bin/ld"
export MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a"
export MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a"
export MANTLE_GCC_LINK_LIBGCC="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
export MANTLE_GCC_LINK_LIBGCC_EH="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc_eh.a"
export MANTLE_GCC_LINK_LIBGCC_S="$LIBGCC_S"
export MANTLE_GCC_LINK_CRT="$MUSL/lib"
exec "$GCC/bin/gcc" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" -isystem "$MUSL/include" "\$@"
EOF
      $BB cat > "$TOOLBIN/gxx10" <<EOF
#!/bin/sh
export MANTLE_GCC_ASSEMBLER="$BINUTILS_OLD/bin/as"
export MANTLE_GCC_LINKER="$BINUTILS_OLD/bin/ld"
export MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a"
export MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a"
export MANTLE_GCC_LINK_LIBGCC="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
export MANTLE_GCC_LINK_LIBGCC_EH="$GCC/lib/gcc/$TARGET/$GCC_VERSION/libgcc_eh.a"
export MANTLE_GCC_LINK_LIBGCC_S="$LIBGCC_S"
export MANTLE_GCC_LINK_CRT="$MUSL/lib"
exec "$GCC/bin/g++" -B"$GCC/bin/" -B"$GCC/lib/gcc/$TARGET/$GCC_VERSION/" -isystem "$MUSL/include" "\$@"
EOF
      $BB chmod 755 "$TOOLBIN/gcc10" "$TOOLBIN/gxx10"
      HOST_CC="$TOOLBIN/gcc10"
      HOST_CXX="$TOOLBIN/gxx10"

      $BB cp -R "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      test -s "$WORK/src/.mantle-regeneration/binutils-2.41-configure-files.txt"
      test -s "$WORK/src/.mantle-regeneration/binutils-2.41-removed-generated-files.txt"
      $BB mkdir -p "$WORK/build"
      cd "$WORK/build"

      CC="$HOST_CC" \
      CXX="$HOST_CXX" \
      CPP="$HOST_CC -E" \
      CXXCPP="$HOST_CXX -E" \
      AR="$BINUTILS_OLD/bin/ar" \
      RANLIB="$BINUTILS_OLD/bin/ranlib" \
      "$WORK/src/configure" \
        --prefix="$out" \
        --build="$TARGET" \
        --host="$TARGET" \
        --target="$TARGET" \
        --disable-nls \
        --disable-shared \
        --enable-static \
        --disable-werror \
        --disable-gdb \
        --disable-gdbserver \
        --disable-gold \
        --disable-libdecnumber \
        --disable-readline \
        --disable-sim \
        --disable-gprofng \
        --disable-debuginfod \
        --without-zstd \
        --with-sysroot="$MUSL" \
        CFLAGS="$BUILD_FLAGS -I$MUSL/include" \
        CXXFLAGS="$BUILD_FLAGS -I$MUSL/include" \
        LDFLAGS="-static -L$MUSL/lib" \
        YACC="$BISON/bin/bison -y" \
        LEX="$FLEX/bin/flex" \
        MAKEINFO=true

      make -j"$JOB_COUNT" MAKEINFO=true || {
        build_status=$?
        for failure_log in "$WORK/build/binutils/config.log" "$WORK/build/gas/config.log" "$WORK/build/ld/config.log"; do
          if [ -f "$failure_log" ]; then
            $BB cat "$failure_log" >&2
          fi
        done
        exit "$build_status"
      }
      make -j"$JOB_COUNT" install MAKEINFO=true

      # Require the parser/lexer products selected by this x86_64 build at
      # their generated build-tree locations. Architecture- and PE-only
      # grammars remain removed from the prepared source but are not falsely
      # claimed as compiled by this target.
      for regenerated_relative_path in \
        binutils/arparse.c \
        binutils/arlex.c \
        binutils/defparse.c \
        binutils/sysinfo.c \
        binutils/syslex.c \
        ld/ldgram.c \
        ld/ldlex.c
      do
        regenerated_path="$WORK/build/$regenerated_relative_path"
        test -s "$regenerated_path"
        echo "binutils241-regenerated: $regenerated_relative_path=$regenerated_path"
      done
      for compiled_generated_object in \
        binutils/arparse.o \
        binutils/arlex.o \
        binutils/sysinfo.o \
        binutils/syslex_wrap.o \
        ld/ldgram.o \
        ld/ldlex-wrapper.o
      do
        compiled_object_path="$WORK/build/$compiled_generated_object"
        test -s "$compiled_object_path"
        echo "binutils241-regenerated-object: $compiled_generated_object=$compiled_object_path"
      done

      for required_tool in as ld ar ranlib nm objcopy objdump readelf size strings strip; do
        test -x "$out/bin/$required_tool"
      done

      $BB cat > "$WORK/positive.s" <<'EOF'
.section .rodata
message:
  .ascii "binutils-2.41-runtime-ok"
.section .text
.global _start
_start:
  mov $60, %rax
  mov $42, %rdi
  syscall
EOF
      "$out/bin/as" -o "$WORK/positive.o" "$WORK/positive.s"
      test -s "$WORK/positive.o"
      "$out/bin/readelf" -h "$WORK/positive.o" | "$GREP/bin/grep" -q 'ELF64'
      "$out/bin/nm" "$WORK/positive.o" | "$GREP/bin/grep" -q '_start'
      "$out/bin/objdump" -d "$WORK/positive.o" | "$GREP/bin/grep" -q 'syscall'
      "$out/bin/strings" "$WORK/positive.o" | "$GREP/bin/grep" -q 'binutils-2.41-runtime-ok'
      "$out/bin/size" "$WORK/positive.o" > "$WORK/positive.size"
      test -s "$WORK/positive.size"

      "$out/bin/ld" -o "$WORK/positive-bin" "$WORK/positive.o"
      test -s "$WORK/positive-bin"
      $BB chmod 755 "$WORK/positive-bin"
      set +e
      "$WORK/positive-bin"
      runtime_status=$?
      set -e
      test "$runtime_status" -eq 42

      "$out/bin/ar" rc "$WORK/libpositive.a" "$WORK/positive.o"
      "$out/bin/ranlib" "$WORK/libpositive.a"
      "$out/bin/ar" t "$WORK/libpositive.a" | "$GREP/bin/grep" -q 'positive.o'
      "$out/bin/objcopy" "$WORK/positive.o" "$WORK/positive-copy.o"
      test -s "$WORK/positive-copy.o"
      $BB cp "$WORK/positive-copy.o" "$WORK/positive-strip.o"
      "$out/bin/strip" "$WORK/positive-strip.o"
      test -s "$WORK/positive-strip.o"

      $BB printf '%s\n' '.this-is-not-an-assembler-directive' > "$WORK/rejected.s"
      $BB rm -f "$WORK/rejected.o"
      if "$out/bin/as" -o "$WORK/rejected.o" "$WORK/rejected.s"; then
        echo "ERROR: binutils 2.41 assembler accepted malformed input" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected.o"

      $BB cat > "$WORK/undefined.s" <<'EOF'
.global _start
.extern mantle_missing_symbol
_start:
  call mantle_missing_symbol
EOF
      "$out/bin/as" -o "$WORK/undefined.o" "$WORK/undefined.s"
      $BB rm -f "$WORK/undefined-bin"
      if "$out/bin/ld" -o "$WORK/undefined-bin" "$WORK/undefined.o"; then
        echo "ERROR: binutils 2.41 linker accepted undefined symbol" >&2
        exit 1
      fi
      test ! -e "$WORK/undefined-bin"

      RELOCATED_BINUTILS="$WORK/relocated-binutils"
      $BB cp -R "$out" "$RELOCATED_BINUTILS"
      "$RELOCATED_BINUTILS/bin/as" -o "$WORK/relocated.o" "$WORK/positive.s"
      "$RELOCATED_BINUTILS/bin/ld" -o "$WORK/relocated-bin" "$WORK/relocated.o"
      test -s "$WORK/relocated-bin"
      $BB chmod 755 "$WORK/relocated-bin"
      set +e
      "$WORK/relocated-bin"
      relocated_status=$?
      set -e
      test "$relocated_status" -eq 42

      $BB mkdir -p "$out/share/binutils241-frontier"
      $BB printf '%s\n' \
        'NON_ADMISSION: binutils 2.41 has component-level regenerated-source and runtime evidence; normalized-provider closure and external identity admission remain separate' \
        > "$out/share/binutils241-frontier/NON_ADMISSION"
      echo "binutils 2.41 assembler/linker/archive/object runtime, relocation, and rejection checks passed; not provider admission"
    "%
  ],
  inputs = [
    gcc10,
    musl125,
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "flex-2.6.4-gcc.ncl",
    crunch.derivationFile "bison-3.4.1-gcc.ncl",
    binutils_src,
  ],
} | crunch.Derivation
