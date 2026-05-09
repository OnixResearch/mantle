# Build autoconf 2.53. Adapted from live-bootstrap.
let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let perl = import "perl-5.6.2-musl.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let prev_am = import "automake-1.6.3.ncl" in
let src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/autoconf-2.53/sources
  # first-consumer: bootstrap/autoconf-2.53.ncl
  url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.53.tar.bz2",
  hash = "sha256-hwCnQbe2RrmoVeSWCV9And4+j+jhAx9IfvT1HsyQgWg=",
  name = "autoconf-2.53-src",
} in
{
  name = "autoconf-2.53",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu; BB=/bin/busybox
    find_input() { for d in $NIX_STORE/*-"$1"; do if [ -d "$d" ]; then echo "$d"; return; fi; done; echo "ERROR: input $1 not found" >&2; exit 1; }
    MAKE=$(find_input make-3.82-tcc); SED=$(find_input sed-4.0.9-musl)
    M4=$(find_input m4-1.4.7-musl); PERL=$(find_input perl-5.6.2-musl)
    CU=$(find_input coreutils-6.10-musl); GAWK=$(find_input gawk-3.0.4-musl)
    GREP=$(find_input grep-2.4-musl); DIFF=$(find_input diffutils-2.7-musl)
    BASH=$(find_input bash-2.05b-tcc); STAGE0=$(find_input stage0-posix)
    SRC=$(find_input autoconf-2.53-src)
    export PATH="$MAKE/bin:$SED/bin:$M4/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$STAGE0/bin:$PATH"
    export CONFIG_SHELL="$BASH/bin/bash"
    WORK=/tmp/ac2.53; $BB mkdir -p "$WORK"; cd "$WORK"
    $BB cp -r "$SRC" src; $BB chmod -R u+w src; cd src
    ./configure --prefix="$out"
    make -j1 install prefix="$out"
    for f in autoconf autoreconf autoheader autom4te; do
      test -f "$out/bin/$f" || { echo "ERROR: $f not installed" >&2; exit 1; }
      $BB chmod 755 "$out/bin/$f"
    done
    test -d "$out/share/autoconf" || { echo "ERROR: share/autoconf not installed" >&2; exit 1; }
  "%],
  inputs = [stage0, make, sed, m4, perl, coreutils, gawk, grep, diffutils, bash, prev_am, src],
} | crunch.Derivation
