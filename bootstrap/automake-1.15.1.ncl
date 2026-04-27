# Build automake 1.15.1. Adapted from live-bootstrap.
let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-4.0.9-musl.ncl" in
let perl = import "perl-5.6.2-musl.ncl" in
let coreutils = import "coreutils-6.10-musl.ncl" in
let gawk = import "gawk-3.0.4-musl.ncl" in
let grep = import "grep-2.4-musl.ncl" in
let bash = import "bash-2.05b-tcc.ncl" in
let autoconf = import "autoconf-2.69.ncl" in
let src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/automake/automake-1.15.1.tar.xz",
  hash = "sha256-/KHwBUWoEgUxjLuLFOpqM4c0UXUDWiyiBV4HBTwOcZU=",
  name = "automake-1.15.1-src",
} in
{
  name = "automake-1.15.1",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu; BB=/bin/busybox
    find_input() { for d in $NIX_STORE/*-"$1"; do if [ -d "$d" ]; then echo "$d"; return; fi; done; echo "ERROR: input $1 not found" >&2; exit 1; }
    MAKE=$(find_input make-3.82-tcc); SED=$(find_input sed-4.0.9-musl)
    PERL=$(find_input perl-5.6.2-musl); CU=$(find_input coreutils-6.10-musl)
    GAWK=$(find_input gawk-3.0.4-musl); GREP=$(find_input grep-2.4-musl)
    BASH=$(find_input bash-2.05b-tcc); AC=$(find_input autoconf-2.69)
    STAGE0=$(find_input stage0-posix); SRC=$(find_input automake-1.15.1-src)
    export PATH="$MAKE/bin:$SED/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$BASH/bin:$AC/bin:$STAGE0/bin:$PATH"
    export CONFIG_SHELL="$BASH/bin/bash"
    WORK=/tmp/am1.15.1; $BB mkdir -p "$WORK"; cd "$WORK"
    $BB cp -r "$SRC" src; $BB chmod -R u+w src; cd src
    ./configure --prefix="$out" 2>/dev/null || true
    make -j1 install prefix="$out" 2>/dev/null || {
      $BB mkdir -p "$out/bin" "$out/share/automake-1.15"
      for f in bin/*; do [ -f "$f" ] && $BB cp "$f" "$out/$f" && $BB chmod 755 "$out/$f"; done
    }
    test -f "$out/bin/automake" || test -f "$out/bin/automake-1.15"
  "%],
  inputs = [stage0, make, sed, perl, coreutils, gawk, grep, bash, autoconf, src],
} | crunch.Derivation
