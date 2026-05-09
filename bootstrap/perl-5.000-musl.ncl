# Build Perl 5.000 linked against musl using tcc-musl-v2.
#
# First Perl in the chain. Provides minimal Perl for autotools.
# Adapted from live-bootstrap steps/perl-5.000.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in

let perl_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/perl-5.000/sources
  # first-consumer: bootstrap/perl-5.000-musl.ncl
  url = "https://github.com/Perl/perl5/archive/perl-5.000.tar.gz",
  hash = "sha256-s9LMsFfFf6Ws84cG/GW55znYlQBbn56U/C9TSiWczFE=",
  name = "perl-5.000-src",
} in

{
  name = "perl-5.000-musl",
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
      COREUTILS=$(find_input coreutils-6.10-musl)
      GAWK=$(find_input gawk-3.0.4-musl)
      GREP=$(find_input grep-2.4-musl)
      SRC=$(find_input perl-5.000-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$COREUTILS/bin:$GAWK/bin:$GREP/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/perl5000-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" perl-5.000
      $BB chmod -R u+w perl-5.000
      cd perl-5.000

      # Generate minimal config.h and config.sh
      $BB cat > config.h << 'CONFIG'
#define PERL_VERSION "5.000"
#define OSNAME "linux"
#define ARCHNAME "x86_64-linux"
#define BIN "$out/bin"
CONFIG

      $BB cat > config.sh << 'CONFIGSH'
osname='linux'
archname='x86_64-linux'
cc='tcc'
ccflags='-static'
ldflags='-static'
ar='tcc -ar'
ranlib='true'
prefix="$out"
bin="$out/bin"
installbin="$out/bin"
installman1dir="$out/man/man1"
installman3dir="$out/man/man3"
CONFIGSH
      sed -i "s|\$out|$out|g" config.sh

      # Attempt standard build; if the early Perl makefile cannot run in this
      # bootstrap environment, use the explicit miniperl source list below.
      if ! make -j1 CC=tcc CFLAGS="-I$MUSL/include -static" LDFLAGS="-static"; then
        CFLAGS="-I. -I$MUSL/include -DPERL_CORE -static"
        OBJS=""
        for src in av dump gv hv mg op perl perly pp regcomp regexec \
                   run scope sv toke util; do
          tcc -c $CFLAGS "$src.c" -o "$src.o"
          OBJS="$OBJS $src.o"
        done
        test -n "$OBJS" || { echo "ERROR: no Perl objects built" >&2; exit 1; }
        if ! tcc -static $OBJS -o miniperl -lm; then
          tcc -static $OBJS -o miniperl
        fi
      fi

      $BB mkdir -p "$out/bin"
      if [ -f perl ]; then
        $BB cp perl "$out/bin/perl"
      elif [ -f miniperl ]; then
        $BB cp miniperl "$out/bin/perl"
        $BB cp miniperl "$out/bin/miniperl"
      else
        echo "ERROR: neither perl nor miniperl was built" >&2
        exit 1
      fi
      $BB chmod 755 "$out/bin/perl"

      test -x "$out/bin/perl" || { echo "ERROR: installed perl missing" >&2; exit 1; }
      "$out/bin/perl" -e 'print "perl5000-ok\\n"' | $BB grep '^perl5000-ok$'
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, coreutils, gawk, grep, diffutils, perl_src],
} | crunch.Derivation
