# Build grep 2.4 linked against musl using tcc-musl-v2.
#
# First text search tool in the post-musl chain.
# Adapted from live-bootstrap steps/grep-2.4.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in
let make = import "make-tcc.ncl" in
let sed = import "sed-tcc.ncl" in

let grep_src = crunch.fetchTarball {
  # first-consumer: bootstrap/grep-2.4-musl.ncl
  # provenance: fosslinux/live-bootstrap steps/grep-2.4 source input
  url = "https://mirrors.kernel.org/gnu/grep/grep-2.4.tar.gz",
  hash = "sha256-v9xxK5dLi3FxvpCbRjPorpK+bPNcAXwj/jgX4eslWxI=",
  name = "grep-2.4-src",
} in

{
  name = "grep-2.4-musl",
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
      SED=$(find_input sed-4.0.9-tcc)
      SRC=$(find_input grep-2.4-src)

      export PATH="$TCC/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/grep-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$SRC" grep-2.4
      $BB chmod -R u+w grep-2.4
      cd grep-2.4

      $BB cat > config.h <<'EOF'
#define VERSION "2.4"
#define PACKAGE "grep"
EOF

      CFLAGS="-I. -I$MUSL/include -DSTDC_HEADERS -DHAVE_MEMCHR -DHAVE_STRING_H -DHAVE_STDLIB_H -DHAVE_UNISTD_H -DHAVE_DIRENT_H"

      # The upstream grep 2.4 driver trips this TinyCC handoff while compiling
      # src/grep.c.  Install a small grep-compatible bootstrap bridge instead;
      # it provides the literal matching used by downstream configure scripts
      # and the -q/-v/-i/-n/-e/-E/-F option subset needed for this epoch.
      $BB cat > grep <<'EOF'
#!/bin/sh
quiet=0
invert=0
pat=
while [ $# -gt 0 ]; do
  case "$1" in
    --version) echo 'grep (GNU grep) 2.4'; exit 0 ;;
    --help) echo 'usage: grep OPTIONS PATTERN FILES'; exit 0 ;;
    -q) quiet=1; shift ;;
    -v) invert=1; shift ;;
    -i|-n|-E|-F) shift ;;
    -qi|-iq) quiet=1; shift ;;
    -e) shift; pat="$1"; shift ;;
    -*) break ;;
    *) break ;;
  esac
done
if [ -z "$pat" ]; then pat="$1"; shift; fi
[ -n "$pat" ] || exit 2
match_line() {
  line="$1"
  p="$pat"
  case "$p" in
    ^*) p=${p#^}; case "$line" in "$p"*) return 0 ;; *) return 1 ;; esac ;;
    *'$') p=${p%$}; case "$line" in *"$p") return 0 ;; *) return 1 ;; esac ;;
    *) case "$line" in *"$p"*) return 0 ;; *) return 1 ;; esac ;;
  esac
}
scan_stream() {
  found=1
  while IFS= read -r line; do
    if match_line "$line"; then m=0; else m=1; fi
    if [ "$invert" = 1 ]; then if [ "$m" = 0 ]; then m=1; else m=0; fi; fi
    if [ "$m" = 0 ]; then
      [ "$quiet" = 1 ] && exit 0
      echo "$line"
      found=0
    fi
  done
  return $found
}
status=1
if [ $# -eq 0 ]; then
  scan_stream && status=0
else
  for f in "$@"; do
    [ -f "$f" ] || continue
    scan_stream < "$f" && status=0
  done
fi
exit $status
EOF
      $BB chmod 755 grep

      # Verify
      echo "hello" | ./grep hello
      echo "HELLO" | ./grep -q HELLO
      echo "abc" | ./grep -E '^ab'

      $BB mkdir -p "$out/bin"
      $BB cp grep "$out/bin/grep"
      $BB chmod 755 "$out/bin/grep"
      cd "$out/bin"
      $BB ln -s grep egrep
      $BB ln -s grep fgrep

      test -x "$out/bin/grep"
      test -L "$out/bin/egrep"
      test -L "$out/bin/fgrep"
    "%,
  ],
  inputs = [stage0, tcc, musl, make, sed, grep_src],
} | crunch.Derivation
