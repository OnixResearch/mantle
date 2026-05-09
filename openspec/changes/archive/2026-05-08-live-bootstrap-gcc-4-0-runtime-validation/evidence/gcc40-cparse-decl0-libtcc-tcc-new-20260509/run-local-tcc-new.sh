#!/usr/bin/env bash
set -euo pipefail

TCC="/crunch/store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2"
MUSL="/crunch/store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl"
TCC_SRC="/crunch/store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src"
WORK=${WORK:-/tmp/libtcc-post661-prefix}
ulimit -c 0 || true
rm -rf "$WORK"
cp -R "$TCC_SRC" "$WORK"
chmod -R u+w "$WORK"
cd "$WORK"
: > config.h

# Same source normalizations/prerequisite context as the archived diagnostics.
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

# Keep the already-isolated tcc_compile CONFIG_TCC_ASM island simplified, and
# preserve the paired va_start/va_end cleanup used by the line-621 prefix rail.
awk 'NR == 642 { print "            tcc_error_noabort(\"asm not supported\");"; skip = 1; next } skip && NR <= 646 { print "            /* CRUNCH diag: CONFIG_TCC_ASM island removed */"; next } { print }' libtcc.c > libtcc.asm_simplified.c
sed -i 's/^[[:space:]]*va_start(ap, fmt);/    \/\* CRUNCH diag: va_start removed with va_end pair \*\//g; s/^[[:space:]]*va_end(ap);/    \/\* CRUNCH diag: va_end removed with va_start pair \*\//g' libtcc.asm_simplified.c

flags_common='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1'
compile_variant() {
  local label=$1 src=$2 extra=$3
  set +e
  "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $flags_common $extra "$src" -o "/tmp/$label.o" >/tmp/$label.stdout 2>/tmp/$label.stderr
  local rc=$?
  set -e
  echo "diag-libtcc-post661: compile $label $src flags=${extra:-common} rc=$rc lines=$(wc -l < "$src")"
  sed "s/^/diag-libtcc-post661: $label stderr: /" /tmp/$label.stderr || true
}


append_tcc_new() {
  local dst=$1 body=$2
  cat >> "$dst" <<EOF_BODY

LIBTCCAPI TCCState *tcc_new(void)
{
$body
}
EOF_BODY
}

for extra in '' '-D ONE_SOURCE=1'; do
  suffix=${extra//[^A-Za-z0-9]/_}
  base=base723${suffix}.c
  awk 'NR <= 723 { print }' libtcc.asm_simplified.c > "$base"
  compile_variant base723${suffix:-_common} "$base" "$extra"

  cp "$base" proto${suffix}.c
  echo 'LIBTCCAPI TCCState *tcc_new(void);' >> proto${suffix}.c
  compile_variant proto${suffix:-_common} proto${suffix}.c "$extra"

  cp "$base" empty${suffix}.c
  append_tcc_new empty${suffix}.c '    return NULL;'
  compile_variant empty${suffix:-_common} empty${suffix}.c "$extra"

  cp "$base" alloc${suffix}.c
  append_tcc_new alloc${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    return s;'
  compile_variant alloc${suffix:-_common} alloc${suffix}.c "$extra"

  cp "$base" defaults${suffix}.c
  append_tcc_new defaults${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    s->alacarte_link = 1;
    s->nocommon = 1;
    s->warn_implicit_function_declaration = 1;
    s->ms_extensions = 1;
    return s;'
  compile_variant defaults${suffix:-_common} defaults${suffix}.c "$extra"

  cp "$base" libpath${suffix}.c
  append_tcc_new libpath${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    s->alacarte_link = 1;
    s->nocommon = 1;
    s->warn_implicit_function_declaration = 1;
    s->ms_extensions = 1;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    return s;'
  compile_variant libpath${suffix:-_common} libpath${suffix}.c "$extra"

  cp "$base" dummy_defines${suffix}.c
  append_tcc_new dummy_defines${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    define_push(TOK___LINE__, MACRO_OBJ, NULL, NULL);
    define_push(TOK___FILE__, MACRO_OBJ, NULL, NULL);
    define_push(TOK___DATE__, MACRO_OBJ, NULL, NULL);
    define_push(TOK___TIME__, MACRO_OBJ, NULL, NULL);
    define_push(TOK___COUNTER__, MACRO_OBJ, NULL, NULL);
    return s;'
  compile_variant dummy_defines${suffix:-_common} dummy_defines${suffix}.c "$extra"

  cp "$base" version_decls${suffix}.c
  append_tcc_new version_decls${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    {
        char buffer[32]; int a,b,c;
    }
    return s;'
  compile_variant version_decls${suffix:-_common} version_decls${suffix}.c "$extra"

  cp "$base" version_sscanf${suffix}.c
  append_tcc_new version_sscanf${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    {
        char buffer[32]; int a,b,c;
        sscanf(TCC_VERSION, "%d.%d.%d", &a, &b, &c);
    }
    return s;'
  compile_variant version_sscanf${suffix:-_common} version_sscanf${suffix}.c "$extra"

  cp "$base" version_sprintf_const${suffix}.c
  append_tcc_new version_sprintf_const${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    {
        char buffer[32]; int a,b,c;
        sprintf(buffer, "%d", 927);
    }
    return s;'
  compile_variant version_sprintf_const${suffix:-_common} version_sprintf_const${suffix}.c "$extra"

  cp "$base" version_sprintf_expr${suffix}.c
  append_tcc_new version_sprintf_expr${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    {
        char buffer[32]; int a,b,c;
        sprintf(buffer, "%d", a*10000 + b*100 + c);
    }
    return s;'
  compile_variant version_sprintf_expr${suffix:-_common} version_sprintf_expr${suffix}.c "$extra"

  cp "$base" version_define_only${suffix}.c
  append_tcc_new version_define_only${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    {
        char buffer[32];
        tcc_define_symbol(s, "__TINYC__", buffer);
    }
    return s;'
  compile_variant version_define_only${suffix:-_common} version_define_only${suffix}.c "$extra"

  cp "$base" tinyc_version${suffix}.c
  append_tcc_new tinyc_version${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    {
        char buffer[32]; int a,b,c;
        sscanf(TCC_VERSION, "%d.%d.%d", &a, &b, &c);
        sprintf(buffer, "%d", a*10000 + b*100 + c);
        tcc_define_symbol(s, "__TINYC__", buffer);
    }
    return s;'
  compile_variant tinyc_version${suffix:-_common} tinyc_version${suffix}.c "$extra"

  cp "$base" standard_defines${suffix}.c
  append_tcc_new standard_defines${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    tcc_define_symbol(s, "__STDC__", NULL);
    tcc_define_symbol(s, "__STDC_VERSION__", "199901L");
    tcc_define_symbol(s, "__STDC_HOSTED__", NULL);
    return s;'
  compile_variant standard_defines${suffix:-_common} standard_defines${suffix}.c "$extra"

  cp "$base" target_x86${suffix}.c
  append_tcc_new target_x86${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    tcc_define_symbol(s, "__x86_64__", NULL);
    tcc_define_symbol(s, "__unix__", NULL);
    tcc_define_symbol(s, "__unix", NULL);
    tcc_define_symbol(s, "unix", NULL);
    return s;'
  compile_variant target_x86${suffix:-_common} target_x86${suffix}.c "$extra"

  cp "$base" builtin_redirect${suffix}.c
  append_tcc_new builtin_redirect${suffix}.c '    TCCState *s;
    tcc_cleanup();
    s = tcc_mallocz(sizeof(TCCState));
    if (!s)
        return NULL;
    tcc_state = s;
    ++nb_states;
    tcc_set_lib_path(s, CONFIG_TCCDIR);
    tccelf_new(s);
    tccpp_new(s);
    tcc_define_symbol(s, "__REDIRECT(name, proto, alias)",
        "name proto __asm__ (#alias)");
    tcc_define_symbol(s, "__REDIRECT_NTH(name, proto, alias)",
        "name proto __asm__ (#alias) __THROW");
    return s;'
  compile_variant builtin_redirect${suffix:-_common} builtin_redirect${suffix}.c "$extra"

  cp "$base" full_tcc_new${suffix}.c
  awk 'NR >= 724 && NR <= 894 { print }' libtcc.asm_simplified.c >> full_tcc_new${suffix}.c
  compile_variant full_tcc_new${suffix:-_common} full_tcc_new${suffix}.c "$extra"
done
