# Build libtool 2.2.4. Adapted from live-bootstrap steps/libtool-2.2.4.
let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let m4 = import "m4-1.4.7-musl.ncl" in
let perl = import "perl-5.6.2-musl.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let diffutils = import "diffutils-2.7-musl.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let autoconf = import "autoconf-2.69.ncl" in
let automake = import "automake-1.15.1.ncl" in

let src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/libtool-2.2.4/sources
  # first-consumer: bootstrap/libtool-2.2.4.ncl
  url = "https://mirrors.kernel.org/gnu/libtool/libtool-2.2.4.tar.bz2",
  hash = "sha256-v2lZrGlw6Mvrv9Fld/G8u3753WBU7Vupwbc2EeRjzwo=",
  name = "libtool-2.2.4-src",
} in

{
  name = "libtool-2.2.4",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu; BB=/bin/busybox
    find_input() { for d in $NIX_STORE/*-"$1"; do if [ -d "$d" ]; then echo "$d"; return; fi; done; echo "ERROR: input $1 not found" >&2; exit 1; }
    TCC=$(find_input tcc-0.9.27-musl-v2); MUSL=$(find_input musl-1.1.24-tcc-musl)
    MAKE=$(find_input make-3.82-tcc); SED=$(find_input sed-4.0.9-musl)
    M4=$(find_input m4-1.4.7-musl); PERL=$(find_input perl-5.6.2-musl)
    CU=$(find_input coreutils-6.10-musl); GAWK=$(find_input gawk-3.0.4-musl)
    GREP=$(find_input grep-2.4-musl); DIFF=$(find_input diffutils-2.7-musl)
    BASH=$(find_input bash-2.05b-tcc); AC=$(find_input autoconf-2.69)
    AM=$(find_input automake-1.15.1); STAGE0=$(find_input stage0-posix)
    SRC=$(find_input libtool-2.2.4-src)
    export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$AC/bin:$AM/bin:$STAGE0/bin:$PATH"
    export CONFIG_SHELL="$BASH/bin/bash"
    WORK=/tmp/libtool; $BB mkdir -p "$WORK"; cd "$WORK"
    $BB cp -r "$SRC" src; $BB chmod -R u+w src; cd src
    ./configure --prefix="$out" CC=tcc CFLAGS="-I$MUSL/include -static"
    make -j1 install prefix="$out"
    test -x "$out/bin/libtool" || { echo "ERROR: installed libtool missing" >&2; exit 1; }
    test -x "$out/bin/libtoolize" || { echo "ERROR: installed libtoolize missing" >&2; exit 1; }
    test -d "$out/share/libtool" || { echo "ERROR: share/libtool missing" >&2; exit 1; }
    "$out/bin/libtool" --version >/dev/null
    "$out/bin/libtoolize" --version >/dev/null
  "%],
  inputs = [stage0, tcc, musl, make, sed, m4, perl, coreutils, gawk, grep, diffutils, bash, autoconf, automake, src],
} | crunch.Derivation
