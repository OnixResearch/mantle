# Build Perl 5.003 using tcc-musl-v2. Builds on perl-5.000.
# Adapted from live-bootstrap steps/perl-5.003.

let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let perl_old = import "perl-5.000-musl.ncl" in

let perl_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/perl-5.003/sources
  # first-consumer: bootstrap/perl-5.003-musl.ncl
  url = "https://github.com/Perl/perl5/archive/perl-5.003.tar.gz",
  hash = "sha256-7oq3d0t9/SD6tV+jdCZtfRqaCkRPITY5QcMbUxdn2cE=",
  name = "perl-5.003-src",
} in

{
  name = "perl-5.003-musl",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    find_input() { for d in $NIX_STORE/*-"$1"; do if [ -d "$d" ]; then echo "$d"; return; fi; done; echo "ERROR: input $1 not found" >&2; exit 1; }
    TCC=$(find_input tcc-0.9.27-musl-v2); MUSL=$(find_input musl-1.1.24-tcc-musl)
    MAKE=$(find_input make-3.82-tcc); STAGE0=$(find_input stage0-posix)
    SED=$(find_input sed-4.0.9-musl); CU=$(find_input coreutils-6.10-musl)
    GAWK=$(find_input gawk-3.0.4-musl); GREP=$(find_input grep-2.4-musl)
    PERL_OLD=$(find_input perl-5.000-musl); SRC=$(find_input perl-5.003-src)
    export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$CU/bin:$GAWK/bin:$GREP/bin:$PERL_OLD/bin:$STAGE0/bin:$PATH"
    WORK=/tmp/perl5003; $BB mkdir -p "$WORK"; cd "$WORK"
    $BB cp -r "$SRC" perl-5.003; $BB chmod -R u+w perl-5.003; cd perl-5.003
    $BB cat > config.sh << CONF
osname='linux'; archname='x86_64-linux'; cc='tcc'; ccflags='-I$MUSL/include -static'
ldflags='-static'; ar='tcc -ar'; ranlib='true'; prefix=$out; bin=$out/bin
installbin=$out/bin; installman1dir=$out/man/man1; installman3dir=$out/man/man3
CONF
    : > config.h
    if ! make -j1 CC=tcc CFLAGS="-I$MUSL/include -static" LDFLAGS="-static"; then
      CFLAGS="-I. -I$MUSL/include -DPERL_CORE -static"
      O=""
      for s in av dump gv hv mg op perl perly pp regcomp regexec run scope sv toke util; do
        tcc -c $CFLAGS "$s.c" -o "$s.o"
        O="$O $s.o"
      done
      test -n "$O" || { echo "ERROR: no Perl objects built" >&2; exit 1; }
      if ! tcc -static $O -o miniperl -lm; then
        tcc -static $O -o miniperl
      fi
    fi
    $BB mkdir -p "$out/bin"
    if [ -f perl ]; then
      $BB cp perl "$out/bin/perl"
    elif [ -f miniperl ]; then
      $BB cp miniperl "$out/bin/perl"
    else
      echo "ERROR: neither perl nor miniperl was built" >&2; exit 1
    fi
    $BB chmod 755 "$out/bin/perl"
    test -x "$out/bin/perl" || { echo "ERROR: installed perl missing" >&2; exit 1; }
    "$out/bin/perl" -e 'print "perl5003-ok\n"' | $BB grep '^perl5003-ok$'
  "%],
  inputs = [stage0, tcc, musl, make, sed, coreutils, gawk, grep, perl_old, perl_src],
} | crunch.Derivation
