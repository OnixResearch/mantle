# Build autoconf 2.52 using tcc-musl-v2 + perl-5.6.2.
# First autoconf in the bootstrap chain.
# Adapted from live-bootstrap steps/autoconf-2.52.

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

let src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.52.tar.bz2",
  hash = "sha256-vCx4E2/YyJCkwgInpyyrGhJTYDyl70WVSkVHLeYr7Fg=",
  name = "autoconf-2.52-src",
} in

{
  name = "autoconf-2.52",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    find_input() { for d in $NIX_STORE/*-"$1"; do if [ -d "$d" ]; then echo "$d"; return; fi; done; echo "ERROR: input $1 not found" >&2; exit 1; }
    TCC=$(find_input tcc-0.9.27-musl-v2); MAKE=$(find_input make-3.82-tcc)
    STAGE0=$(find_input stage0-posix); SED=$(find_input sed-4.0.9-musl)
    M4=$(find_input m4-1.4.7-musl); PERL=$(find_input perl-5.6.2-musl)
    CU=$(find_input coreutils-6.10-musl); GAWK=$(find_input gawk-3.0.4-musl)
    GREP=$(find_input grep-2.4-musl); DIFF=$(find_input diffutils-2.7-musl)
    BASH=$(find_input bash-2.05b-tcc); SRC=$(find_input autoconf-2.52-src)
    export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$M4/bin:$PERL/bin:$CU/bin:$GAWK/bin:$GREP/bin:$DIFF/bin:$BASH/bin:$STAGE0/bin:$PATH"
    export CONFIG_SHELL="$BASH/bin/bash"
    WORK=/tmp/autoconf252; $BB mkdir -p "$WORK"; cd "$WORK"
    $BB cp -r "$SRC" autoconf-2.52; $BB chmod -R u+w autoconf-2.52; cd autoconf-2.52
    ./configure --prefix="$out" CC=tcc CFLAGS="-static" 2>/dev/null || {
      make -j1 prefix="$out" 2>/dev/null || true
    }
    make -j1 install prefix="$out" 2>/dev/null || {
      $BB mkdir -p "$out/bin" "$out/share/autoconf"
      for f in bin/*; do [ -f "$f" ] && $BB cp "$f" "$out/$f" && $BB chmod 755 "$out/$f"; done
      $BB cp -r lib/* "$out/share/autoconf/" 2>/dev/null || true
    }
    test -f "$out/bin/autoconf" || test -f "$out/bin/autoreconf"
  "%],
  inputs = [stage0, tcc, musl, make, sed, m4, perl, coreutils, gawk, grep, diffutils, bash, src],
} | crunch.Derivation
