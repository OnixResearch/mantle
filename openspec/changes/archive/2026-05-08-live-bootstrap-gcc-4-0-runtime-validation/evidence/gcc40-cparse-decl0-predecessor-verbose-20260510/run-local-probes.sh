#!/usr/bin/env bash
set -u
ROOT=${ROOT:-/home/brittonr/git/crunch/crunch}
STORE="$ROOT/.crunch-drain/post621-restored-store"
TCC="$STORE/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2"
MUSL="$STORE/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl"
SRC="$STORE/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src"
WORK=${WORK:-/tmp/tcc-predecessor-verbose-local}
rm -rf "$WORK"
mkdir -p "$WORK/tiny" "$WORK/src"

run_tiny() {
  local name=$1
  shift
  cd "$WORK/tiny"
  printf 'int x;\n' > tiny.c
  echo "diag-predecessor-verbose: tiny $name"
  set +e
  "$TCC/bin/tcc" -c -v "$@" tiny.c -o "$name.o" >"$name.out" 2>"$name.err"
  local rc=$?
  set -e
  echo "diag-predecessor-verbose: tiny $name rc=$rc obj=$(test -f "$name.o" && echo yes || echo no)"
  sed "s/^/diag-predecessor-verbose: tiny $name stdout: /" "$name.out"
  sed "s/^/diag-predecessor-verbose: tiny $name stderr: /" "$name.err"
}

prep_source() {
  rm -rf "$WORK/src"
  cp -R "$SRC" "$WORK/src"
  chmod -R u+w "$WORK/src"
  cd "$WORK/src"
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
    return 0;\
}' libtcc.c
  sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
  sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
  sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
  sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
  sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
  awk '{ if ($0 ~ /^[[:space:]]*sscanf\(TCC_VERSION,/) print "        a = 0; b = 9; c = 27; /* CRUNCH diag: sscanf(TCC_VERSION) bypassed */"; else print }' libtcc.c > libtcc.c.tmp
  mv libtcc.c.tmp libtcc.c
  sed -i '/copy_linker_arg(\&s->/c\            /* CRUNCH diag: copy_linker_arg field call removed */' libtcc.c
}

run_source() {
  local name=$1
  local source=$2
  cd "$WORK/src"
  echo "diag-predecessor-verbose: source $name"
  set +e
  "$TCC/bin/tcc" -c -I . -I "$MUSL/include" \
    -D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 \
    -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 \
    -D TCC_VERSION=\"0.9.27-decl0-diag\" -D ONE_SOURCE=1 \
    -D CONFIG_TCCDIR="$TCC/lib/tcc" -D CONFIG_TCC_CRTPREFIX="$MUSL/lib" \
    -D CONFIG_TCC_ELFINTERP="/lib/ld-musl-x86_64.so.1" \
    -D CONFIG_TCC_LIBPATHS="$MUSL/lib:$TCC/lib/tcc" \
    -D CONFIG_TCC_SYSINCLUDEPATHS="$MUSL/include" -D CONFIG_SYSROOT="/" \
    -D TCC_LIBGCC="$MUSL/lib/libc.a" -D TCC_LIBTCC1="$TCC/lib/tcc/libtcc1.a" \
    "$source" -o "$WORK/src/$name.o" >"$WORK/src/$name.out" 2>"$WORK/src/$name.err"
  local rc=$?
  set -e
  echo "diag-predecessor-verbose: source $name rc=$rc obj=$(test -f "$WORK/src/$name.o" && echo yes || echo no)"
  sed "s/^/diag-predecessor-verbose: source $name stdout: /" "$WORK/src/$name.out"
  sed "s/^/diag-predecessor-verbose: source $name stderr: /" "$WORK/src/$name.err"
}

run_tiny base -I . -I "$MUSL/include"
run_tiny boot -I . -I "$MUSL/include" \
  -D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 \
  -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 \
  -D TCC_VERSION=\"0.9.27-decl0-diag\" -D ONE_SOURCE=1
run_tiny full -I . -I "$MUSL/include" \
  -D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 \
  -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 \
  -D TCC_VERSION=\"0.9.27-decl0-diag\" -D ONE_SOURCE=1 \
  -D CONFIG_TCCDIR="$TCC/lib/tcc" -D CONFIG_TCC_CRTPREFIX="$MUSL/lib" \
  -D CONFIG_TCC_ELFINTERP="/lib/ld-musl-x86_64.so.1" \
  -D CONFIG_TCC_LIBPATHS="$MUSL/lib:$TCC/lib/tcc" \
  -D CONFIG_TCC_SYSINCLUDEPATHS="$MUSL/include" -D CONFIG_SYSROOT="/" \
  -D TCC_LIBGCC="$MUSL/lib/libc.a" -D TCC_LIBTCC1="$TCC/lib/tcc/libtcc1.a"

prep_source
printf '#include "tcc.h"\nint main(void){return 0;}\n' > h.c
printf '#include "tcc.h"\n#include "tcctools.c"\nint main(void){return 0;}\n' > h_tools.c
printf '#include "tcc.h"\n#include "libtcc.c"\nint main(void){return 0;}\n' > h_lib.c
printf '#include "tcc.h"\n#include "libtcc.c"\n#include "tcctools.c"\nint main(void){return 0;}\n' > h_lib_tools.c
run_source h_only h.c
run_source tcctools tcctools.c
run_source h_tools h_tools.c
run_source libtcc libtcc.c
run_source h_lib h_lib.c
run_source h_lib_tools h_lib_tools.c
