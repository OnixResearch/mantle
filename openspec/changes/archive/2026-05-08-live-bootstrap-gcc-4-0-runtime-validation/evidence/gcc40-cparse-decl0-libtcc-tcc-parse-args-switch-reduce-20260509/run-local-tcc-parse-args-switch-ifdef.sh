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
  python3 - "$TCC/bin/tcc" "$MUSL/include" "$flags_common" "$label" "$src" "$extra" <<'PY'
import subprocess, sys
from pathlib import Path
tcc, musl_inc, flags_common, label, src, extra = sys.argv[1:]
cmd = [tcc, '-c', '-I', '.', '-I', musl_inc]
cmd.extend(flags_common.split())
if extra:
    cmd.extend(extra.split())
cmd.extend([src, '-o', f'/tmp/{label}.o'])
res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
Path(f'/tmp/{label}.stdout').write_bytes(res.stdout)
Path(f'/tmp/{label}.stderr').write_bytes(res.stderr)
line_count = sum(1 for _ in open(src, 'rb'))
print(f"diag-libtcc-post-options-m: compile {label} {src} flags={extra or 'common'} rc={res.returncode} lines={line_count}")
if res.stderr:
    for line in res.stderr.decode('utf-8', errors='replace').splitlines():
        print(f"diag-libtcc-post-options-m: {label} stderr: {line}")
PY
  return 0
}

python3 - <<'PY'
from pathlib import Path
p = Path('libtcc.asm_simplified.c')
lines = p.read_text().splitlines()
out=[]; removed=0; skipped_ifdef=0; i=0
while i < len(lines):
    line = lines[i]
    if 'copy_linker_arg(&s->' in line:
        out.append('            /* CRUNCH diag: copy_linker_arg field call removed */')
        removed += 1
        i += 1
        continue
    # Normalize the known options_m preprocessor island by removing only the guard lines.
    if line.strip() == '#ifdef TCC_TARGET_X86_64' and i > 0 and 'ms_bitfields' in lines[i-1]:
        out.append('/* CRUNCH diag: options_m TCC_TARGET_X86_64 guard removed */')
        i += 1
        while i < len(lines) and lines[i].strip() != '#endif':
            out.append(lines[i])
            i += 1
        if i < len(lines) and lines[i].strip() == '#endif':
            out.append('/* CRUNCH diag: end removed options_m guard */')
            i += 1
        skipped_ifdef += 1
        continue
    out.append(line)
    i += 1
p.write_text('\n'.join(out)+'\n')
print(f'diag-libtcc-post-options-m: removed_copy_linker_arg_field_calls={removed}')
print(f'diag-libtcc-post-options-m: normalized_options_m_ifdef_islands={skipped_ifdef}')
PY


python3 - <<'PY'
from pathlib import Path
src=Path('libtcc.asm_simplified.c').read_text().splitlines()
def find_idx(needle, start=0):
    for i in range(start,len(src)):
        if needle in src[i]: return i
    raise SystemExit('missing '+needle)
parse_start=find_idx('PUB_FUNC int tcc_parse_args')
switch_start=find_idx('switch(popt->index)', parse_start)
prefix=src[:parse_start]
pre_switch_func=src[parse_start:switch_start]
epilogue=['    }','    return 0;','}']
# Manual active cases through bench known-good, then targeted preprocessor islands.
through_bench=[
'        case TCC_OPTION_HELP:', '            return OPT_HELP;',
'        case TCC_OPTION_HELP2:', '            return OPT_HELP2;',
'        case TCC_OPTION_I:', '            tcc_add_include_path(s, optarg);', '            break;',
'        case TCC_OPTION_D:', '            parse_option_D(s, optarg);', '            break;',
'        case TCC_OPTION_U:', '            tcc_undefine_symbol(s, optarg);', '            break;',
'        case TCC_OPTION_L:', '            tcc_add_library_path(s, optarg);', '            break;',
'        case TCC_OPTION_B:', '            tcc_set_lib_path(s, optarg);', '            break;',
'        case TCC_OPTION_l:', '            args_parser_add_file(s, optarg, AFF_TYPE_LIB);', '            s->nb_libraries++;', '            break;',
'        case TCC_OPTION_pthread:', '            parse_option_D(s, "_REENTRANT");', '            s->option_pthread = 1;', '            break;',
'        case TCC_OPTION_bench:', '            s->do_bench = 1;', '            break;',
]
bt_block=['#ifdef CONFIG_TCC_BACKTRACE','        case TCC_OPTION_bt:','            tcc_set_num_callers(atoi(optarg));','            break;','#endif']
bcheck_block=['#ifdef CONFIG_TCC_BCHECK','        case TCC_OPTION_b:','            s->do_bounds_check = 1;','            s->do_debug = 1;','            break;','#endif']
after_bcheck=[
'        case TCC_OPTION_g:', '            s->do_debug = 1;', '            break;',
'        case TCC_OPTION_c:', '            x = TCC_OUTPUT_OBJ;',
'        set_output_type:',
'            if (s->output_type)',
'                tcc_warning("-%s: overriding compiler action already specified", popt->name);',
'            s->output_type = x;',
'            break;',
]
def make(name, body):
    return name, prefix+pre_switch_func+['        switch(popt->index) {']+body+['        }']+epilogue
variants=[
 make('00_through_bench_manual', through_bench),
 make('01_through_bench_empty_bt_ifdef', through_bench+['#ifdef CONFIG_TCC_BACKTRACE','#endif']),
 make('02_through_bt_guarded', through_bench+bt_block),
 make('03_through_bcheck_guarded', through_bench+bt_block+bcheck_block),
 make('04_through_c_after_guarded', through_bench+bt_block+bcheck_block+after_bcheck),
 make('05_bt_block_only', bt_block),
 make('06_bcheck_block_only', bcheck_block),
]
out=Path('tcc_parse_args_switch_ifdef_variants'); out.mkdir(exist_ok=True)
for name,lines in variants:
    safe=''.join(c if c.isalnum() else '_' for c in name)
    (out/f'{safe}.c').write_text('\n'.join(lines)+'\n')
print('diag-libtcc-tcc-parse-args-switch-ifdef: generated_variants=' + str(len(variants)))
for name,lines in variants: print(f'diag-libtcc-tcc-parse-args-switch-ifdef: variant {name} lines={len(lines)}')
PY
for extra in '' '-D ONE_SOURCE=1'; do
  suffix=${extra//[^A-Za-z0-9]/_}
  for src in tcc_parse_args_switch_ifdef_variants/*.c; do
    label="tcc_parse_args_switch_ifdef_${src##*/}${suffix:-_common}"
    label=${label%.c}
    compile_variant "$label" "$src" "$extra"
  done
done
