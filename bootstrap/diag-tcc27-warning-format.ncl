# Diagnostic: reduce TinyCC 0.9.27/Mes warning-format corruption to a tiny C input.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tinycc.ncl" in

{
  name = "diag-tcc27-warning-format",
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

      TCC=$(find_input tinycc-0.9.27)
      STAGE0=$(find_input stage0-posix)
      export PATH="$TCC/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/tcc27-warning-format
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cat > warn.c <<'EOF'
      int main(void) {
        char *p;
        p = missing_warning_symbol(1, 2, 3);
        return p != 0;
      }
EOF

      set +e
      tcc -c warn.c > warn.stdout 2> warn.stderr
      rc=$?
      set -e

      $BB mkdir -p "$out/src" "$out/objects"
      $BB cp warn.c warn.stdout warn.stderr "$out/src/"
      echo "$rc" > "$out/src/warn.rc"
      if [ -f warn.o ]; then
        $BB chmod 644 warn.o
        $BB cp warn.o "$out/objects/"
      fi

      # This diagnostic intentionally captures the current bug: TinyCC/Mes
      # should name warn.c, line, and missing symbol, but currently emits
      # literal printf placeholders.  Fail closed if the reduced input stops
      # reproducing the corruption so follow-up repair evidence is explicit.
      needle=$($BB printf '\045s:\045d: warning')
      $BB grep -q "$needle" warn.stderr
      test "$rc" = 0
    "%,
  ],
  inputs = [stage0, tcc],
} | crunch.Derivation
