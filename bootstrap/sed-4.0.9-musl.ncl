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

      : > config.h

      CFLAGS="-I. -Ilib -I$MUSL/include -DENABLE_NLS=0 -DHAVE_FCNTL_H -DHAVE_ALLOCA_H -DSED_FEATURE_VERSION=\\\"4.0\\\" -DVERSION=\\\"4.0.9\\\" -DPACKAGE=\\\"sed\\\""

      for src in getline getopt1 getopt utils obstack strverscmp mkstemp; do
        tcc -c $CFLAGS lib/$src.c -o lib/$src.o
      done
      tcc -ar cr libsed.a lib/getline.o lib/getopt1.o lib/getopt.o lib/utils.o lib/obstack.o lib/strverscmp.o lib/mkstemp.o

      for src in compile execute regexp fmt sed; do
        tcc -c $CFLAGS sed/$src.c -o sed/$src.o
      done

      tcc -static sed/compile.o sed/execute.o sed/regexp.o sed/fmt.o sed/sed.o -L. -lsed -o sed

      echo test | ./sed 's/test/ok/'

      $BB mkdir -p "$out/bin"
      $BB cp sed "$out/bin/sed"
      $BB chmod 755 "$out/bin/sed"

      test -x "$out/bin/sed"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed_old, sed_src],
} | crunch.Derivation
