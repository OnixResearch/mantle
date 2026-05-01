let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let tcc = import "tinycc.ncl" in
{
  name = "diag-tcc-link-matrix",
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
      $BB mkdir -p $out
      $BB cat > hello.c <<'EOF'
int main(void) { return 0; }
EOF
      echo 'matrix: tcc version' >&2
      tcc -v || true
      echo 'matrix: compile hello.c' >&2
      tcc -c hello.c
      $BB chmod 644 hello.o

      run_case() {
        name=$1
        shift
        echo "matrix: BEGIN $name" >&2
        set +e
        "$@"
        code=$?
        set -e
        echo "matrix: END $name status=$code" >&2
        return 0
      }

      run_case implicit tcc -static -o hello-implicit hello.o
      run_case nostdlib-full tcc -nostdlib -static -o hello-full "$TCC/lib/mes/crt1.o" hello.o "$TCC/lib/mes/crti.o" "$TCC/lib/mes/crtn.o" "$TCC/lib/mes/tcc/libtcc1.a" "$TCC/lib/mes/libc.a"
      run_case nostdlib-no-crti tcc -nostdlib -static -o hello-no-crti "$TCC/lib/mes/crt1.o" hello.o "$TCC/lib/mes/tcc/libtcc1.a" "$TCC/lib/mes/libc.a"
      run_case nostdlib-libc-first tcc -nostdlib -static -o hello-libc-first "$TCC/lib/mes/crt1.o" hello.o "$TCC/lib/mes/libc.a" "$TCC/lib/mes/tcc/libtcc1.a"
      run_case nostdlib-libc-only tcc -nostdlib -static -o hello-libc-only "$TCC/lib/mes/crt1.o" hello.o "$TCC/lib/mes/libc.a"
      run_case no-archives tcc -nostdlib -static -o hello-no-archives "$TCC/lib/mes/crt1.o" hello.o

      $BB touch $out/done
    "%
  ],
  inputs = [stage0, tcc],
}
