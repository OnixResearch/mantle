#!/usr/bin/env bash
set -euo pipefail

TCC="/crunch/store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2"
MUSL="/crunch/store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl"
TCC_SRC="/crunch/store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src"
WORK=${WORK:-/tmp/libtcc-compile-string-reduce}
ulimit -c 0 || true
rm -rf "$WORK"
cp -R "$TCC_SRC" "$WORK"
chmod -R u+w "$WORK"
cd "$WORK"
: > config.h

# Same source normalizations/prerequisite context as the post661 prefix replay.
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
cp tccgen.c tccgen.c.with-native387
awk 'done == 0 && $0 == "#if defined TCC_IS_NATIVE_387" { print "#if 0 /* CRUNCH diag: predecessor tccgen native x87 compile boundary */"; done = 1; next } { print }' tccgen.c.with-native387 > tccgen.c
awk 'NR == 642 { print "            tcc_error_noabort(\"asm not supported\");"; skip = 1; next } skip && NR <= 646 { print "            /* CRUNCH diag: CONFIG_TCC_ASM island removed */"; next } { print }' libtcc.c > libtcc.asm_simplified.c
sed -i 's/^[[:space:]]*va_start(ap, fmt);/    \/\* CRUNCH diag: va_start removed with va_end pair \*\//g; s/^[[:space:]]*va_end(ap);/    \/\* CRUNCH diag: va_end removed with va_start pair \*\//g' libtcc.asm_simplified.c

flags_common='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1'
compile_variant() {
  local label=$1 src=$2 extra=$3
  set +e
  "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $flags_common $extra "$src" -o "/tmp/$label.o" >/tmp/$label.stdout 2>/tmp/$label.stderr
  local rc=$?
  set -e
  echo "diag-libtcc-compile-string: compile $label $src flags=${extra:-common} rc=$rc lines=$(wc -l < "$src")"
  sed "s/^/diag-libtcc-compile-string: $label stderr: /" /tmp/$label.stderr || true
}
append_compile_string() {
  local dst=$1 body=$2
  cat >> "$dst" <<EOF_BODY

LIBTCCAPI int tcc_compile_string(TCCState *s, const char *str)
{
$body
}
EOF_BODY
}

for extra in '' '-D ONE_SOURCE=1'; do
  suffix=${extra//[^A-Za-z0-9]/_}
  base=base662${suffix}.c
  awk 'NR <= 662 { print }' libtcc.asm_simplified.c > "$base"
  compile_variant base662${suffix:-_common} "$base" "$extra"

  cp "$base" proto${suffix}.c
  echo 'LIBTCCAPI int tcc_compile_string(TCCState *s, const char *str);' >> proto${suffix}.c
  compile_variant proto${suffix:-_common} proto${suffix}.c "$extra"

  cp "$base" empty${suffix}.c
  append_compile_string empty${suffix}.c '    return 0;'
  compile_variant empty${suffix:-_common} empty${suffix}.c "$extra"

  cp "$base" locals${suffix}.c
  append_compile_string locals${suffix}.c '    int len, ret;
    len = strlen(str);
    ret = 0;
    return ret;'
  compile_variant locals${suffix:-_common} locals${suffix}.c "$extra"

  cp "$base" open_only${suffix}.c
  append_compile_string open_only${suffix}.c '    int len;
    len = strlen(str);
    tcc_open_bf(s, "<string>", len);
    return 0;'
  compile_variant open_only${suffix:-_common} open_only${suffix}.c "$extra"

  cp "$base" open_memcpy${suffix}.c
  append_compile_string open_memcpy${suffix}.c '    int len;
    len = strlen(str);
    tcc_open_bf(s, "<string>", len);
    memcpy(file->buffer, str, len);
    return 0;'
  compile_variant open_memcpy${suffix:-_common} open_memcpy${suffix}.c "$extra"

  cp "$base" call_compile${suffix}.c
  append_compile_string call_compile${suffix}.c '    int len, ret;
    len = strlen(str);
    tcc_open_bf(s, "<string>", len);
    memcpy(file->buffer, str, len);
    ret = tcc_compile(s);
    return ret;'
  compile_variant call_compile${suffix:-_common} call_compile${suffix}.c "$extra"

  cp "$base" full_compile_string${suffix}.c
  awk 'NR >= 664 && NR <= 674 { print }' libtcc.asm_simplified.c >> full_compile_string${suffix}.c
  compile_variant full_compile_string${suffix:-_common} full_compile_string${suffix}.c "$extra"
done
