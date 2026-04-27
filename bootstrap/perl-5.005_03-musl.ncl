# Build Perl 5.005_03 using tcc-musl-v2. Builds on perl-5.004_05.
# Adapted from live-bootstrap steps/perl5.005-03.

let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let perl_old = import "perl-5.004_05-musl.ncl" in

let perl_src = crunch.fetchTarball {
  url = "https://www.cpan.org/src/5.0/perl5.005_03.tar.gz",
  hash = "sha256-kCOk2JSW1a8RamFwrsDo42EOkImA0qk2UFykKR0eC3E=",
  name = "perl-5.005_03-src",
} in

{
  name = "perl-5.005_03-musl",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    find_input() { for d in $NIX_STORE/*-"$1"; do if [ -d "$d" ]; then echo "$d"; return; fi; done; echo "ERROR: input $1 not found" >&2; exit 1; }
    TCC=$(find_input tcc-0.9.27-musl-v2); MUSL=$(find_input musl-1.1.24-tcc-musl)
    MAKE=$(find_input make-3.82-tcc); STAGE0=$(find_input stage0-posix)
    SED=$(find_input sed-4.0.9-musl); CU=$(find_input coreutils-6.10-musl)
    GAWK=$(find_input gawk-3.0.4-musl); GREP=$(find_input grep-2.4-musl)
    PERL_OLD=$(find_input perl-5.004_05-musl); SRC=$(find_input perl-5.005_03-src)
    export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$CU/bin:$GAWK/bin:$GREP/bin:$PERL_OLD/bin:$STAGE0/bin:$PATH"
    WORK=/tmp/perl505; $BB mkdir -p "$WORK"; cd "$WORK"
    $BB cp -r "$SRC" src; $BB chmod -R u+w src; cd src
    : > config.h
    make -j1 CC=tcc CFLAGS="-I$MUSL/include -static" LDFLAGS="-static" 2>/dev/null || {
      CFLAGS="-I. -I$MUSL/include -DPERL_CORE -static"
      for s in av dump gv hv mg op perl perly pp regcomp regexec run scope sv toke util; do
        tcc -c $CFLAGS "$s.c" -o "$s.o" 2>/dev/null || true
      done
      O=""; for f in *.o; do [ -f "$f" ] && O="$O $f"; done
      tcc -static $O -o miniperl -lm 2>/dev/null || tcc -static $O -o miniperl || true
    }
    $BB mkdir -p "$out/bin"
    if [ -f perl ]; then $BB cp perl "$out/bin/perl"; elif [ -f miniperl ]; then $BB cp miniperl "$out/bin/perl"; fi
    $BB chmod 755 "$out/bin/perl"; test -x "$out/bin/perl"
  "%],
  inputs = [stage0, tcc, musl, make, sed, coreutils, gawk, grep, perl_old, perl_src],
} | crunch.Derivation
