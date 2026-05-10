# Diagnostic: prove the libtcc one-source predecessor boundary isolated by
# CONFIG_TCC_ASM simplification and paired va_start/va_end cleanup.
#
# This intentionally avoids the later GCC/binutils/bash path so it can run even
# while bash-2.05b-tcc has its own parser handoff failure.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc = import "tcc-musl-v2.ncl" in
let musl = import "musl-1.1.24-tcc-musl.ncl" in

let tcc_src = crunch.fetchTarball {
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-iaWFw3NwCH5fV71ybBYKzXJKC8ALl6Hn6woEW29aXfs=",
  name = "tcc-0.9.27-src",
} in

{
  name = "diag-libtcc-va-cleanup-boundary",
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
      SRC=$(find_input tcc-0.9.27-src)
      export PATH="$TCC/bin:$STAGE0/bin:$PATH"

      prep_common() {
        w="$1"
        $BB rm -rf "$w"
        $BB cp -R "$SRC" "$w"
        $BB chmod -R u+w "$w"
        cd "$w"
        : > config.h
        $BB sed -i '/#include <inttypes.h>/a\
#define uint8_t unsigned char\
#define uint16_t unsigned short\
#define uint32_t unsigned int\
#define uint64_t unsigned long long\
#define int32_t int\
#define int64_t long long\
#define CRUNCH_TCC_ELF_FIXED_WIDTH_ALIASES 1\
' elf.h
        $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
        $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
        $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
        $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
        $BB sed -i 's/SValue tmp = vtop\[0\];/SValue tmp; memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/tmp = vtop\[0\];/memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/vtop\[0\] = vtop\[-i\];/memcpy(\&vtop[0], \&vtop[-i], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/vtop\[-i\] = tmp;/memcpy(\&vtop[-i], \&tmp, sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/#define REX_BASE(reg).*/#define REX_BASE(reg) (((reg) \& 8) ? 1 : 0)/' x86_64-gen.c
        $BB sed -i 's/g(vtop->c.i \& (ll ? 63 : 31));/g(vtop->c.i);/' x86_64-gen.c
        $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c
        $BB sed -i '/sprintf(buf,/c\    buf[0] = 34; strcpy(buf + 1, file->filename); strcat(buf, "\\"");' tccpp.c
        $BB sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rela"); strcat(buf, s->name);|' tccelf.c
        $BB sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c
        $BB sed -i '/ST_FUNC int tcc_add_crt(TCCState \*s, const char \*filename)/,/^}/c\ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\
{\
    char buf[1024]; int i;\
    if (!strcmp(filename, "crti.o") || !strcmp(filename, "crtn.o")) return 0;\
    for (i = 0; i < s->nb_crt_paths; i++) { strcpy(buf, s->crt_paths[i]); strcat(buf, "/"); strcat(buf, filename); if (tcc_add_file_internal(s, buf, AFF_TYPE_BIN) == 0) return 0; }\
    return 0; /* CRUNCH diag: suppress missing crt error in predecessor build */;\
}' libtcc.c
        $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
        $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
        $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
        $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
        $BB sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
        $BB cp tccgen.c tccgen.c.before-native387-diag
        $BB awk 'done == 0 && $0 == "#if defined TCC_IS_NATIVE_387" { print "#if 0 /* CRUNCH diag: predecessor tccgen native x87 compile boundary */"; done = 1; next } { print }' tccgen.c.before-native387-diag > tccgen.c
        $BB awk 'NR == 642 { print "            tcc_error_noabort(\"asm not supported\");"; skip = 1; next } skip && NR <= 646 { print "            /* CRUNCH diag: CONFIG_TCC_ASM island removed */"; next } { print }' libtcc.c > libtcc.c.tmp
        $BB mv libtcc.c.tmp libtcc.c
      }

      compile_case() {
        label="$1"
        dir="$2"
        cd "$dir"
        flags='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION="0.9.27-decl0-diag" -D ONE_SOURCE=1'
        set +e
        "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $flags libtcc.c -o "/tmp/$label.o" > "/tmp/$label.stdout" 2> "/tmp/$label.stderr"
        rc=$?
        set -e
        object=no
        test -f "/tmp/$label.o" && object=yes
        echo "diag-libtcc-va-cleanup: compile $label rc=$rc object=$object" >&2
      }

      echo "diag-libtcc-va-cleanup: tcc=$TCC/bin/tcc" >&2
      echo "diag-libtcc-va-cleanup: source=$SRC" >&2
      prep_common /tmp/asm-only
      compile_case asm_simplified_one_source /tmp/asm-only
      prep_common /tmp/with-va-cleanup
      cd /tmp/with-va-cleanup
      $BB sed -i 's/^[[:space:]]*va_start(ap, fmt);/    \/\* CRUNCH diag: va_start removed with va_end pair \*\//g; s/^[[:space:]]*va_end(ap);/    \/\* CRUNCH diag: va_end removed with va_start pair \*\//g' libtcc.c
      compile_case va_cleanup_one_source /tmp/with-va-cleanup

      if [ -f /tmp/asm_simplified_one_source.o ]; then
        echo "ERROR: asm_simplified unexpectedly produced an object" >&2
        exit 1
      fi
      test -f /tmp/va_cleanup_one_source.o
      $BB mkdir -p "$out"
      $BB cp /tmp/va_cleanup_one_source.o "$out/libtcc-va-cleanup.o"
    "%,
  ],
  inputs = [stage0, tcc, musl, tcc_src],
} | crunch.Derivation
