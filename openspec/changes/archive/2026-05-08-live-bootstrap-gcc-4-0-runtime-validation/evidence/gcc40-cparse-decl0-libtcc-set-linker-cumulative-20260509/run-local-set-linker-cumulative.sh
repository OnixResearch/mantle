#!/usr/bin/env bash
set -euo pipefail

TCC="/crunch/store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2"
MUSL="/crunch/store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl"
TCC_SRC="/crunch/store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src"
WORK=${WORK:-/tmp/libtcc-post894-prefix}
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
# Keep the already-isolated tcc_new version parse trigger bypassed.
sed -i 's/^[[:space:]]*sscanf(TCC_VERSION, "%d\.%d\.%d", &a, &b, &c);/        a = 0; b = 9; c = 27; \/\* CRUNCH diag: sscanf(TCC_VERSION) bypassed \*\//g' libtcc.asm_simplified.c

flags_common='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1'
compile_variant() {
  local label=$1 src=$2 extra=$3
  set +e
  "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $flags_common $extra "$src" -o "/tmp/$label.o" >/tmp/$label.stdout 2>/tmp/$label.stderr
  local rc=$?
  set -e
  echo "diag-libtcc-post894: compile $label $src flags=${extra:-common} rc=$rc lines=$(wc -l < "$src")"
  sed "s/^/diag-libtcc-post894: $label stderr: /" /tmp/$label.stderr || true
}


cat_body() {
  local n=$1
  cat <<'EOF_HEAD'
static int tcc_set_linker(TCCState *s, const char *option)
{
    while (*option) {
        const char *p = NULL;
        char *end = NULL;
        int ignoring = 0;
        int ret;
EOF_HEAD
  if [ "$n" -ge 1 ]; then cat <<'EOF'
        if (link_option(option, "Bsymbolic", &p)) {
            s->symbolic = 1;
EOF
  else cat <<'EOF'
        if (0) {
EOF
  fi
  [ "$n" -ge 2 ] && cat <<'EOF'
        } else if (link_option(option, "nostdlib", &p)) {
            s->nostdlib = 1;
EOF
  [ "$n" -ge 3 ] && cat <<'EOF'
        } else if (link_option(option, "fini=", &p)) {
            copy_linker_arg(&s->fini_symbol, p, 0);
            ignoring = 1;
EOF
  [ "$n" -ge 4 ] && cat <<'EOF'
        } else if (link_option(option, "image-base=", &p)
                || link_option(option, "Ttext=", &p)) {
            s->text_addr = strtoull(p, &end, 16);
            s->has_text_addr = 1;
EOF
  [ "$n" -ge 5 ] && cat <<'EOF'
        } else if (link_option(option, "init=", &p)) {
            copy_linker_arg(&s->init_symbol, p, 0);
            ignoring = 1;
EOF
  [ "$n" -ge 6 ] && cat <<'EOF'
        } else if (link_option(option, "oformat=", &p)) {
            if (strstart("elf64-", &p)) {
                s->output_format = TCC_OUTPUT_FORMAT_ELF;
            } else if (!strcmp(p, "binary")) {
                s->output_format = TCC_OUTPUT_FORMAT_BINARY;
            } else
                goto err;
EOF
  [ "$n" -ge 7 ] && cat <<'EOF'
        } else if (link_option(option, "as-needed", &p)) {
            ignoring = 1;
EOF
  [ "$n" -ge 8 ] && cat <<'EOF'
        } else if (link_option(option, "O", &p)) {
            ignoring = 1;
EOF
  [ "$n" -ge 9 ] && cat <<'EOF'
        } else if (link_option(option, "export-all-symbols", &p)) {
            s->rdynamic = 1;
EOF
  [ "$n" -ge 10 ] && cat <<'EOF'
        } else if (link_option(option, "rpath=", &p)) {
            copy_linker_arg(&s->rpath, p, ':');
EOF
  [ "$n" -ge 11 ] && cat <<'EOF'
        } else if (link_option(option, "enable-new-dtags", &p)) {
            s->enable_new_dtags = 1;
EOF
  [ "$n" -ge 12 ] && cat <<'EOF'
        } else if (link_option(option, "section-alignment=", &p)) {
            s->section_align = strtoul(p, &end, 16);
EOF
  [ "$n" -ge 13 ] && cat <<'EOF'
        } else if (link_option(option, "soname=", &p)) {
            copy_linker_arg(&s->soname, p, 0);
EOF
  [ "$n" -ge 14 ] && cat <<'EOF'
        } else if (ret = link_option(option, "?whole-archive", &p), ret) {
            s->alacarte_link = ret < 0;
EOF
  cat <<'EOF_TAIL'
        } else if (p) {
            return 0;
        } else {
    err:
            tcc_error("unsupported linker option '%s'", option);
        }
        if (ignoring && s->warn_unsupported)
            tcc_warning("unsupported linker option '%s'", option);
        option = skip_linker_arg(&p);
    }
    return 1;
}
EOF_TAIL
}

for extra in '' '-D ONE_SOURCE=1'; do
  suffix=${extra//[^A-Za-z0-9]/_}
  for n in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14; do
    src=cumulative_${n}${suffix}.c
    awk 'NR <= 1322 { print }' libtcc.asm_simplified.c > "$src"
    cat_body "$n" >> "$src"
    compile_variant cumulative_${n}${suffix:-_common} "$src" "$extra"
  done
done
