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

      $BB mkdir -p "$out/bin" "$out/lib" "$out/lib/${MES_ARCH}-mes" "$out/lib/linux/${MES_ARCH}-mes"
      $BB mkdir -p "$out/include" "$out/mes"

      # Install mes-m2 binary
      $BB cp bin/mes-m2 "$out/bin/mes-m2"
      $BB chmod 755 "$out/bin/mes-m2"

      # Install mescc (the Scheme-based C compiler script)
      if [ -f scripts/mescc.scm ]; then
        $BB cp scripts/mescc.scm "$out/bin/mescc.scm"
      elif [ -f scripts/mescc ]; then
        $BB cp scripts/mescc "$out/bin/mescc.scm"
      fi

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

      # Merge nyacc modules into the same tree
      $BB cp -r "$NYACC_PREFIX/module"/* "$out/mes/module/"

      # Install M2libc for later M2-Planet-based builds
      $BB cp -r "$STAGE0/lib/M2libc" "$out/lib/M2libc"

      # Verify critical outputs
      test -x "$out/bin/mes-m2"
      test -d "$out/lib/${MES_ARCH}-mes"
      test -d "$out/include/mes-include"
    "%,
  ],
  inputs = [stage0, mes_src, nyacc_src],
} | crunch.Derivation
