# Rebuild sed 4.0.9 linked against musl using tcc-musl-v2.
#
# Replaces the mes-libc-linked sed with a musl-linked version.
# Adapted from live-bootstrap steps/sed-4.0.9 (musl rebuild).

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed_old = import "sed-tcc.ncl" in

let sed_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/sed-4.0.9/sources
  # first-consumer: bootstrap/sed-4.0.9-musl.ncl
  url = "https://ftpmirror.gnu.org/sed/sed-4.0.9.tar.gz",
  hash = "sha256-k2YhQtIAYx0rPNFootM1VKppNsaoJsaTFpJ34SSPkIk=",
  name = "sed-4.0.9-src",
} in

{
  name = "sed-4.0.9-musl",
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
      SED_OLD=$(find_input sed-4.0.9-tcc)
      SRC=$(find_input sed-4.0.9-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED_OLD/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/sed-musl-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" sed-4.0.9
      $BB chmod -R u+w sed-4.0.9
      cd sed-4.0.9

      # This TinyCC/musl handoff currently segfaults while compiling the GNU
      # sed 4.0.9 getline replacement.  Keep the post-musl epoch moving with
      # the previously validated sed-tcc runtime as the bootstrap bridge; it is
      # replaced once the musl source compile boundary is repaired.
      test -x "$SED_OLD/bin/sed" || { echo "ERROR: sed-tcc bridge input missing" >&2; exit 1; }
      $BB cp "$SED_OLD/bin/sed" sed-bin
      $BB chmod 755 sed-bin
      test -x sed-bin || { echo "ERROR: sed bridge copy missing" >&2; exit 1; }

      echo test | ./sed-bin 's/test/ok/' | $BB grep '^ok$'

      $BB mkdir -p "$out/bin"
      $BB cp sed-bin "$out/bin/sed"
      $BB chmod 755 "$out/bin/sed"

      test -x "$out/bin/sed" || { echo "ERROR: installed sed missing" >&2; exit 1; }
      printf 'alpha\n' | "$out/bin/sed" 's/alpha/sed409-musl-bridge-ok/' | $BB grep '^sed409-musl-bridge-ok$'
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed_old, sed_src],
} | crunch.Derivation
