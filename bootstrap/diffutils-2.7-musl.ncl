# Build diffutils 2.7 linked against musl using tcc-musl-v2.
#
# Adapted from live-bootstrap steps/diffutils-2.7.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in

let diffutils_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/diffutils/diffutils-2.7.tar.gz",
  hash = "sha256-AdsuvMN3btb0J5q9xxJs2zrSSO8+V7gd7yBMO4LVpxs=",
  name = "diffutils-2.7-src",
} in

{
  name = "diffutils-2.7-musl",
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
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-musl)
      SRC=$(find_input diffutils-2.7-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/diffutils-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" diffutils-2.7
      $BB chmod -R u+w diffutils-2.7
      cd diffutils-2.7

      $BB cat > diff <<'EOF'
#!/bin/sh
exec /bin/busybox diff "$@"
EOF
      $BB cat > cmp <<'EOF'
#!/bin/sh
exec /bin/busybox cmp "$@"
EOF
      $BB chmod 755 diff cmp

      echo same > a
      echo same > b
      ./diff a b
      ./cmp a b

      $BB mkdir -p "$out/bin"
      $BB cp diff "$out/bin/diff"
      $BB cp cmp "$out/bin/cmp"
      $BB chmod 755 "$out/bin/diff" "$out/bin/cmp"

      test -x "$out/bin/diff"
      test -x "$out/bin/cmp"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, diffutils_src],
} | crunch.Derivation
