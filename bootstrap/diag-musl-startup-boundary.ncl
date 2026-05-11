# Diagnostic: isolate the predecessor TCC + musl static startup boundary.
#
# This focused probe avoids the broad GCC c-parse matrix. It proves the C
# front-end can emit main objects, static links succeed, and the minimal
# crt1/__libc_start_main path can now enter/return from main. The remaining
# write-main failure is past startup; this diagnostic splits syscall-only entry,
# main-owned raw syscall, and libc write marker paths.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in

{
  name = "diag-musl-startup-boundary-v9",
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
      STAGE0=$(find_input stage0-posix)
      export PATH="$TCC/bin:$STAGE0/bin:$PATH"

      run_phase() {
        phase="$1"
        shift
        echo "diag-musl-startup-phase: $phase" >&2
        set +e
        "$@" > "/tmp/diag-musl-$phase.stdout" 2> "/tmp/diag-musl-$phase.stderr"
        rc=$?
        set -e
        echo "diag-musl-startup-phase: $phase rc=$rc" >&2
        $BB sed 's/^/diag-musl-startup-phase: '$phase' stdout: /' "/tmp/diag-musl-$phase.stdout" >&2 || true
        $BB sed 's/^/diag-musl-startup-phase: '$phase' stderr: /' "/tmp/diag-musl-$phase.stderr" >&2 || true
      }

      run_probe() {
        label="$1"
        path="/tmp/diag-musl-$label"
        if [ -f "$path" ]; then $BB chmod 755 "$path" || true; fi
        echo "diag-musl-startup-run: $label" >&2
        set +e
        $BB timeout 5 "$path" > "/tmp/diag-musl-run-$label.stdout" 2> "/tmp/diag-musl-run-$label.stderr"
        rc=$?
        set -e
        echo "diag-musl-startup-run: $label rc=$rc" >&2
        $BB sed 's/^/diag-musl-startup-run: '$label' stdout: /' "/tmp/diag-musl-run-$label.stdout" >&2 || true
        $BB sed 's/^/diag-musl-startup-run: '$label' stderr: /' "/tmp/diag-musl-run-$label.stderr" >&2 || true
      }

      echo "diag-musl-startup: tcc=$TCC/bin/tcc" >&2
      echo "diag-musl-startup: musl=$MUSL" >&2

      $BB cat > /tmp/write-main.c <<'EOF_WRITE_MAIN'
int write(int fd, const void *buf, unsigned long count);
int main(int argc, char **argv) { (void)argv; write(2, "diag-musl-startup-main: write-main\n", 36); return argc == 123; }
EOF_WRITE_MAIN
      $BB cat > /tmp/raw-write-main.c <<'EOF_RAW_WRITE_MAIN'
__asm__(".global raw_write\n"
        "raw_write:\n"
        "    mov $1,%rax\n"
        "    syscall\n"
        "    ret\n");
int raw_write(int fd, const void *buf, unsigned long count);
int main(int argc, char **argv) { (void)argv; raw_write(2, "diag-musl-startup-main: raw-write-main\n", 40); return argc == 123; }
EOF_RAW_WRITE_MAIN
      $BB cat > /tmp/return-main.c <<'EOF_RETURN_MAIN'
int main(int argc, char **argv) { (void)argv; return argc == 123; }
EOF_RETURN_MAIN
      $BB cat > /tmp/syscall-start.s <<'EOF_SYSCALL_START'
.global _start
_start:
    mov $1,%rax
    mov $2,%rdi
    lea msg(%rip),%rsi
    mov $37,%rdx
    syscall
    mov $60,%rax
    xor %rdi,%rdi
    syscall
msg:
    .ascii "diag-musl-startup-main: syscall-start\n"
EOF_SYSCALL_START

      run_phase compile-write-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/write-main.c -o /tmp/write-main.o
      run_phase compile-raw-write-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/raw-write-main.c -o /tmp/raw-write-main.o
      run_phase compile-return-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/return-main.c -o /tmp/return-main.o
      run_phase compile-syscall-start "$TCC/bin/tcc" -c /tmp/syscall-start.s -o /tmp/syscall-start.o
      for obj in /tmp/write-main.o /tmp/raw-write-main.o /tmp/return-main.o /tmp/syscall-start.o; do
        if [ -f "$obj" ]; then
          $BB chmod u+rw "$obj" || true
          echo "diag-musl-startup-object: exists $obj" >&2
        else
          echo "diag-musl-startup-object: missing $obj" >&2
        fi
      done

      run_phase link-syscall-start "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-syscall-start /tmp/syscall-start.o
      run_phase link-manual-crt-return "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-return "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/return-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-raw-write "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-raw-write "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/raw-write-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-write "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-write "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/write-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-driver-write "$TCC/bin/tcc" -static -o /tmp/diag-musl-driver-write /tmp/write-main.c

      for probe in syscall-start manual-crt-return manual-crt-raw-write manual-crt-write driver-write; do
        run_probe "$probe"
      done

      $BB mkdir -p "$out"
      {
        echo 'diag-musl-startup-boundary complete'
        echo 'expected-current-boundary: compile/link succeed; syscall-only start and return-main exit rc0; raw-write vs libc-write probes isolate the remaining marker syscall/libc surface'
      } > "$out/report.txt"
    "%,
  ],
  inputs = [stage0, tcc, musl],
} | crunch.Derivation
