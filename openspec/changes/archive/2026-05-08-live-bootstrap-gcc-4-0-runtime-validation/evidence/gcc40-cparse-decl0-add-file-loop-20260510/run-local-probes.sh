#!/usr/bin/env bash
set -euo pipefail
ROOT=${ROOT:-/home/brittonr/git/crunch/crunch}
STORE="$ROOT/.crunch-drain/post621-restored-store"
TCC="$STORE/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2"
MUSL="$STORE/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl"
TCC_SRC="$STORE/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src"
WORK=${WORK:-/tmp/tcc-decl0-add-file-local}
rm -rf "$WORK"
cp -R "$TCC_SRC" "$WORK"
chmod -R u+w "$WORK"
cd "$WORK"
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
    char buf[1024];\
    int i;\
    if (!strcmp(filename, "crti.o") || !strcmp(filename, "crtn.o"))\
        return 0;\
    for (i = 0; i < s->nb_crt_paths; i++) {\
        strcpy(buf, s->crt_paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        if (tcc_add_file_internal(s, buf, AFF_TYPE_BIN) == 0)\
            return 0;\
    }\
    return 0; /* CRUNCH diag: suppress missing crt error in predecessor build */;\
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
awk '
  /^static int decl0\(int l, int is_for_loop_init, Sym \*func_sym\)/ { in_decl0=1 }
  { print; if (in_decl0 && $0 ~ /^[[:space:]]*while \(1\) \{/) print "        fputs(\"diag-tcc-decl0-runtime: enter\\n\", stderr);"; if (in_decl0 && $0 ~ /^}/) in_decl0=0 }
' tccgen.c > tccgen.c.tmp
mv tccgen.c.tmp tccgen.c
awk '
  {
    print
    if ($0 ~ /^ST_FUNC int tcc_open\(TCCState \*s1, const char \*filename\)/) in_open = 1
    if (in_open && $0 ~ /^[[:space:]]*int fd;/) { print "    fputs(\"diag-tcc-file: tcc_open enter\\n\", stderr);"; in_open = 0 }
    if ($0 ~ /^ST_FUNC int tcc_add_file_internal\(TCCState \*s1, const char \*filename, int flags\)/) in_internal = 1
    if (in_internal && $0 ~ /^[[:space:]]*int ret;/) { print "    fputs(\"diag-tcc-file: add_file_internal enter\\n\", stderr);"; in_internal = 0 }
    if ($0 ~ /^LIBTCCAPI int tcc_add_file\(TCCState \*s, const char \*filename\)/) in_public = 1
    if (in_public && $0 ~ /^[[:space:]]*int filetype = s->filetype;/) { print "    fputs(\"diag-tcc-file: add_file public enter\\n\", stderr);"; in_public = 0 }
  }
' libtcc.c > libtcc.c.tmp
mv libtcc.c.tmp libtcc.c
awk '
  { if ($0 ~ /if \(tcc_add_file\(s, f->name\) < 0\)/) print "            fputs(\"diag-tcc-file: before tcc_add_file loop call\\n\", stderr);"; print }
' tcc.c > tcc.c.tmp
mv tcc.c.tmp tcc.c
flags='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION="0.9.27-decl0-diag" -D ONE_SOURCE=1'
run_phase() {
  local phase=$1
  shift
  echo "diag-tcc-add-file-phase: $phase"
  set +e
  "$@" >/tmp/tcc-add-file-$phase.stdout 2>/tmp/tcc-add-file-$phase.stderr
  local rc=$?
  set -e
  echo "diag-tcc-add-file-phase: $phase rc=$rc"
  sed "s/^/diag-tcc-add-file-phase: $phase stdout: /" /tmp/tcc-add-file-$phase.stdout || true
  sed "s/^/diag-tcc-add-file-phase: $phase stderr: /" /tmp/tcc-add-file-$phase.stderr || true
}
run_phase compile-only-quiet "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $flags \
  -D CONFIG_TCCDIR="$TCC/lib/tcc" \
  -D CONFIG_TCC_CRTPREFIX="$MUSL/lib" \
  -D CONFIG_TCC_ELFINTERP="/lib/ld-musl-x86_64.so.1" \
  -D CONFIG_TCC_LIBPATHS="$MUSL/lib:$TCC/lib/tcc" \
  -D CONFIG_TCC_SYSINCLUDEPATHS="$MUSL/include" \
  -D CONFIG_SYSROOT="/" -D TCC_LIBGCC="$MUSL/lib/libc.a" -D TCC_LIBTCC1="$TCC/lib/tcc/libtcc1.a" \
  tcc.c -o /tmp/tcc-add-file-instrumented.o
run_phase compile-only-verbose "$TCC/bin/tcc" -c -v -I . -I "$MUSL/include" $flags \
  -D CONFIG_TCCDIR="$TCC/lib/tcc" \
  -D CONFIG_TCC_CRTPREFIX="$MUSL/lib" \
  -D CONFIG_TCC_ELFINTERP="/lib/ld-musl-x86_64.so.1" \
  -D CONFIG_TCC_LIBPATHS="$MUSL/lib:$TCC/lib/tcc" \
  -D CONFIG_TCC_SYSINCLUDEPATHS="$MUSL/include" \
  -D CONFIG_SYSROOT="/" -D TCC_LIBGCC="$MUSL/lib/libc.a" -D TCC_LIBTCC1="$TCC/lib/tcc/libtcc1.a" \
  tcc.c -o /tmp/tcc-add-file-instrumented.o
run_phase link-object-default "$TCC/bin/tcc" -v -static -o /tmp/tcc-add-file-from-object /tmp/tcc-add-file-instrumented.o
run_phase link-object-explicit-archives "$TCC/bin/tcc" -v -static -o /tmp/tcc-add-file-from-object-explicit /tmp/tcc-add-file-instrumented.o "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a"
for input in tcc.c /tmp/tcc-add-file-instrumented.o "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a" "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" "$MUSL/lib/crtn.o"; do
  if [ -e "$input" ]; then
    echo "diag-tcc-add-file-phase: input-exists $input"
  else
    echo "diag-tcc-add-file-phase: input-missing $input"
  fi
done
