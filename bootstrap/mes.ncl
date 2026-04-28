# Build GNU mes 0.27.1 from M2-Planet and mescc-tools.
#
# mes provides:
# - mes-m2: Scheme interpreter (built from M2-Planet/C via mescc-tools)
# - mescc: C compiler (a Scheme program running on mes)
# - mes libc: minimal C library for programs compiled by mescc
#
# Adapted from live-bootstrap steps/mes-0.27.1/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in

let mes_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/mes/mes-0.27.1.tar.gz",
  hash = "sha256-pWARnEISClTfRHAhu69MRaVilq2UFuWsBcKMREsixmk=",
  name = "mes-src",
} in

let nyacc_src = crunch.fetchTarball {
  url = "https://github.com/Googulator/nyacc/releases/download/V1.00.2-lb1/nyacc-1.00.2-lb1.tar.gz",
  hash = "sha256-QKSPdXF1oEUAo4yZcPXKhxeCiUJ3us4CynQVzNsMXWc=",
  name = "nyacc-src",
} in

{
  name = "mes",
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

      STAGE0=$(find_input stage0-posix)
      MES_SRC=$(find_input mes-src)
      NYACC_SRC=$(find_input nyacc-src)

      ARCH=amd64
      MES_ARCH=x86_64
      ARCH_BITS=64
      MES_ARENA=20000000
      MES_MAX_ARENA=20000000
      MES_STACK=6000000
      NYACC_C99_ARENA=32000000
      MES_LIB_ARENA=30000000
      MES_LIB_MAX_ARENA="$MES_LIB_ARENA"
      MES_LIB_STACK=15000000

      # Set up PATH with stage0-posix tools.
      export PATH="$STAGE0/bin:$PATH"

      WORK=/tmp/mes-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      # fetchTarball strips the single top-level directory, so source
      # files are directly in the store path root.
      $BB cp -r "$MES_SRC" mes-0.27.1
      $BB cp -r "$NYACC_SRC" nyacc
      $BB chmod -R u+w mes-0.27.1 nyacc

      MES_PREFIX="$WORK/mes-0.27.1"
      NYACC_PREFIX="$WORK/nyacc"
      export MES_ARENA MES_MAX_ARENA MES_STACK MES_PREFIX

      # Provide config.h
      $BB mkdir -p "$MES_PREFIX/include/mes"
      $BB cat > "$MES_PREFIX/include/mes/config.h" << 'EOF'
#undef SYSTEM_LIBC
#define MES_VERSION "0.27.1"
EOF

      # Patch wait4.c (upstreamed after 0.27.1)
      cd "$MES_PREFIX"

      # Set up arch includes
      $BB mkdir -p include/arch
      $BB cp include/linux/${MES_ARCH}/kernel-stat.h include/arch/kernel-stat.h
      $BB cp include/linux/${MES_ARCH}/signal.h include/arch/signal.h
      $BB cp include/linux/${MES_ARCH}/syscall.h include/arch/syscall.h

      # Remove pregenerated files (rebuild from source).
      $BB rm -f mes/module/mes/psyntax.pp mes/module/mes/psyntax.pp.header 2>/dev/null || true

      # Symlinked files in the repo need to be real copies.
      if [ -f mes/module/srfi/srfi-9-struct.mes ]; then
        $BB cp mes/module/srfi/srfi-9-struct.mes mes/module/srfi/srfi-9.mes
      fi
      if [ -f mes/module/srfi/srfi-9/gnu-struct.mes ]; then
        $BB cp mes/module/srfi/srfi-9/gnu-struct.mes mes/module/srfi/srfi-9/gnu.mes
      fi

      # Build mes-m2 using M2-Planet (from stage0-posix).
      # The kaem.x86_64 script in the mes source drives this.
      kaem --verbose --strict --file kaem.${MES_ARCH}

      # Rebuild NYACC's generated C parser tables with mes-m2. The pregenerated
      # release files can carry Guile-specific assumptions that make mescc emit
      # invalid M1 for tcc.
      cd "$NYACC_PREFIX"
      $BB rm -f \
        module/nyacc/lang/c99/mach.d/c99-act.scm \
        module/nyacc/lang/c99/mach.d/c99-tab.scm \
        module/nyacc/lang/c99/mach.d/c99cx-act.scm \
        module/nyacc/lang/c99/mach.d/c99cx-tab.scm \
        module/nyacc/lang/c99/mach.d/c99x-act.scm \
        module/nyacc/lang/c99/mach.d/c99x-tab.scm \
        module/nyacc/lang/c99/mach.d/cpp-act.scm \
        module/nyacc/lang/c99/mach.d/cpp-tab.scm
      export GUILE_LOAD_PATH="$MES_PREFIX/mes/module:$MES_PREFIX/module:$NYACC_PREFIX/module"
      "$MES_PREFIX/bin/mes-m2" -L module gen-cpp-files.scm
      MES_ARENA="$NYACC_C99_ARENA" "$MES_PREFIX/bin/mes-m2" -L module gen-c99-files.scm
      MES_ARENA=20000000 "$MES_PREFIX/bin/mes-m2" -L module gen-c99cx-files.scm
      cd "$MES_PREFIX"

      $BB mkdir -p "$out/bin" "$out/lib" "$out/lib/${MES_ARCH}-mes" "$out/lib/linux/${MES_ARCH}-mes"
      $BB mkdir -p "$out/include" "$out/mes"

      # Install mes-m2 binary
      $BB cp bin/mes-m2 "$out/bin/mes-m2"
      $BB chmod 755 "$out/bin/mes-m2"

      # Generate mescc.scm from template
      $BB sed \
        -e "s|@prefix@|$out|g" \
        -e 's|@VERSION@|0.27.1|g' \
        -e "s|@mes_cpu@|${MES_ARCH}|g" \
        -e 's|@mes_kernel@|linux|g' \
        -e "s|@guile_site_dir@|$out/mes/module|g" \
        -e "s|@guile_site_ccache_dir@|$out/mes/module|g" \
        -e "s|@GUILE@|$out/bin/mes-m2|g" \
        scripts/mescc.scm.in > "$out/bin/mescc.scm"
      $BB chmod 755 "$out/bin/mescc.scm"

      # Install mes libc object files and libraries
      for f in crt1.s crt1.o ${MES_ARCH}.M1 libmescc.s libc+tcc.s libc.s \
               libmescc.a libc+tcc.a libc.a; do
        if [ -f "lib/${MES_ARCH}-mes/$f" ]; then
          $BB cp "lib/${MES_ARCH}-mes/$f" "$out/lib/${MES_ARCH}-mes/$f"
        fi
      done

      # Install ELF headers
      for f in "elf${ARCH_BITS}-header.hex2" "elf${ARCH_BITS}-footer-single-main.hex2"; do
        if [ -f "lib/linux/${MES_ARCH}-mes/$f" ]; then
          $BB cp "lib/linux/${MES_ARCH}-mes/$f" "$out/lib/linux/${MES_ARCH}-mes/$f"
        fi
      done

      # Install mes headers for later stages (tinycc)
      $BB cp -r include "$out/include/mes-include"

      # Install mes Scheme modules at $out/mes/module so mes finds
      # them when MES_PREFIX=$out (it searches $MES_PREFIX/mes/module/)
      $BB cp -r mes/module "$out/mes/module"

      # Merge top-level module/ (mescc compiler, mes/getopt-long, etc.)
      # module/mescc/ and module/mescc.scm are the actual compiler.
      # module/mes/ has some files that overlap with mes/module/mes/ but
      # the mes/module/ versions are the correct ones for mes-m2, so
      # only copy non-overlapping files.
      $BB cp -r module/mescc "$out/mes/module/mescc"
      $BB cp module/mescc.scm "$out/mes/module/mescc.scm"
      for f in getopt-long.scm misc.scm test.scm; do
        if [ -f "module/mes/$f" ]; then
          $BB cp "module/mes/$f" "$out/mes/module/mes/$f"
        fi
      done

      # Merge nyacc modules into the same tree
      $BB cp -r "$NYACC_PREFIX/module"/* "$out/mes/module/"

      # Preserve Mes-safe compatibility shims that NYACC would overwrite.
      for f in \
        nyacc/compat18.scm \
        nyacc/lang/c99/pprint.scm \
        rnrs/arithmetic/bitwise.scm \
        system/foreign.scm; do
        $BB mkdir -p "$out/mes/module/$($BB dirname "$f")"
        $BB cp "mes/module/$f" "$out/mes/module/$f"
      done

      # nyacc compatibility modules reference (ice-9 syncase) for Guile 1.8.
      # Mes supplies syntax-rules directly, so an empty module shim is enough
      # to satisfy #:use-syntax without dragging Guile-specific code into mes-m2.
      $BB mkdir -p "$out/mes/module/ice-9"
      $BB cat > "$out/mes/module/ice-9/syncase.scm" << 'EOF'
(define-module (ice-9 syncase))
EOF

      # mes-m2 reports successful system* calls as nonzero pointer-like values
      # on this bootstrap path. mescc's command runner treats any nonzero value
      # as failure, so normalize it before using mescc to build libc archives.
      $BB sed -i '/(define (assert-system\* \. args)/,/    status))/c\
(define (assert-system* . args)\
  (apply system* args)\
  0)' "$out/mes/module/mescc/mescc.scm"

      # Build mes libc archives with mescc's own mescc-flavored source files.
      # The gcc-flavored sources contain GNU inline assembly that mescc cannot
      # parse; compiler=mescc selects the Mes assembly syntax variants.
      APPLET_DIR="$WORK/busybox-bin"
      $BB mkdir -p "$APPLET_DIR"
      for tool in basename cat chmod cp dirname echo false mkdir rm sed test true; do
        $BB ln -sf "$BB" "$APPLET_DIR/$tool"
      done

      $BB sed \
        -e 's|@SHELL@|/bin/sh|g' \
        -e 's|@VERSION@|0.27.1|g' \
        scripts/mesar.in > "$WORK/mesar"
      $BB chmod 755 "$WORK/mesar"

      $BB cat > "$WORK/mescc-wrapper" << EOF
#!/bin/sh
exec "$out/bin/mes-m2" --no-auto-compile -e main "$out/bin/mescc.scm" -- "\$@"
EOF
      $BB chmod 755 "$WORK/mescc-wrapper"

      LIB_BUILD="$WORK/mes-lib-build"
      $BB rm -rf "$LIB_BUILD"
      $BB mkdir -p "$LIB_BUILD"
      $BB cat > "$LIB_BUILD/config.sh" << EOF
config_sh=1
AR=$WORK/mesar
CC=$WORK/mescc-wrapper
CFLAGS=
CPPFLAGS=
AM_CPPFLAGS="-D HAVE_CONFIG_H=1 -I$MES_PREFIX/include -I$MES_PREFIX/include/linux/$MES_ARCH"
AM_CFLAGS=
AM_LDFLAGS=
LDFLAGS=
LIBS=
compiler=mescc
courageous=true
mes_cpu=$MES_ARCH
mes_bits=$ARCH_BITS
mes_libc=mes
mes_kernel=linux
srcdest=$MES_PREFIX/
srcdir=$MES_PREFIX
V=1
EOF
      cd "$LIB_BUILD"
      PATH="$APPLET_DIR:$STAGE0/bin:$out/bin:$PATH" \
      MES_PREFIX="$out" \
      GUILE_LOAD_PATH="$out/mes/module" \
      MES_ARENA="$MES_LIB_ARENA" \
      MES_MAX_ARENA="$MES_LIB_MAX_ARENA" \
      MES_STACK="$MES_LIB_STACK" \
        /bin/sh "$MES_PREFIX/build-aux/build-lib.sh"
      $BB cp ${MES_ARCH}-mes/* "$out/lib/${MES_ARCH}-mes/"
      cd "$MES_PREFIX"

      # Install M2libc for later M2-Planet-based builds
      $BB cp -r "$STAGE0/lib/M2libc" "$out/lib/M2libc"

      # Verify critical outputs
      test -x "$out/bin/mes-m2"
      test -d "$out/lib/${MES_ARCH}-mes"
      test -d "$out/include/mes-include"
      for f in ${MES_ARCH}.M1 crt1.o libmescc.a libc.a libc+tcc.a libtcc1.a; do
        test -s "$out/lib/${MES_ARCH}-mes/$f"
      done
    "%,
  ],
  inputs = [stage0, mes_src, nyacc_src],
} | crunch.Derivation
