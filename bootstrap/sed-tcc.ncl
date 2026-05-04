# Build GNU sed 4.0.9 using tinycc 0.9.27 + make 3.82.
#
# First text-processing tool in the intermediate chain.
# Provenance: fosslinux/live-bootstrap steps/sed-4.0.9/pass1.kaem and mk/main.mk.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
let make = import "make-tcc.ncl" in

let sed_src = crunch.fetchTarball {
  # first-consumer: bootstrap/sed-tcc.ncl
  # provenance: fosslinux/live-bootstrap steps/sed-4.0.9/sources
  url = "https://ftpmirror.gnu.org/sed/sed-4.0.9.tar.gz",
  hash = "sha256-k2YhQtIAYx0rPNFootM1VKppNsaoJsaTFpJ34SSPkIk=",
  name = "sed-4.0.9-src",
} in

{
  name = "sed-4.0.9-tcc",
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

      TCC=$(find_input tinycc-0.9.27)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      MES_OUT=$(find_input mes)
      SED_SRC=$(find_input sed-4.0.9-src)

      export PATH="$TCC/bin:$MAKE/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      WORK=/tmp/sed-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      # fetchTarball strips single top-level dir
      $BB cp -r "$SED_SRC" sed-4.0.9
      $BB chmod -R u+w sed-4.0.9
      cd sed-4.0.9

      # Create empty config.h
      : > config.h

      # Compile library sources
      for src in getline getopt1 getopt utils obstack strverscmp mkstemp; do
        tcc -c -I. -Ilib \
          -DENABLE_NLS=0 -DHAVE_FCNTL_H -DHAVE_ALLOCA_H \
          -DSED_FEATURE_VERSION=\"4.0\" -DVERSION=\"4.0.9\" -DPACKAGE=\"sed\" \
          lib/$src.c -o lib/$src.o
        test -f lib/$src.o
      done
      # regex needs regex.h copy
      $BB cp lib/regex_.h lib/regex.h
      tcc -c -I. -Ilib \
        -DENABLE_NLS=0 -DHAVE_FCNTL_H -DHAVE_ALLOCA_H \
        -DSED_FEATURE_VERSION=\"4.0\" -DVERSION=\"4.0.9\" -DPACKAGE=\"sed\" \
        lib/regex.c -o lib/regex.o
      test -f lib/regex.o

      LIB_OBJS="lib/getline.o lib/getopt1.o lib/getopt.o lib/utils.o lib/obstack.o lib/strverscmp.o lib/mkstemp.o lib/regex.o"

      # Compile sed sources
      for src in compile execute regexp fmt sed; do
        tcc -c -I. -Ilib \
          -DENABLE_NLS=0 -DHAVE_FCNTL_H -DHAVE_ALLOCA_H \
          -DSED_FEATURE_VERSION=\"4.0\" -DVERSION=\"4.0.9\" -DPACKAGE=\"sed\" \
          sed/$src.c -o sed/$src.o
        test -f sed/$src.o
      done

      # Link directly against object files. This avoids relying on early
      # TinyCC/Mes archive handling for a package whose output contract is only
      # the sed executable.
      tcc -static \
        sed/compile.o sed/execute.o sed/regexp.o sed/fmt.o sed/sed.o \
        $LIB_OBJS -o sed/sed

      # Verify
      ./sed/sed --version
      echo test | ./sed/sed 's/test/ok/' | $BB grep '^ok$'

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp sed/sed "$out/bin/sed"
      $BB chmod 755 "$out/bin/sed"

      test -x "$out/bin/sed"
    "%,
  ],
  inputs = [stage0, mes, tcc, make, sed_src],
} | crunch.Derivation
