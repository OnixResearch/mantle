#!/usr/bin/env bash
set -euo pipefail
ROOT=${ROOT:-/home/brittonr/git/crunch/crunch}
STORE="$ROOT/.crunch-drain/post621-restored-store"
TCC="$STORE/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2"
MUSL="$STORE/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl"
TCC_SRC="$STORE/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src"
WORK=${WORK:-/tmp/tcc-post621-local-probes}
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
    tcc_error_noabort("file '\''%s'\'' not found", filename);\
    return 0;\
}' libtcc.c
sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
flags_common='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1'
make_probe() {
  local label=$1 mode=$2 dst="libtcc-post621-declaration-$label.c"
  sed '/va_start(ap, fmt);/d; /va_end(ap);/d' libtcc.c | awk 'NR <= 621 { print }' > "$dst"
  case "$mode" in
    prefix_only) ;;
    int_global) echo 'int crunch_diag_post621_global;' >> "$dst" ;;
    int_extern) echo 'extern int crunch_diag_post621_extern;' >> "$dst" ;;
    typedef_int) echo 'typedef int crunch_diag_post621_typedef;' >> "$dst" ;;
    static_func_prototype_void) echo 'static int crunch_diag_post621_func(void);' >> "$dst" ;;
    static_func_prototype_tccstate) echo 'static int crunch_diag_post621_func(TCCState *s1);' >> "$dst" ;;
    static_func_empty_void) cat >> "$dst" <<'EOF_PROBE'
static int crunch_diag_post621_func(void)
{
    return 0;
}
EOF_PROBE
      ;;
    struct_ptr_global) echo 'TCCState *crunch_diag_post621_state;' >> "$dst" ;;
    *) echo "unknown mode $mode" >&2; return 1 ;;
  esac
}
run_probe() {
  local label=$1 mode=$2 extra=$3
  make_probe "$label" "$mode"
  set +e
  "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $flags_common $extra "libtcc-post621-declaration-$label.c" -o "/tmp/$label.o" >/tmp/$label.stdout 2>/tmp/$label.stderr
  rc=$?
  set -e
  echo "diag-tcc-decl0-prepart: compile libtcc_post621_declaration_${label} libtcc-post621-declaration-${label}.c flags=${extra:-common} rc=$rc"
  sed "s/^/diag-tcc-decl0-prepart: libtcc-post621-declaration-${label}.c stderr: /" /tmp/$label.stderr || true
}
for extra in '' '-D ONE_SOURCE=1'; do
  for spec in \
    prefix_only:prefix_only \
    int_global:int_global \
    int_extern:int_extern \
    typedef_int:typedef_int \
    static_func_prototype_void:static_func_prototype_void \
    static_func_prototype_tccstate:static_func_prototype_tccstate \
    static_func_empty_void:static_func_empty_void \
    struct_ptr_global:struct_ptr_global; do
      run_probe "${spec%%:*}${extra//[^A-Za-z0-9]/_}" "${spec#*:}" "$extra"
  done
done
