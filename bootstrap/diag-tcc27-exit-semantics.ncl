# Diagnostic: TinyCC 0.9.27/Mes process termination semantics.

let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let tcc = import "tinycc.ncl" in

{
  name = "diag-tcc27-exit-semantics",
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
      WORK=/tmp/tcc27-exit-semantics
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK" "$out/src" "$out/bin"
      cd "$WORK"

      $BB cat > return0.c <<'EOF'
      int main(void) { return 0; }
      EOF
      $BB cat > return7.c <<'EOF'
      int main(void) { return 7; }
      EOF
      $BB cat > exit0.c <<'EOF'
      extern void exit(int);
      int main(void) { exit(0); return 99; }
      EOF
      $BB cat > exit7.c <<'EOF'
      extern void exit(int);
      int main(void) { exit(7); return 99; }
      EOF
      $BB cat > underscore_exit0.c <<'EOF'
      extern void _exit(int);
      int main(void) { _exit(0); return 99; }
      EOF
      $BB cat > underscore_exit7.c <<'EOF'
      extern void _exit(int);
      int main(void) { _exit(7); return 99; }
      EOF

      for name in return0 return7 exit0 exit7 underscore_exit0 underscore_exit7; do
        tcc -static -o "$name" "$name.c"
        $BB chmod 755 "$name"
        set +e
        ./"$name" > "$out/src/$name.stdout" 2> "$out/src/$name.stderr"
        rc=$?
        set -e
        printf '%s\n' "$rc" > "$out/src/$name.rc"
        $BB cp "$name" "$out/bin/$name"
        echo "$name.rc=$rc" >&2
      done

      test "$($BB cat "$out/src/return0.rc")" = 0
      test "$($BB cat "$out/src/return7.rc")" = 7
      test "$($BB cat "$out/src/exit0.rc")" = 0
      test "$($BB cat "$out/src/exit7.rc")" = 7
      test "$($BB cat "$out/src/underscore_exit0.rc")" = 0
      test "$($BB cat "$out/src/underscore_exit7.rc")" = 7
    "%,
  ],
  inputs = [stage0, tcc],
} | crunch.Derivation
