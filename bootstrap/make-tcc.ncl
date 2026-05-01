# Build GNU make 3.82 using tinycc 0.9.27.
#
# make is the first build system in the chain. Earlier stages used
# kaem scripts; from here on, Makefiles drive compilation.
#
# Adapted from live-bootstrap steps/make-3.82/pass1.kaem.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in

let make_src = crunch.fetchTarball {
  # first-consumer: bootstrap/make-tcc.ncl
  # provenance: fosslinux/live-bootstrap steps/make-3.82/pass1.kaem source input
  # ftpmirror.gnu.org returned 502 during runtime validation; use the stable
  # kernel.org GNU mirror for the same fixed-output tarball.
  url = "https://mirrors.kernel.org/gnu/make/make-3.82.tar.bz2",
  hash = "sha256-GwvLvcEtBMdaC/3YqmUkE9aoNTmrj9yQ39Vyd+Fw+M4=",
  name = "make-3.82-src",
} in

{
  name = "make-3.82-tcc",
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
      MES_OUT=$(find_input mes)
      STAGE0=$(find_input stage0-posix)
      MAKE_SRC=$(find_input make-3.82-src)

      export PATH="$TCC/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      WORK=/tmp/make-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      # fetchTarball strips single top-level dir
      $BB cp -r "$MAKE_SRC" make-3.82
      $BB chmod -R u+w make-3.82
      cd make-3.82

      # Create empty config.h
      : > config.h

      # TinyCC 0.9.27 on this Mes libc path rejects GNU Make's static
      # command-switch initializer for the `-j/--jobs` no-argument and
      # default values. This pass1 make is used only as a bootstrap driver,
      # so keep the option entry present but avoid the address constants that
      # TCC cannot compile.
      $BB sed -i \
        -e 's/&inf_jobs, &default_job_slots/0, 0/' \
        -e 's/&default_load_average/0/g' \
        -e 's/"basic", 0, "debug"/0, 0, "debug"/' \
        main.c
      # Compile each source file with tcc
      tcc -c getopt.c
      tcc -c getopt1.c
      tcc -c -I. -Iglob -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_STDINT_H ar.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_FCNTL_H arscan.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DFILE_TIMESTAMP_HI_RES=0 commands.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DSCCS_GET=\"/nullop\" default.c
      tcc -c -I. -Iglob -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_DIRENT_H dir.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART expand.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DFILE_TIMESTAMP_HI_RES=0 file.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -Dvfork=fork function.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART implicit.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_DUP2 -DHAVE_STRCHR -Dvfork=fork job.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DNO_FLOAT -DLOCALEDIR=\"/fake\" -DPACKAGE=\"make\" -DHAVE_MKTEMP -DHAVE_GETCWD main.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_STRERROR -DHAVE_VPRINTF -DHAVE_ANSI_COMPILER -DHAVE_STDARG_H misc.c
      tcc -c -I. -Iglob -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DINCLUDEDIR=\"/include\" read.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DFILE_TIMESTAMP_HI_RES=0 -DHAVE_FCNTL_H -DLIBDIR=\"/lib\" remake.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART rule.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART signame.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART strcache.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART variable.c
      tcc -c -I. -DVERSION=\"3.82\" version.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART vpath.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART hash.c
      tcc -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART remote-stub.c
      tcc -c -DHAVE_FCNTL_H getloadavg.c
      tcc -c -Iglob -DSTDC_HEADERS -o fnmatch.o glob/fnmatch.c
      tcc -c -Iglob -DHAVE_STRDUP -DHAVE_DIRENT_H -o glob.o glob/glob.c
      $BB chmod 644 *.o

      # Link
      tcc -L"$TCC/lib/mes" -L"$TCC/lib/mes/tcc" -static -o make \
        getopt.o getopt1.o ar.o arscan.o commands.o default.o dir.o \
        expand.o file.o function.o implicit.o job.o main.o misc.o \
        read.o remake.o rule.o signame.o strcache.o variable.o \
        version.o vpath.o hash.o remote-stub.o getloadavg.o \
        fnmatch.o glob.o

      # Verify
      ./make --version

      # Install
      $BB mkdir -p "$out/bin"
      $BB cp make "$out/bin/make"
      $BB chmod 755 "$out/bin/make"

      test -x "$out/bin/make"
    "%,
  ],
  inputs = [stage0, mes, tcc, make_src],
} | crunch.Derivation
