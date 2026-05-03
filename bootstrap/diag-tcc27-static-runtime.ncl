# Diagnostic: exercise TinyCC 0.9.27/Mes static executable runtime behavior.
#
# This is intentionally a focused reproducer for the Make 3.82 startup/runtime
# segfault class. It builds a nontrivial static executable with the same
# bootstrapped TinyCC used by bootstrap/make-tcc.ncl, then runs it.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tinycc.ncl" in

{
  name = "diag-tcc27-static-runtime",
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

      WORK=/tmp/tcc27-static-runtime
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cat > helper.c <<'EOF'
      typedef unsigned long size_t;
      extern int strcmp(const char *, const char *);
      extern int sprintf(char *, const char *, ...);
      extern int puts(const char *);
      extern void *memset(void *, int, size_t);

      static int global_counter = 41;
      static const char *global_words[] = { "alpha", "beta", "gamma", 0 };

      int helper_value(const char *needle) {
        int i;
        int score = 0;
        char buf[64];
        memset(buf, 0, sizeof(buf));
        for (i = 0; global_words[i]; i++) {
          if (!strcmp(global_words[i], needle))
            score += i + 1;
        }
        sprintf(buf, "score=%d", score + global_counter);
        if (strcmp(buf, "score=43"))
          return 10;
        puts(buf);
        return 0;
      }
      EOF

      $BB cat > main.c <<'EOF'
      extern int helper_value(const char *);
      extern int printf(const char *, ...);
      extern int puts(const char *);

      static int add3(int x) { return x + 3; }
      static int (*global_fp)(int) = add3;
      static const char banner[] = "diag-tcc27-static-runtime";

      int main(int argc, char **argv) {
        int v;
        (void)argc;
        (void)argv;
        puts(banner);
        v = global_fp(39);
        if (v != 42)
          return 20;
        if (helper_value("beta"))
          return 21;
        printf("value=%d\n", v);
        return 0;
      }
      EOF

      echo "diag: compile helper" >&2
      tcc -c helper.c
      echo "diag: compile main" >&2
      tcc -c main.c
      $BB chmod 644 helper.o main.o
      echo "diag: link static executable" >&2
      tcc -L"$TCC/lib/mes" -L"$TCC/lib/mes/tcc" -static -o diag main.o helper.o
      $BB chmod 755 diag
      echo "diag: execute static executable" >&2
      ./diag > diag.out
      $BB grep -q 'diag-tcc27-static-runtime' diag.out
      $BB grep -q 'score=43' diag.out
      $BB grep -q 'value=42' diag.out

      $BB mkdir -p "$out/bin" "$out/share"
      $BB cp diag "$out/bin/diag-tcc27-static-runtime"
      $BB cp diag.out "$out/share/diag.out"
      test -x "$out/bin/diag-tcc27-static-runtime"
    "%,
  ],
  inputs = [stage0, tcc],
} | crunch.Derivation
