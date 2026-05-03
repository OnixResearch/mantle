# Diagnostic: inspect TinyCC 0.9.27/Mes static runtime formatting/string behavior.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tinycc.ncl" in

{
  name = "diag-tcc27-runtime-format-inspect",
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

      WORK=/tmp/tcc27-runtime-format-inspect
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cat > helper.c <<'EOF'
      typedef unsigned long size_t;
      extern int strcmp(const char *, const char *);
      extern int sprintf(char *, const char *, ...);
      extern int printf(const char *, ...);
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
          int cmp = strcmp(global_words[i], needle);
          printf("loop i=%d word=%s cmp=%d\n", i, global_words[i], cmp);
          if (!cmp)
            score += i + 1;
        }
        printf("score_raw=%d total=%d\n", score, score + global_counter);
        sprintf(buf, "score=%d", score + global_counter);
        printf("buf=[%s] cmp=%d\n", buf, strcmp(buf, "score=43"));
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
      static const char banner[] = "diag-tcc27-runtime-format-inspect";

      int main(int argc, char **argv) {
        int v;
        (void)argc;
        (void)argv;
        puts(banner);
        v = global_fp(39);
        printf("v=%d\n", v);
        return helper_value("beta");
      }
      EOF

      echo "format-inspect: compile helper" >&2
      tcc -c helper.c
      echo "format-inspect: compile main" >&2
      tcc -c main.c
      $BB chmod 644 helper.o main.o
      echo "format-inspect: link static executable" >&2
      tcc -L"$TCC/lib/mes" -L"$TCC/lib/mes/tcc" -static -o diag main.o helper.o
      $BB chmod 755 diag
      set +e
      ./diag > probe.stdout 2> probe.stderr
      rc=$?
      set -e
      $BB mkdir -p "$out/bin" "$out/objects" "$out/src"
      $BB cp diag "$out/bin/diag-tcc27-runtime-format-inspect"
      $BB cp helper.o main.o "$out/objects/"
      $BB cp helper.c main.c probe.stdout probe.stderr "$out/src/"
      echo "$rc" > "$out/src/probe.rc"
      test -x "$out/bin/diag-tcc27-runtime-format-inspect"
    "%,
  ],
  inputs = [stage0, tcc],
} | crunch.Derivation
