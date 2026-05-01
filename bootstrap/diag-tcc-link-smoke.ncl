let crunch = import "lib.ncl" in
let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc = import "tinycc.ncl" in
{
  name = "diag-tcc-link-smoke",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eux
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

      $BB mkdir -p $out/bin
      $BB cat > hello.c <<'EOF'
int main(void) { return 0; }
EOF
      echo 'diag: compile hello.c' >&2
      tcc -c hello.c
      $BB chmod 644 hello.o
      echo 'diag: link hello explicitly' >&2
      tcc -nostdlib -static -o hello "$TCC/lib/mes/crt1.o" hello.o "$TCC/lib/mes/crti.o" "$TCC/lib/mes/crtn.o" "$TCC/lib/mes/tcc/libtcc1.a" "$TCC/lib/mes/libc.a"
      $BB chmod 755 hello
      echo 'diag: run hello' >&2
      ./hello
      $BB cp hello $out/bin/hello
    "%
  ],
  inputs = [stage0, mes, tcc],
}
