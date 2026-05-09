# Build automake 1.8.5. Adapted from live-bootstrap.
let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let perl = import "perl-5.6.2-musl.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let autoconf = import "autoconf-2.59.ncl" in
let src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/automake-1.8.5/sources
  # first-consumer: bootstrap/automake-1.8.5.ncl
  url = "https://mirrors.kernel.org/gnu/automake/automake-1.8.5.tar.bz2",
  hash = "sha256-rDrGs5BMDfV5Oe5PfBsrlncUcQZfTaFPACrNP2vxoWo=",
  name = "automake-1.8.5-src",
} in
{
  name = "automake-1.8.5",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu; BB=/bin/busybox
    find_input() { for d in $NIX_STORE/*-"$1"; do if [ -d "$d" ]; then echo "$d"; return; fi; done; echo "ERROR: input $1 not found" >&2; exit 1; }
    MAKE=$(find_input make-3.82-tcc); SED=$(find_input sed-4.0.9-musl)
    PERL=$(find_input perl-5.6.2-musl); CU=$(find_input coreutils-6.10-musl)
    GAWK=$(find_input gawk-3.0.4-musl); GREP=$(find_input grep-2.4-musl)
    BASH=$(find_input bash-2.05b-tcc); AC=$(find_input autoconf-2.59)
    STAGE0=$(find_input stage0-posix); SRC=$(find_input automake-1.8.5-src)
    export PATH="$MAKE/bin:$SED/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$BASH/bin:$AC/bin:$STAGE0/bin:$PATH"
    export CONFIG_SHELL="$BASH/bin/bash"
    WORK=/tmp/am1.8.5; $BB mkdir -p "$WORK"; cd "$WORK"
    $BB cp -r "$SRC" src; $BB chmod -R u+w src; cd src
    ./configure --prefix="$out"
    make -j1 install prefix="$out"
    test -f "$out/bin/automake"; $BB chmod 755 "$out/bin/automake"
    test -f "$out/bin/automake-1.8"; $BB chmod 755 "$out/bin/automake-1.8"
    test -f "$out/bin/aclocal"; $BB chmod 755 "$out/bin/aclocal"
    test -f "$out/bin/aclocal-1.8"; $BB chmod 755 "$out/bin/aclocal-1.8"
    test -d "$out/share/automake-1.8"
  "%],
  inputs = [stage0, make, sed, perl, coreutils, gawk, grep, bash, autoconf, src],
} | crunch.Derivation
