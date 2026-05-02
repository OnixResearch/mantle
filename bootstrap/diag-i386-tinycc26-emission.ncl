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
      # Carry the Mes/TCC source-normalization seams needed when the
      # Mes-built 0.9.26 compiler emits a new static compiler binary.
      $BB sed -i 's/const char filename\[\]/const char *filename/' tccelf.c
      $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
      $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
      $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
      $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
      $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c
      # Mes/TCC varargs can corrupt snprintf/sprintf string expansion. Spell
      # deterministic bootstrap path/string constructors directly.
      $BB sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\
        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c
      $BB sed -i '/sprintf(buf,/c\
    buf[0] = '\''"'\''; strcpy(buf + 1, file->filename); strcat(buf, "\\"");' tccpp.c
      $BB sed -i 's/snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);/strcpy(buf, ".rel"); strcat(buf, s->name);/' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__start_%s", s->name);|strcpy(buf, "__start_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__stop_%s", s->name);|strcpy(buf, "__stop_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(sym_start, sizeof(sym_start), "__%s_start", section_name + 1);|strcpy(sym_start, "__"); strcat(sym_start, section_name + 1); strcat(sym_start, "_start");|' tccelf.c
      $BB sed -i 's|snprintf(sym_end, sizeof(sym_end), "__%s_end", section_name + 1);|strcpy(sym_end, "__"); strcat(sym_end, section_name + 1); strcat(sym_end, "_end");|' tccelf.c
      $BB grep -F 'strcpy(buf, ".rel"); strcat(buf, s->name);' tccelf.c >/dev/null
      # Keep any remaining errors from re-entering the broken Mes varargs
      # formatter while diagnostics converge.
      $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
      $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
      $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
      $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c

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
          $BB chmod u+rw "$path" 2>/dev/null || true
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
      run_step link_from_asm ./tcc26-i386 -nostdlib -static -o i386-exit42-from-asm exit42.s
      note_file i386-exit42-from-asm i386-exit42-from-asm
      if [ -e exit42.o ]; then
        run_step link_from_object ./tcc26-i386 -nostdlib -static -o i386-exit42-from-object exit42.o
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
