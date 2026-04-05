# Build GNU Make from source using the fetched musl-gcc toolchain.
#
# Part of the Nix-free bootstrap chain:
#   musl-gcc (fetched) → make (this file) → bash → coreutils → ...
#
# Usage:
#   crunch build bootstrap/make.ncl -I lib --store /tmp/crunch-store
#
# GNU Make can bootstrap itself without a pre-existing make — its
# configure script generates a shell-based build.sh that compiles
# make using only a C compiler and sh.

let crunch = import "lib.ncl" in

let toolchain = crunch.fetchTarball {
  url = "https://musl.cc/x86_64-linux-musl-native.tgz",
  hash = "sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=",
  name = "musl-gcc",
} in

let make_src = crunch.fetchTarball {
  url = "https://ftp.gnu.org/gnu/make/make-4.4.1.tar.gz",
  hash = "sha256-+Cg7R8wougcWcFf6u5B3lVX6RRgMRYZ1CD+e21URP5A=",
  name = "make-src",
} in

{
  name = "gnumake",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink sleep touch date od \
                 uname id hostname diff cmp true false tee; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done
      export PATH="/tmp/tools"

      # Find the musl-gcc toolchain.
      TOOLCHAIN=""
      for d in /nix/store/*-musl-gcc; do
        if [ -d "$d" ]; then TOOLCHAIN="$d"; break; fi
      done
      if [ -z "$TOOLCHAIN" ]; then
        echo "ERROR: musl-gcc not found" >&2; exit 1
      fi

      # Find make source.
      SRCDIR=""
      for d in /nix/store/*-make-src; do
        if [ -d "$d" ]; then SRCDIR="$d"; break; fi
      done
      if [ -z "$SRCDIR" ]; then
        echo "ERROR: make-src not found" >&2; exit 1
      fi

      export CC="$TOOLCHAIN/bin/x86_64-linux-musl-gcc"
      export AR="$TOOLCHAIN/bin/x86_64-linux-musl-ar"
      export RANLIB="$TOOLCHAIN/bin/x86_64-linux-musl-ranlib"
      export CFLAGS="-static -O2"
      export LDFLAGS="-static"

      # Work in /tmp. The source tree is read-only (mounted by the
      # sandbox). Copy without -a to avoid ownership issues in the
      # sandbox (no chown), then fix permissions.
      $BB cp -r "$SRCDIR" /tmp/make-build 2>/dev/null
      $BB chmod -R u+w /tmp/make-build
      cd /tmp/make-build

      # GNU Make ships a Basic.mk that bootstraps without autoconf.
      # This avoids the need for a full grep/sed/awk that autoconf
      # demands (busybox versions are too limited for configure).
      #
      # Basic.mk reads config from src/mkconfig.h and src/mkcustom.h.
      # We create a minimal config.h and point it at our toolchain.

      # Create a minimal config.h with just enough to build.
      cat > src/config.h << 'CFGEOF'
#define HAVE_STRING_H 1
#define HAVE_STDLIB_H 1
#define HAVE_UNISTD_H 1
#define HAVE_LIMITS_H 1
#define HAVE_SYS_PARAM_H 1
#define HAVE_SYS_STAT_H 1
#define HAVE_SYS_WAIT_H 1
#define HAVE_FCNTL_H 1
#define HAVE_DIRENT_H 1
#define HAVE_STDINT_H 1
#define HAVE_INTTYPES_H 1
#define HAVE_LOCALE_H 1
#define HAVE_SIGNAL_H 1
#define HAVE_SA_RESTART 1
#define HAVE_DUP2 1
#define HAVE_GETCWD 1
#define HAVE_MKSTEMP 1
#define HAVE_STRERROR 1
#define HAVE_STRSIGNAL 1
#define HAVE_ISATTY 1
#define HAVE_FDOPEN 1
#define HAVE_FILENO 1
#define HAVE_TTYNAME 1
#define HAVE_SETLINEBUF 1
#define HAVE_STRNCASECMP 1
/* musl provides mode_t via sys/types.h */
#define HAVE_VFORK 1
#define FILE_TIMESTAMP_HI_RES 1
#define SCCS_GET "get"
#define PACKAGE "make"
#define PACKAGE_VERSION "4.4.1"
#define VERSION "4.4.1"
#define LOCALEDIR "/tmp/locale"
#define INCLUDEDIR "/tmp/include"
#define LIBDIR "/tmp/lib"
CFGEOF

      # Compile all source files. Compile each to a .o in /tmp/obj/.
      mkdir -p /tmp/obj

      SRCS="src/ar.c src/arscan.c src/commands.c src/default.c
            src/dir.c src/expand.c src/file.c src/function.c
            src/getopt.c src/getopt1.c src/hash.c src/implicit.c
            src/job.c src/load.c src/loadapi.c src/main.c
            src/misc.c src/output.c src/read.c src/remake.c
            src/remote-stub.c src/rule.c src/shuffle.c
            src/signame.c src/strcache.c src/variable.c
            src/version.c src/vpath.c src/posixos.c"

      OBJS=""
      for f in $SRCS; do
        base=$(basename $f .c)
        echo "CC $f"
        $CC $CFLAGS -I src -I lib -I . -DHAVE_CONFIG_H -c -o /tmp/obj/$base.o $f || exit 1
        OBJS="$OBJS /tmp/obj/$base.o"
      done

      # lib/ provides portability shims. musl has glob/fnmatch natively,
      # but we still need concat-filename and findprog-in.
      for f in lib/concat-filename.c lib/findprog-in.c lib/fnmatch.c lib/glob.c; do
        base=$(basename $f .c)
        echo "CC $f"
        $CC $CFLAGS -I src -I lib -I . -DHAVE_CONFIG_H -c -o /tmp/obj/lib_$base.o $f || exit 1
        OBJS="$OBJS /tmp/obj/lib_$base.o"
      done

      echo "LINK make"
      $CC $LDFLAGS -o /tmp/make $OBJS

      # Install.
      mkdir -p $out/bin
      cp /tmp/make $out/bin/make
      chmod 755 $out/bin/make

      # Verify.
      $out/bin/make --version
    "%,
  ],
  inputs = [toolchain, make_src],
} | crunch.Derivation
