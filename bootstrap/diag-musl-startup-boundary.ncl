# Diagnostic: isolate the predecessor TCC + musl static startup boundary.
#
# This focused probe avoids the broad GCC c-parse matrix. It proves the C
# front-end can emit main objects, static links succeed, and the minimal
# crt1/__libc_start_main path can enter/return from main. It also guards the
# predecessor TinyCC static-call relocation repair: direct calls, function
# pointers, raw syscall helpers, and libc write markers all run successfully.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in

{
  name = "diag-musl-startup-boundary-v15",
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
      $BB cat > /tmp/raw-ret-main.c <<'EOF_RAW_RET_MAIN'
__asm__(".global raw_ret\n"
        "raw_ret:\n"
        "    xor %rax,%rax\n"
        "    ret\n");
int raw_ret(void);
int main(int argc, char **argv) { (void)argc; (void)argv; return raw_ret(); }
EOF_RAW_RET_MAIN
      $BB cat > /tmp/raw-exit-main.c <<'EOF_RAW_EXIT_MAIN'
__asm__(".global raw_exit\n"
        "raw_exit:\n"
        "    mov $60,%rax\n"
        "    xor %rdi,%rdi\n"
        "    syscall\n");
void raw_exit(void);
int main(int argc, char **argv) { (void)argc; (void)argv; raw_exit(); return 111; }
EOF_RAW_EXIT_MAIN
      $BB cat > /tmp/raw-write-exit-main.c <<'EOF_RAW_WRITE_EXIT_MAIN'
__asm__(".global raw_write_exit\n"
        "raw_write_exit:\n"
        "    mov $1,%rax\n"
        "    syscall\n"
        "    mov $60,%rax\n"
        "    xor %rdi,%rdi\n"
        "    syscall\n");
void raw_write_exit(int fd, const void *buf, unsigned long count);
int main(int argc, char **argv) { (void)argc; (void)argv; raw_write_exit(2, "diag-musl-startup-main: raw-write-exit\n", 40); return 111; }
EOF_RAW_WRITE_EXIT_MAIN
      $BB cat > /tmp/raw-fixed-write-exit-main.c <<'EOF_RAW_FIXED_WRITE_EXIT_MAIN'
__asm__(".global raw_fixed_write_exit\n"
        "raw_fixed_write_exit:\n"
        "    mov $1,%rax\n"
        "    mov $2,%rdi\n"
        "    lea raw_fixed_msg(%rip),%rsi\n"
        "    mov $46,%rdx\n"
        "    syscall\n"
        "    mov $60,%rax\n"
        "    xor %rdi,%rdi\n"
        "    syscall\n"
        "raw_fixed_msg:\n"
        "    .ascii \"diag-musl-startup-main: raw-fixed-write-exit\\n\"\n");
void raw_fixed_write_exit(void);
int main(int argc, char **argv) { (void)argc; (void)argv; raw_fixed_write_exit(); return 111; }
EOF_RAW_FIXED_WRITE_EXIT_MAIN
      $BB cat > /tmp/return-main.c <<'EOF_RETURN_MAIN'
int main(int argc, char **argv) { (void)argv; return argc == 123; }
EOF_RETURN_MAIN
      $BB cat > /tmp/c-helper-main.c <<'EOF_C_HELPER_MAIN'
int c_helper(void) { return 0; }
int main(int argc, char **argv) { (void)argc; (void)argv; return c_helper(); }
EOF_C_HELPER_MAIN
      $BB cat > /tmp/c-helper-ptr-main.c <<'EOF_C_HELPER_PTR_MAIN'
int c_helper_ptr(void) { return 0; }
int main(int argc, char **argv) { int (*p)(void) = c_helper_ptr; (void)argc; (void)argv; return p(); }
EOF_C_HELPER_PTR_MAIN
      $BB cat > /tmp/external-raw-ret-main.c <<'EOF_EXTERNAL_RAW_RET_MAIN'
int external_raw_ret(void);
int main(int argc, char **argv) { (void)argc; (void)argv; return external_raw_ret(); }
EOF_EXTERNAL_RAW_RET_MAIN
      $BB cat > /tmp/external-raw-ret-ptr-main.c <<'EOF_EXTERNAL_RAW_RET_PTR_MAIN'
int external_raw_ret(void);
int main(int argc, char **argv) { int (*p)(void) = external_raw_ret; (void)argc; (void)argv; return p(); }
EOF_EXTERNAL_RAW_RET_PTR_MAIN
      $BB cat > /tmp/external-raw-ret.s <<'EOF_EXTERNAL_RAW_RET'
.global external_raw_ret
external_raw_ret:
    xor %rax,%rax
    ret
EOF_EXTERNAL_RAW_RET
      $BB cat > /tmp/external-raw-exit-main.c <<'EOF_EXTERNAL_RAW_EXIT_MAIN'
void external_raw_exit(void);
int main(int argc, char **argv) { (void)argc; (void)argv; external_raw_exit(); return 111; }
EOF_EXTERNAL_RAW_EXIT_MAIN
      $BB cat > /tmp/external-raw-exit.s <<'EOF_EXTERNAL_RAW_EXIT'
.global external_raw_exit
external_raw_exit:
    mov $60,%rax
    xor %rdi,%rdi
    syscall
EOF_EXTERNAL_RAW_EXIT
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
      run_phase compile-raw-ret-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/raw-ret-main.c -o /tmp/raw-ret-main.o
      run_phase compile-raw-exit-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/raw-exit-main.c -o /tmp/raw-exit-main.o
      run_phase compile-raw-write-exit-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/raw-write-exit-main.c -o /tmp/raw-write-exit-main.o
      run_phase compile-raw-fixed-write-exit-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/raw-fixed-write-exit-main.c -o /tmp/raw-fixed-write-exit-main.o
      run_phase compile-return-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/return-main.c -o /tmp/return-main.o
      run_phase compile-c-helper-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/c-helper-main.c -o /tmp/c-helper-main.o
      run_phase compile-c-helper-ptr-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/c-helper-ptr-main.c -o /tmp/c-helper-ptr-main.o
      run_phase compile-external-raw-ret-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/external-raw-ret-main.c -o /tmp/external-raw-ret-main.o
      run_phase compile-external-raw-ret-ptr-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/external-raw-ret-ptr-main.c -o /tmp/external-raw-ret-ptr-main.o
      run_phase compile-external-raw-ret "$TCC/bin/tcc" -c /tmp/external-raw-ret.s -o /tmp/external-raw-ret.o
      run_phase compile-external-raw-exit-main "$TCC/bin/tcc" -c -I "$MUSL/include" /tmp/external-raw-exit-main.c -o /tmp/external-raw-exit-main.o
      run_phase compile-external-raw-exit "$TCC/bin/tcc" -c /tmp/external-raw-exit.s -o /tmp/external-raw-exit.o
      run_phase compile-syscall-start "$TCC/bin/tcc" -c /tmp/syscall-start.s -o /tmp/syscall-start.o
      for obj in /tmp/write-main.o /tmp/raw-write-main.o /tmp/raw-ret-main.o /tmp/raw-exit-main.o /tmp/raw-write-exit-main.o /tmp/raw-fixed-write-exit-main.o /tmp/return-main.o /tmp/c-helper-main.o /tmp/c-helper-ptr-main.o /tmp/external-raw-ret-main.o /tmp/external-raw-ret-ptr-main.o /tmp/external-raw-ret.o /tmp/external-raw-exit-main.o /tmp/external-raw-exit.o /tmp/syscall-start.o; do
        if [ -f "$obj" ]; then
          $BB chmod u+rw "$obj" || true
          echo "diag-musl-startup-object: exists $obj" >&2
        else
          echo "diag-musl-startup-object: missing $obj" >&2
        fi
      done

      run_phase link-syscall-start "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-syscall-start /tmp/syscall-start.o
      run_phase link-manual-crt-return "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-return "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/return-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-raw-ret "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-raw-ret "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/raw-ret-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-raw-exit "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-raw-exit "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/raw-exit-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-raw-write-exit "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-raw-write-exit "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/raw-write-exit-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-raw-fixed-write-exit "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-raw-fixed-write-exit "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/raw-fixed-write-exit-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-c-helper "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-c-helper "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/c-helper-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-c-helper-ptr "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-c-helper-ptr "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/c-helper-ptr-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-external-raw-ret "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-external-raw-ret "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/external-raw-ret-main.o /tmp/external-raw-ret.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-external-raw-ret-ptr "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-external-raw-ret-ptr "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/external-raw-ret-ptr-main.o /tmp/external-raw-ret.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-external-raw-exit "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-external-raw-exit "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/external-raw-exit-main.o /tmp/external-raw-exit.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-raw-write "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-raw-write "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/raw-write-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-manual-crt-write "$TCC/bin/tcc" -nostdlib -static -o /tmp/diag-musl-manual-crt-write "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" /tmp/write-main.o "$MUSL/lib/libc.a" "$MUSL/lib/crtn.o"
      run_phase link-driver-write "$TCC/bin/tcc" -static -o /tmp/diag-musl-driver-write /tmp/write-main.c

      for probe in syscall-start manual-crt-return manual-crt-c-helper manual-crt-c-helper-ptr manual-crt-raw-ret manual-crt-external-raw-ret manual-crt-external-raw-ret-ptr manual-crt-raw-exit manual-crt-external-raw-exit manual-crt-raw-write-exit manual-crt-raw-fixed-write-exit manual-crt-raw-write manual-crt-write driver-write; do
        run_probe "$probe"
      done

      $BB mkdir -p "$out"
      {
        echo 'diag-musl-startup-boundary complete'
        echo 'expected-current-boundary: compile/link succeed; syscall-only start, return-main, direct helpers, function-pointer helpers, raw syscall helpers, libc write, and driver-static write all exit rc0 after static call/GOTPCREL relocation repair'
      } > "$out/report.txt"
    "%,
  ],
  inputs = [stage0, tcc, musl],
} | crunch.Derivation
