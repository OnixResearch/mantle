# Diagnostic proof: split i386-targeting TinyCC 0.9.26 output generation.
#
# This is intentionally diagnostic-only. It builds the same x86_64-hosted / i386
# targeting TinyCC 0.9.26 as spike-i386-tinycc26-cross-smoke.ncl, then records
# version, assemble-only, link-from-assembly, link-from-object, and run-output
# stages separately so the segfault boundary is visible in the derivation log.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc26 = import "tinycc-mes.ncl" in

let tcc26_src = crunch.fetchTarball {
  url = "https://lilypond.org/janneke/tcc/tcc-0.9.26-1147-gee75a10c.tar.gz",
  hash = "sha256-Bkmpev9x8vKh2P9X8Z4VSpcy4BJtzc0Tra3WPImnuhc=",
  name = "tcc-0.9.26-src",
} in

{
  name = "diag-i386-tinycc26-emission",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
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

      STAGE0=$(find_input stage0-posix)
      TCC26=$(find_input tinycc-0.9.26)
      TCC_SRC=$(find_input tcc-0.9.26-src)
      export PATH="$TCC26/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/diag-i386-tinycc26-emission
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK/logs" "$WORK/artifacts"
      cd "$WORK"
      $BB cp -r "$TCC_SRC" tcc-0.9.26
      $BB chmod -R u+w tcc-0.9.26
      cd tcc-0.9.26
      : > config.h

      echo 'diag-i386-tcc26: build x86_64-hosted/i386-targeting TinyCC 0.9.26' >&2
      tcc-0.9.26 \
        -v \
        -static \
        -o tcc26-i386 \
        -D BOOTSTRAP=1 \
        -D HAVE_FLOAT=1 \
        -D HAVE_BITFIELD=1 \
        -D HAVE_LONG_LONG=1 \
        -D HAVE_SETJMP=1 \
        -D TCC_TARGET_I386=1 \
        -D CONFIG_TCCDIR=\"/no-runtime/tcc\" \
        -D CONFIG_TCC_CRTPREFIX=\"/no-runtime\" \
        -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
        -D CONFIG_TCC_LIBPATHS=\"/no-runtime:/no-runtime/tcc\" \
        -D CONFIG_TCC_SYSINCLUDEPATHS=\"/no-runtime/include\" \
        -D TCC_LIBGCC=\"/no-runtime/libc.a\" \
        -D TCC_LIBTCC1=\"libtcc1.a\" \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.26-i386-diag\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc26-i386

      $BB cat > exit42.s <<'EOF'
.globl _start
_start:
  movl $1, %eax
  movl $42, %ebx
  int $0x80
EOF

      $BB cat > trivial.c <<'EOF'
int x = 1;
int f(void) { return x; }
EOF

      run_step() {
        name="$1"
        shift
        echo "diag-i386-tcc26: STEP $name: $*" >&2
        set +e
        "$@" >"../logs/$name.stdout" 2>"../logs/$name.stderr"
        rc=$?
        set -e
        echo "$rc" >"../logs/$name.rc"
        echo "diag-i386-tcc26: RESULT $name rc=$rc" >&2
      }

      note_file() {
        name="$1"
        path="$2"
        if [ -e "$path" ]; then
          size=$($BB wc -c < "$path")
          mode=$($BB ls -l "$path")
          echo "diag-i386-tcc26: FILE $name exists size=$size mode=$mode" >&2
          $BB cp "$path" "../artifacts/$name" 2>/dev/null || true
          $BB od -An -tx1 -N64 "$path" >"../logs/$name.head64" 2>/dev/null || true
        else
          echo "diag-i386-tcc26: FILE $name missing" >&2
        fi
      }

      run_step version ./tcc26-i386 -version
      run_step assemble_object ./tcc26-i386 -c -o exit42.o exit42.s
      note_file exit42.o exit42.o
      run_step compile_c_object ./tcc26-i386 -c -o trivial.o trivial.c
      note_file trivial.o trivial.o
      run_step link_from_asm ./tcc26-i386 -nostdlib -static -Wl,-e,_start -o i386-exit42-from-asm exit42.s
      note_file i386-exit42-from-asm i386-exit42-from-asm
      if [ -e exit42.o ]; then
        run_step link_from_object ./tcc26-i386 -nostdlib -static -Wl,-e,_start -o i386-exit42-from-object exit42.o
      else
        echo 127 > ../logs/link_from_object.rc
        echo 'skipped: exit42.o missing' > ../logs/link_from_object.stdout
        : > ../logs/link_from_object.stderr
        echo 'diag-i386-tcc26: RESULT link_from_object rc=127 skipped-missing-object' >&2
      fi
      note_file i386-exit42-from-object i386-exit42-from-object

      for exe in i386-exit42-from-asm i386-exit42-from-object; do
        if [ -e "$exe" ]; then
          $BB chmod 755 "$exe"
          run_step "run_$exe" "./$exe"
        else
          echo 127 > "../logs/run_$exe.rc"
          echo "skipped: $exe missing" > "../logs/run_$exe.stdout"
          : > "../logs/run_$exe.stderr"
          echo "diag-i386-tcc26: RESULT run_$exe rc=127 skipped-missing-exe" >&2
        fi
      done

      $BB mkdir -p "$out/bin" "$out/share/diag-i386-tinycc26-emission/logs" "$out/share/diag-i386-tinycc26-emission/artifacts"
      $BB cp tcc26-i386 "$out/bin/tcc26-i386"
      $BB cp ../logs/* "$out/share/diag-i386-tinycc26-emission/logs/"
      $BB cp ../artifacts/* "$out/share/diag-i386-tinycc26-emission/artifacts/" 2>/dev/null || true
      {
        echo 'diag-i386-tcc26 summary'
        for rc in ../logs/*.rc; do
          step=${rc##*/}
          step=${step%.rc}
          printf '%s rc=' "$step"
          $BB cat "$rc"
        done
      } > "$out/share/diag-i386-tinycc26-emission/summary.txt"
      $BB cat "$out/share/diag-i386-tinycc26-emission/summary.txt"
    "%,
  ],
  inputs = [stage0, tcc26, tcc26_src],
} | crunch.Derivation
