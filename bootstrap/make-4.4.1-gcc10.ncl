# Source-built GNU Make for the full-source Rust provider route.
#
# This direct, configure-free build is compiled by the admitted normalized
# GCC/musl/binutils provider. Sandbox BusyBox is an explicit orchestration
# assumption outside the installed host-tool claim.

let crunch = import "lib.ncl" in
let native_provider = import "seed-full-admitted.ncl" in
# Upstream archive SHA-256: dd16fb1d67bfab79a72f5e8390735c49e3e8e70b4945a15ab1f81ddb78658fb3.
let make_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/make/make-4.4.1.tar.gz",
  hash = "sha256-+Cg7R8wougcWcFf6u5B3lVX6RRgMRYZ1CD+e21URP5A=",
  name = "make-4.4.1-full-source-src",
} in

{
  name = "make-4.4.1-full-source-gcc10-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      WORK=/tmp/make-4.4.1-full-source
      TARGET=x86_64-linux-musl

      find_input() {
        input_suffix=$1
        matched_input=
        match_count=0
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            matched_input=$input_path
            match_count=$((match_count + 1))
          fi
        done
        if [ "$match_count" -ne 1 ]; then
          echo "ERROR: expected exactly one declared input $input_suffix, got $match_count" >&2
          exit 1
        fi
        echo "$matched_input"
      }

      PROVIDER=$(find_input full-source-seed-toolchain)
      SRC=$(find_input make-4.4.1-full-source-src)
      test -x "$PROVIDER/bin/$TARGET-gcc"
      test -x "$PROVIDER/bin/ar"
      test -x "$PROVIDER/bin/ranlib"
      test -d "$SRC/src"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK/tools" "$WORK/obj"
      for applet in awk basename cat chmod cmp cp cut date dirname echo env expr find grep head hostname id install ln ls mkdir mv od paste printf pwd readlink realpath rm sed sh sleep sort tail tee test touch tr uname uniq wc xargs; do
        $BB ln -sf "$BB" "$WORK/tools/$applet"
      done
      export PATH="$WORK/tools"
      export CC="$PROVIDER/bin/$TARGET-gcc"
      export AR="$PROVIDER/bin/ar"
      export RANLIB="$PROVIDER/bin/ranlib"
      export CFLAGS="-static -O2 -D_GNU_SOURCE -include stddef.h"
      export LDFLAGS="-static"
      export SOURCE_DATE_EPOCH=1

      $BB cp -R "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"

      $BB cat > src/config.h <<'CFGEOF'
#define HAVE_STRING_H 1
#define HAVE_STDLIB_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_LOCALE_H 1
#define HAVE_FCNTL_H 1
#define HAVE_ALLOCA_H 1
#define HAVE_MEMORY_H 1
#define HAVE_DIRENT_H 1
#define HAVE_STDINT_H 1
#define HAVE_INTTYPES_H 1
#define HAVE_SIGNAL_H 1
#define HAVE_SYS_PARAM_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_TIME_H 1
#define HAVE_SYS_TYPES_H 1
#define HAVE_SYS_WAIT_H 1
#define HAVE_SYS_FILE_H 1
#define HAVE_SYS_RESOURCE_H 1
#define HAVE_SYS_SELECT_H 1
#define HAVE_STRINGS_H 1
#define HAVE_DUP 1
#define HAVE_DUP2 1
#define HAVE_GETCWD 1
#define HAVE_MKSTEMP 1
#define HAVE_MKTEMP 1
#define HAVE_STRERROR 1
#define HAVE_STRSIGNAL 1
#define HAVE_STRNDUP 1
#define HAVE_STRTOLL 1
#define HAVE_ISATTY 1
#define HAVE_TTYNAME 1
#define HAVE_FDOPEN 1
#define HAVE_FILENO 1
#define HAVE_SETLINEBUF 1
#define HAVE_SETVBUF 1
#define HAVE_STRNCASECMP 1
#define HAVE_UMASK 1
#define HAVE_MKFIFO 1
#define HAVE_REALPATH 1
#define HAVE_ATEXIT 1
#define HAVE_GETRLIMIT 1
#define HAVE_SETRLIMIT 1
#define HAVE_GETTIMEOFDAY 1
#define HAVE_PSELECT 1
#define HAVE_VFORK 1
#define HAVE_WAIT_NOHANG 1
#define HAVE_WAITPID 1
#define HAVE_MEMPCPY 1
#define HAVE_MEMRCHR 1
#define HAVE_SA_RESTART 1
#define HAVE_CLOCK_GETTIME 1
#define HAVE_STRUCT_DIRENT_D_TYPE 1
#define HAVE_POSIX_SPAWNATTR_SETSIGMASK 1
#define FILE_TIMESTAMP_HI_RES 1
#define ST_MTIM_NSEC st_mtim.tv_nsec
#define PACKAGE "make"
#define PACKAGE_VERSION "4.4.1"
#define VERSION "4.4.1"
#define LOCALEDIR "/dev/null"
#define INCLUDEDIR "/dev/null"
#define LIBDIR "/dev/null"
#define SCCS_GET "get"
#undef ENABLE_NLS
#undef HAVE_GUILE
#undef HAVE_DMALLOC_H
#ifndef GLOB_ALTDIRFUNC
#define GLOB_ALTDIRFUNC 0
#endif
CFGEOF

      $BB sed -i 's/gl->gl_opendir = open_dirstream;//' src/dir.c
      $BB sed -i 's/gl->gl_readdir = read_dirstream;//' src/dir.c
      $BB sed -i 's/gl->gl_closedir = free;//' src/dir.c
      $BB sed -i 's/gl->gl_lstat = local_lstat;//' src/dir.c
      $BB sed -i 's/gl->gl_stat = local_stat;//' src/dir.c

      SRCS="src/ar.c src/arscan.c src/commands.c src/default.c
            src/dir.c src/expand.c src/file.c src/function.c
            src/getopt.c src/getopt1.c src/hash.c src/implicit.c
            src/job.c src/load.c src/loadapi.c src/main.c
            src/misc.c src/output.c src/read.c src/remake.c
            src/remote-stub.c src/rule.c src/shuffle.c
            src/signame.c src/strcache.c src/variable.c
            src/version.c src/vpath.c src/posixos.c src/guile.c"
      for source_file in $SRCS; do
        object_name=$($BB basename "$source_file" .c)
        "$CC" $CFLAGS -I src -I lib -I . -DHAVE_CONFIG_H -c \
          -o "$WORK/obj/$object_name.o" "$source_file"
      done
      $BB cat > "$WORK/obj/lib_stubs.c" <<'STUBEOF'
#include <stdlib.h>
char *concatenated_filename(const char *d, const char *f, const char *s) { return NULL; }
const char *find_in_given_path(const char *n, const char *p, const char *d, int o) { return NULL; }
STUBEOF
      "$CC" $CFLAGS -c -o "$WORK/obj/lib_stubs.o" "$WORK/obj/lib_stubs.c"
      "$CC" $LDFLAGS -o "$WORK/make" "$WORK"/obj/*.o

      $BB mkdir -p "$out/bin"
      $BB cp "$WORK/make" "$out/bin/make"
      $BB chmod 755 "$out/bin/make"
      "$out/bin/make" --version | "$BB" grep -q 'GNU Make 4.4.1'

      $BB mkdir -p "$WORK/positive"
      $BB cat > "$WORK/positive/Makefile" <<'EOF'
all:
	@printf '%s\n' mantle-source-make-ok > result.txt
EOF
      "$out/bin/make" -C "$WORK/positive" all
      "$BB" grep -q mantle-source-make-ok "$WORK/positive/result.txt"

      $BB mkdir -p "$WORK/rejected"
      $BB printf '%s\n' 'all:' ' this recipe lacks the required tab' > "$WORK/rejected/Makefile"
      if "$out/bin/make" -C "$WORK/rejected" all > "$WORK/rejected.stdout" 2> "$WORK/rejected.stderr"; then
        echo "ERROR: source-built GNU Make accepted malformed Makefile" >&2
        exit 1
      fi
      test ! -e "$WORK/rejected/all"

      RELOCATED_MAKE="$WORK/relocated-make"
      $BB cp -R "$out" "$RELOCATED_MAKE"
      "$RELOCATED_MAKE/bin/make" --version | "$BB" grep -q 'GNU Make 4.4.1'

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/full-source-rust-host-make.txt" <<'EOF'
claim: bounded source-built GNU Make host-tool matrix passed
positive: version, recipe execution, copied-tree version
negative: malformed Makefile rejected
source_authority: authenticated GNU Make 4.4.1 source
native_authority: admitted full-source GCC/musl/binutils provider
execution_policy: declared source and native-provider inputs only
evidence_revision: 3
non_claims: GNU Make correctness, Rust provider completion, whole-bootstrap correctness
EOF
      echo "source-built GNU Make 4.4.1 host-tool matrix passed"
    "%
  ],
  inputs = [native_provider, make_src],
} | crunch.Derivation
