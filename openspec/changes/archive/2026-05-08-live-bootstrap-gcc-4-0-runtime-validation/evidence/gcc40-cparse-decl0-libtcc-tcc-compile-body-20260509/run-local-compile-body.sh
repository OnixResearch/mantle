#!/usr/bin/env bash
set -euo pipefail

TCC="/crunch/store/b557z7jmjqnlz118c8zvwdjvkv4v5qb3-tcc-0.9.27-musl-v2"
MUSL="/crunch/store/g38fpdz6wrs7n2p5f3afqbcb3w4s2m64-musl-1.1.24-tcc-musl"
TCC_SRC="/crunch/store/4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src"
WORK=${WORK:-/tmp/tcc-compile-body-reduce}
rm -rf "$WORK"
cp -R "$TCC_SRC" "$WORK"
chmod -R u+w "$WORK"
cd "$WORK"
: > config.h

# Same source normalizations/prerequisite context as the archived diagnostic,
# including the native387-disabled tccgen.c mutation required before libtcc
# paired-cleanup prefixes are meaningful.
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

flags_common='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1'
make_base() {
  local dst=$1
  awk 'NR <= 621 { print }' libtcc.c > "$dst"
  sed -i 's/^[[:space:]]*va_start(ap, fmt);/    \/\* CRUNCH diag: va_start removed with va_end pair \*\//g; s/^[[:space:]]*va_end(ap);/    \/\* CRUNCH diag: va_end removed with va_start pair \*\//g' "$dst"
}
compile_variant() {
  local label=$1 src=$2 extra=$3
  set +e
  "$TCC/bin/tcc" -c -I . -I "$MUSL/include" $flags_common $extra "$src" -o "/tmp/$label.o" >/tmp/$label.stdout 2>/tmp/$label.stderr
  local rc=$?
  set -e
  echo "diag-tcc-compile-body: compile $label $src flags=${extra:-common} rc=$rc lines=$(wc -l < "$src")"
  sed "s/^/diag-tcc-compile-body: $label stderr: /" /tmp/$label.stderr || true
}
append_body() {
  local dst=$1 body=$2
  cat >> "$dst" <<EOF_BODY

/* compile the file opened in 'file'. Return non zero if errors. */
static int tcc_compile(TCCState *s1)
{
$body
}
EOF_BODY
}

for extra in '' '-D ONE_SOURCE=1'; do
  suffix=${extra//[^A-Za-z0-9]/_}
  make_base prefix621${suffix}.c
  compile_variant prefix621${suffix:-_common} prefix621${suffix}.c "$extra"

  cp prefix621${suffix}.c proto${suffix}.c
  echo 'static int tcc_compile(TCCState *s1);' >> proto${suffix}.c
  compile_variant proto${suffix:-_common} proto${suffix}.c "$extra"

  cp prefix621${suffix}.c empty${suffix}.c
  append_body empty${suffix}.c '    return 0;'
  compile_variant empty${suffix:-_common} empty${suffix}.c "$extra"

  cp prefix621${suffix}.c locals${suffix}.c
  append_body locals${suffix}.c '    Sym *define_start;
    int filetype, is_asm;
    define_start = define_stack;
    filetype = s1->filetype;
    is_asm = filetype == AFF_TYPE_ASM || filetype == AFF_TYPE_ASMPP;
    return 0;'
  compile_variant locals${suffix:-_common} locals${suffix}.c "$extra"

  cp prefix621${suffix}.c begin${suffix}.c
  append_body begin${suffix}.c '    Sym *define_start;
    int filetype, is_asm;
    define_start = define_stack;
    filetype = s1->filetype;
    is_asm = filetype == AFF_TYPE_ASM || filetype == AFF_TYPE_ASMPP;
    tccelf_begin_file(s1);
    return 0;'
  compile_variant begin${suffix:-_common} begin${suffix}.c "$extra"

  cp prefix621${suffix}.c setjmp_empty${suffix}.c
  append_body setjmp_empty${suffix}.c '    if (setjmp(s1->error_jmp_buf) == 0) {
    }
    return 0;'
  compile_variant setjmp_empty${suffix:-_common} setjmp_empty${suffix}.c "$extra"

  cp prefix621${suffix}.c setjmp_flags${suffix}.c
  append_body setjmp_flags${suffix}.c '    if (setjmp(s1->error_jmp_buf) == 0) {
        s1->nb_errors = 0;
        s1->error_set_jmp_enabled = 1;
    }
    s1->error_set_jmp_enabled = 0;
    return 0;'
  compile_variant setjmp_flags${suffix:-_common} setjmp_flags${suffix}.c "$extra"

  cp prefix621${suffix}.c full_no_codegen${suffix}.c
  append_body full_no_codegen${suffix}.c '    Sym *define_start;
    int filetype, is_asm;
    define_start = define_stack;
    filetype = s1->filetype;
    is_asm = filetype == AFF_TYPE_ASM || filetype == AFF_TYPE_ASMPP;
    tccelf_begin_file(s1);
    if (setjmp(s1->error_jmp_buf) == 0) {
        s1->nb_errors = 0;
        s1->error_set_jmp_enabled = 1;
        preprocess_start(s1, is_asm);
    }
    s1->error_set_jmp_enabled = 0;
    preprocess_end(s1);
    free_inline_functions(s1);
    free_defines(define_start);
    sym_pop(&global_stack, NULL, 0);
    sym_pop(&local_stack, NULL, 0);
    tccelf_end_file(s1);
    return s1->nb_errors != 0 ? -1 : 0;'
  compile_variant full_no_codegen${suffix:-_common} full_no_codegen${suffix}.c "$extra"

  cp prefix621${suffix}.c direct_tccgen${suffix}.c
  append_body direct_tccgen${suffix}.c '    tccgen_compile(s1);
    return 0;'
  compile_variant direct_tccgen${suffix:-_common} direct_tccgen${suffix}.c "$extra"

  cp prefix621${suffix}.c preprocess_branch${suffix}.c
  append_body preprocess_branch${suffix}.c '    int filetype, is_asm;
    filetype = s1->filetype;
    is_asm = filetype == AFF_TYPE_ASM || filetype == AFF_TYPE_ASMPP;
    preprocess_start(s1, is_asm);
    if (s1->output_type == TCC_OUTPUT_PREPROCESS) {
        tcc_preprocess(s1);
    }
    return 0;'
  compile_variant preprocess_branch${suffix:-_common} preprocess_branch${suffix}.c "$extra"

  cp prefix621${suffix}.c asm_branch${suffix}.c
  append_body asm_branch${suffix}.c '    int filetype, is_asm;
    filetype = s1->filetype;
    is_asm = filetype == AFF_TYPE_ASM || filetype == AFF_TYPE_ASMPP;
    if (is_asm) {
        tcc_error_noabort("asm not supported");
    }
    return 0;'
  compile_variant asm_branch${suffix:-_common} asm_branch${suffix}.c "$extra"

  cp prefix621${suffix}.c else_tccgen_branch${suffix}.c
  append_body else_tccgen_branch${suffix}.c '    int filetype, is_asm;
    filetype = s1->filetype;
    is_asm = filetype == AFF_TYPE_ASM || filetype == AFF_TYPE_ASMPP;
    if (s1->output_type == TCC_OUTPUT_PREPROCESS) {
        tcc_preprocess(s1);
    } else if (is_asm) {
        tcc_error_noabort("asm not supported");
    } else {
        tccgen_compile(s1);
    }
    return 0;'
  compile_variant else_tccgen_branch${suffix:-_common} else_tccgen_branch${suffix}.c "$extra"

  cp prefix621${suffix}.c full_branch_no_asm_ifdef${suffix}.c
  append_body full_branch_no_asm_ifdef${suffix}.c '    Sym *define_start;
    int filetype, is_asm;
    define_start = define_stack;
    filetype = s1->filetype;
    is_asm = filetype == AFF_TYPE_ASM || filetype == AFF_TYPE_ASMPP;
    tccelf_begin_file(s1);
    if (setjmp(s1->error_jmp_buf) == 0) {
        s1->nb_errors = 0;
        s1->error_set_jmp_enabled = 1;
        preprocess_start(s1, is_asm);
        if (s1->output_type == TCC_OUTPUT_PREPROCESS) {
            tcc_preprocess(s1);
        } else if (is_asm) {
            tcc_error_noabort("asm not supported");
        } else {
            tccgen_compile(s1);
        }
    }
    s1->error_set_jmp_enabled = 0;
    preprocess_end(s1);
    free_inline_functions(s1);
    free_defines(define_start);
    sym_pop(&global_stack, NULL, 0);
    sym_pop(&local_stack, NULL, 0);
    tccelf_end_file(s1);
    return s1->nb_errors != 0 ? -1 : 0;'
  compile_variant full_branch_no_asm_ifdef${suffix:-_common} full_branch_no_asm_ifdef${suffix}.c "$extra"

  cp prefix621${suffix}.c full_branch_no_return_ternary${suffix}.c
  append_body full_branch_no_return_ternary${suffix}.c '    Sym *define_start;
    int filetype, is_asm;
    define_start = define_stack;
    filetype = s1->filetype;
    is_asm = filetype == AFF_TYPE_ASM || filetype == AFF_TYPE_ASMPP;
    tccelf_begin_file(s1);
    if (setjmp(s1->error_jmp_buf) == 0) {
        s1->nb_errors = 0;
        s1->error_set_jmp_enabled = 1;
        preprocess_start(s1, is_asm);
        if (s1->output_type == TCC_OUTPUT_PREPROCESS) {
            tcc_preprocess(s1);
        } else if (is_asm) {
            tcc_error_noabort("asm not supported");
        } else {
            tccgen_compile(s1);
        }
    }
    s1->error_set_jmp_enabled = 0;
    preprocess_end(s1);
    free_inline_functions(s1);
    free_defines(define_start);
    sym_pop(&global_stack, NULL, 0);
    sym_pop(&local_stack, NULL, 0);
    tccelf_end_file(s1);
    if (s1->nb_errors != 0) return -1;
    return 0;'
  compile_variant full_branch_no_return_ternary${suffix:-_common} full_branch_no_return_ternary${suffix}.c "$extra"

  cp prefix621${suffix}.c full_original${suffix}.c
  awk 'NR >= 623 && NR <= 661 { print }' libtcc.c >> full_original${suffix}.c
  compile_variant full_original${suffix:-_common} full_original${suffix}.c "$extra"

  cp libtcc.c full_libtcc_no_compile_asm_ifdef${suffix}.c
  awk 'NR == 642 { print "            tcc_error_noabort(\"asm not supported\");"; skip = 1; next } skip && NR <= 646 { next } { print }' libtcc.c > full_libtcc_no_compile_asm_ifdef${suffix}.c
  compile_variant full_libtcc_no_compile_asm_ifdef${suffix:-_common} full_libtcc_no_compile_asm_ifdef${suffix}.c "$extra"
done
