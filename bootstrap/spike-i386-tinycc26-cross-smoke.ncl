# Spike proof: build an i386-targeting TinyCC 0.9.26 and execute its i386 output.
#
# This follows the earlier Mes/TinyCC handoff than the 0.9.27 cross-smoke: use
# Crunch's Mes-built TinyCC 0.9.26 to build a x86_64-hosted/I386-targeting
# TinyCC 0.9.26, then use that compiler to emit a no-libc native i386 ELF.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc26 = import "tinycc-mes.ncl" in

let tcc26_src = crunch.fetchTarball {
  url = "https://lilypond.org/janneke/tcc/tcc-0.9.26-1147-gee75a10c.tar.gz",
  hash = "sha256-Bkmpev9x8vKh2P9X8Z4VSpcy4BJtzc0Tra3WPImnuhc=",
  name = "tcc-0.9.26-src",
} in

{
  name = "spike-i386-tinycc26-cross-smoke",
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

      WORK=/tmp/spike-i386-tinycc26-cross-smoke
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"
      cd "$WORK"
      $BB cp -r "$TCC_SRC" tcc-0.9.26
      $BB chmod -R u+w tcc-0.9.26
      cd tcc-0.9.26
      : > config.h
      # Carry the same Mes/TCC source-normalization seams used by the amd64
      # handoff. The Mes-built 0.9.26 compiler can emit a new static compiler,
      # but the source must avoid known varargs/string-constructor traps.
      $BB sed -i 's/const char filename\[\]/const char *filename/' tccelf.c
      $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
      $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
      $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
      $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
      $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c
      $BB sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\
        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c
      $BB sed -i '/ST_FUNC int tcc_add_crt(TCCState \*s, const char \*filename)/,/^}/c\
ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\
{\
    char buf[1024];\
    int i;\
    if (!strcmp(filename, "crti.o") || !strcmp(filename, "crtn.o"))\
        return 0;\
    for (i = 0; i < s->nb_crt_paths; i++) {\
        strcpy(buf, s->crt_paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        if (tcc_add_file_internal(s, buf, AFF_TYPE_BIN) == 0)\
            return 0;\
    }\
    tcc_error_noabort("file '\''%s'\'' not found", filename);\
    return 0;\
}' libtcc.c
      $BB sed -i '/sprintf(buf,/c\
    buf[0] = '\''"'\''; strcpy(buf + 1, file->filename); strcat(buf, "\\"");' tccpp.c
      $BB sed -i 's/snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);/strcpy(buf, ".rel"); strcat(buf, s->name);/' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__start_%s", s->name);|strcpy(buf, "__start_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__stop_%s", s->name);|strcpy(buf, "__stop_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(sym_start, sizeof(sym_start), "__%s_start", section_name + 1);|strcpy(sym_start, "__"); strcat(sym_start, section_name + 1); strcat(sym_start, "_start");|' tccelf.c
      $BB sed -i 's|snprintf(sym_end, sizeof(sym_end), "__%s_end", section_name + 1);|strcpy(sym_end, "__"); strcat(sym_end, section_name + 1); strcat(sym_end, "_end");|' tccelf.c
      $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
      $BB sed -i 's|tcc_error_noabort("file '\''%s'\'' not found", filename);|fputs("file not found: ", stderr); fputs(filename, stderr); fputs("\\n", stderr);|g' libtcc.c
      $BB sed -i 's|tcc_error_noabort("library '\''%s'\'' not found", libname);|fputs("library not found: ", stderr); fputs(libname, stderr); fputs("\\n", stderr);|g' libtcc.c
      $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
      $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
      $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c

      echo 'i386-tcc26: build x86_64-hosted/i386-targeting TinyCC 0.9.26' >&2
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
        -D TCC_VERSION=\"0.9.26-i386-spike\" \
        -D ONE_SOURCE=1 \
        tcc.c
      $BB chmod 755 tcc26-i386
      ./tcc26-i386 -version

      $BB cat > exit42.s <<'EOF'
.globl _start
_start:
  movl $1, %eax
  movl $42, %ebx
  int $0x80
EOF

      echo 'i386-tcc26: assemble/link no-libc i386 exit42 executable' >&2
      ./tcc26-i386 -nostdlib -static -o i386-exit42 exit42.s
      $BB chmod 755 i386-exit42

      set +e
      ./i386-exit42
      rc=$?
      set -e
      echo "i386-tcc26: i386-exit42 rc=$rc"
      test "$rc" = 42

      $BB mkdir -p "$out/bin" "$out/share/spike-i386-tinycc26-cross-smoke"
      $BB cp tcc26-i386 "$out/bin/tcc26-i386"
      $BB cp i386-exit42 "$out/bin/i386-exit42"
      $BB printf 'TinyCC 0.9.26 i386 target produced native i386 executable with rc=42\n' > "$out/share/spike-i386-tinycc26-cross-smoke/result.txt"
      test -x "$out/bin/tcc26-i386"
      test -x "$out/bin/i386-exit42"
    "%,
  ],
  inputs = [stage0, tcc26, tcc26_src],
} | crunch.Derivation
