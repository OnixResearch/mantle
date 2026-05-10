#!/usr/bin/env bash
set -euo pipefail
ROOT=${ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}
STORE=${STORE:-$ROOT/.crunch-drain/post621-restored-store}
TCC=${TCC:-$STORE/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2}
MUSL=${MUSL:-$STORE/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl}
SRC=${SRC:-$STORE/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src}
WORK=${WORK:-/tmp/tcc-va-cleanup-reconcile}
rm -rf "$WORK"
mkdir -p "$WORK"

prep_common() {
  local w=$1
  rm -rf "$w"
  cp -R "$SRC" "$w"
  chmod -R u+w "$w"
  cd "$w"
  : > config.h
  sed -i '/#include <inttypes.h>/a\
#define uint8_t unsigned char\
#define uint16_t unsigned short\
#define uint32_t unsigned int\
#define uint64_t unsigned long long\
#define int32_t int\
#define int64_t long long\
#define CRUNCH_TCC_ELF_FIXED_WIDTH_ALIASES 1\
' elf.h
  sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
  sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
  sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
  sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
  sed -i 's/SValue tmp = vtop\[0\];/SValue tmp; memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
  sed -i 's/tmp = vtop\[0\];/memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
  sed -i 's/vtop\[0\] = vtop\[-i\];/memcpy(\&vtop[0], \&vtop[-i], sizeof(SValue));/' x86_64-gen.c
  sed -i 's/vtop\[-i\] = tmp;/memcpy(\&vtop[-i], \&tmp, sizeof(SValue));/' x86_64-gen.c
  sed -i 's/#define REX_BASE(reg).*/#define REX_BASE(reg) (((reg) \& 8) ? 1 : 0)/' x86_64-gen.c
  sed -i 's/g(vtop->c.i \& (ll ? 63 : 31));/g(vtop->c.i);/' x86_64-gen.c
  sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c
  sed -i '/sprintf(buf,/c\    buf[0] = 34; strcpy(buf + 1, file->filename); strcat(buf, "\\\"");' tccpp.c
  sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rela"); strcat(buf, s->name);|' tccelf.c
  sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c
  sed -i '/ST_FUNC int tcc_add_crt(TCCState \*s, const char \*filename)/,/^}/c\ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\
{\
    char buf[1024]; int i;\
    if (!strcmp(filename, "crti.o") || !strcmp(filename, "crtn.o")) return 0;\
    for (i = 0; i < s->nb_crt_paths; i++) { strcpy(buf, s->crt_paths[i]); strcat(buf, "/"); strcat(buf, filename); if (tcc_add_file_internal(s, buf, AFF_TYPE_BIN) == 0) return 0; }\
    return 0; /* CRUNCH diag: suppress missing crt error in predecessor build */;\
}' libtcc.c
  sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
  sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
  sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
  sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
  sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
  cp tccgen.c tccgen.c.before-native387-diag
  awk 'done == 0 && $0 == "#if defined TCC_IS_NATIVE_387" { print "#if 0 /* CRUNCH diag: predecessor tccgen native x87 compile boundary */"; done = 1; next } { print }' tccgen.c.before-native387-diag > tccgen.c
  awk 'NR == 642 { print "            tcc_error_noabort(\"asm not supported\");"; skip = 1; next } skip && NR <= 646 { print "            /* CRUNCH diag: CONFIG_TCC_ASM island removed */"; next } { print }' libtcc.c > libtcc.c.tmp
  mv libtcc.c.tmp libtcc.c
}

apply_va_cleanup() {
  sed -i 's/^[[:space:]]*va_start(ap, fmt);/    \/\* CRUNCH diag: va_start removed with va_end pair \*\//g; s/^[[:space:]]*va_end(ap);/    \/\* CRUNCH diag: va_end removed with va_start pair \*\//g' libtcc.c
}

compile_case() {
  local label=$1
  local dir=$2
  cd "$dir"
  local flags='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION="0.9.27-decl0-diag" -D ONE_SOURCE=1'
  set +e
  "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $flags libtcc.c -o "$WORK/$label.o" > "$WORK/$label.stdout" 2> "$WORK/$label.stderr"
  local rc=$?
  set -e
  printf 'diag-libtcc-va-cleanup: compile %s rc=%s object=%s\n' "$label" "$rc" "$(test -f "$WORK/$label.o" && echo yes || echo no)"
  sed 's/^/diag-libtcc-va-cleanup: stderr: /' "$WORK/$label.stderr" || true
}

printf 'diag-libtcc-va-cleanup: tcc=%s\n' "$TCC/bin/tcc"
printf 'diag-libtcc-va-cleanup: source=%s\n' "$SRC"
mkdir -p "$WORK/asm-only" "$WORK/with-va-cleanup"
prep_common "$WORK/asm-only"
compile_case asm_simplified_one_source "$WORK/asm-only"
prep_common "$WORK/with-va-cleanup"
apply_va_cleanup
compile_case va_cleanup_one_source "$WORK/with-va-cleanup"
