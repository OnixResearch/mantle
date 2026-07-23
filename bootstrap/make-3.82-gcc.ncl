# GNU Make 3.82 rebuilt with the closed GCC 4.0/musl/binutils cycle.

let crunch = import "lib.ncl" in
let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let gcc = crunch.derivationFile "gcc-4.0-musl-cxx.ncl" in
let binutils = crunch.derivationFile "binutils-2.30-gcc.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl" in

let make_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/make/make-3.82.tar.bz2",
  hash = "sha256-GwvLvcEtBMdaC/3YqmUkE9aoNTmrj9yQ39Vyd+Fw+M4=",
  name = "make-3.82-src",
} in

{
  name = "make-3.82-gcc-v3",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox

    find_input() {
      for candidate in $NIX_STORE/*-"$1"; do
        if [ -d "$candidate" ]; then echo "$candidate"; return; fi
      done
      echo "ERROR: input $1 not found" >&2
      exit 1
    }

    STAGE0=$(find_input stage0-posix)
    GCC=$(find_input gcc-4.0.4-musl-cxx-v7)
    BINUTILS=$(find_input binutils-2.30-gcc-pass4-v16)
    MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
    SRC=$(find_input make-3.82-src)

    WORK=/tmp/make-3.82-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN" "$WORK/src"
    for applet in cat chmod cp echo grep mkdir printf rm sed sh test; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
    $BB cp -r "$SRC/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"

    export PATH="$GCC/bin:$BINUTILS/bin:$TOOLBIN:$STAGE0/bin"
    CC="$GCC/bin/gcc"
    COMMON_CFLAGS='-DSTDC_HEADERS -DHAVE_STDLIB_H -DHAVE_STRING_H -DHAVE_STRINGS_H -DHAVE_LIMITS_H -DHAVE_UNISTD_H -DHAVE_FCNTL_H -DHAVE_SYS_WAIT_H -DHAVE_WAITPID -DHAVE_STRCASECMP -DHAVE_STRNCASECMP -DPOSIX'

    $BB sed -i \
      -e 's/&inf_jobs, &default_job_slots/0, 0/' \
      -e 's/&default_load_average/0/g' \
      -e 's/"basic", 0, "debug"/0, 0, "debug"/' \
      main.c
    : > config.h

    "$CC" $COMMON_CFLAGS -c getopt.c
    "$CC" $COMMON_CFLAGS -c getopt1.c
    "$CC" $COMMON_CFLAGS -c -I. -Iglob -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_STDINT_H ar.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_FCNTL_H arscan.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DFILE_TIMESTAMP_HI_RES=0 commands.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DSCCS_GET=\"/nullop\" default.c
    "$CC" $COMMON_CFLAGS -c -I. -Iglob -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_DIRENT_H dir.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART expand.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DFILE_TIMESTAMP_HI_RES=0 file.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -Dvfork=fork function.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART implicit.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_DUP2 -DHAVE_STRCHR -Dvfork=fork job.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DNO_FLOAT -DLOCALEDIR=\"/fake\" -DPACKAGE=\"make\" -DHAVE_MKTEMP -DHAVE_GETCWD main.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DHAVE_STRERROR -DHAVE_VPRINTF -DHAVE_ANSI_COMPILER -DHAVE_STDARG_H misc.c
    "$CC" $COMMON_CFLAGS -c -I. -Iglob -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DINCLUDEDIR=\"/include\" read.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART -DFILE_TIMESTAMP_HI_RES=0 -DHAVE_FCNTL_H -DLIBDIR=\"/lib\" remake.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART rule.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART signame.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART strcache.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART variable.c
    "$CC" $COMMON_CFLAGS -c -I. -DVERSION=\"3.82\" version.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART vpath.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART hash.c
    "$CC" $COMMON_CFLAGS -c -I. -DHAVE_INTTYPES_H -DHAVE_SA_RESTART remote-stub.c
    "$CC" $COMMON_CFLAGS -c -DHAVE_FCNTL_H getloadavg.c
    "$CC" $COMMON_CFLAGS -c -Iglob -o fnmatch.o glob/fnmatch.c
    "$CC" $COMMON_CFLAGS -c -Iglob -DHAVE_STRDUP -DHAVE_DIRENT_H -o glob.o glob/glob.c

    "$CC" -static -o make \
      getopt.o getopt1.o ar.o arscan.o commands.o default.o dir.o \
      expand.o file.o function.o implicit.o job.o main.o misc.o \
      read.o remake.o rule.o signame.o strcache.o variable.o \
      version.o vpath.o hash.o remote-stub.o getloadavg.o fnmatch.o glob.o
    $BB chmod 755 make

    ./make --version > "$WORK/version.txt"
    $BB grep -q 'GNU Make 3.82' "$WORK/version.txt"

    POSITIVE="$WORK/positive"
    $BB mkdir -p "$POSITIVE"
    $BB printf 'all:\n\t/bin/busybox echo make-positive > result\n' > "$POSITIVE/Makefile"
    ./make -C "$POSITIVE" all
    $BB grep -q '^make-positive$' "$POSITIVE/result"

    NEGATIVE="$WORK/negative"
    $BB mkdir -p "$NEGATIVE"
    $BB printf 'all:\n/bin/busybox echo rejected > result\n' > "$NEGATIVE/Makefile"
    if ./make -C "$NEGATIVE" all > "$WORK/negative.out" 2> "$WORK/negative.err"; then
      echo 'ERROR: make accepted malformed recipe indentation' >&2
      exit 1
    fi
    test ! -e "$NEGATIVE/result"
    test -s "$WORK/negative.err"

    $BB mkdir -p "$out/bin"
    $BB cp make "$out/bin/make"
    $BB chmod 755 "$out/bin/make"
    test -x "$out/bin/make"

    echo 'GNU Make 3.82 GCC runtime and rejection checks passed; not provider admission'
  "%],
  inputs = [stage0, gcc, binutils, musl, make_src],
} | crunch.Derivation
