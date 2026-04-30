# Rebuild tcc 0.9.27 against musl 1.1.24.
#
# Now that musl is available, recompile tcc to link against musl
# instead of mes libc. This produces a tcc that generates musl-linked
# binaries.
#
# Adapted from live-bootstrap steps/tcc-0.9.27 (musl pass).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc_prep = import "tcc-musl-prep.ncl" in
let musl = import "musl-1.1.24-tcc.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in

let tcc27_src = crunch.fetchTarball {
  # first-consumer: bootstrap/tcc-musl.ncl
  # provenance: fosslinux/live-bootstrap steps/tcc-0.9.27 musl pass source input
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-J1qVIv8ti91hOO6Gp2Wqho3t8C7jQHcXpTI85IPwZn0=",
  name = "tcc-0.9.27-src",
} in

{
  name = "tcc-0.9.27-musl",
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

      TCC_PREP=$(find_input tcc-0.9.27-musl-prep)
      MUSL=$(find_input musl-1.1.24-tcc)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-tcc)
      TCC27_SRC=$(find_input tcc-0.9.27-src)

      export PATH="$TCC_PREP/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/tcc-musl-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$TCC27_SRC" tcc-0.9.27
      $BB chmod -R u+w tcc-0.9.27
      cd tcc-0.9.27

      : > config.h

      # Build tcc linked against musl
      tcc \
        -v -static \
        -o tcc-musl \
        -D TCC_TARGET_X86_64=1 \
        -D CONFIG_TCCDIR=\"$out/lib/tcc\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MUSL/lib\" \
        -D CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\" \
        -D CONFIG_TCC_LIBPATHS=\"$MUSL/lib\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MUSL/include"'"' \
        -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.27\" \
        -D ONE_SOURCE=1 \
        -I "$MUSL/include" \
        tcc.c
      $BB chmod 755 tcc-musl

      $BB mkdir -p "$out/bin" "$out/lib/tcc"
      $BB cp tcc-musl "$out/bin/tcc"
      $BB cp tcc-musl "$out/bin/tcc-0.9.27-musl"
      $BB chmod 755 "$out/bin/tcc" "$out/bin/tcc-0.9.27-musl"

      # libtcc1.a is part of the compiler runtime contract.  Do not silently
      # produce a compiler output that cannot link downstream programs.
      if [ ! -f libtcc1.a ]; then
        echo "ERROR: tcc musl build did not produce libtcc1.a" >&2
        exit 1
      fi
      $BB cp libtcc1.a "$out/lib/tcc/"

      test -x "$out/bin/tcc"
      test -x "$out/bin/tcc-0.9.27-musl"
      test -s "$out/lib/tcc/libtcc1.a"
      $BB cat > smoke.c <<'EOF'
int main(void) { return 0; }
EOF
      "$out/bin/tcc" -static smoke.c -o smoke
      test -s smoke
    "%,
  ],
  inputs = [stage0, mes, tcc_prep, musl, make, sed, tcc27_src],
} | crunch.Derivation
