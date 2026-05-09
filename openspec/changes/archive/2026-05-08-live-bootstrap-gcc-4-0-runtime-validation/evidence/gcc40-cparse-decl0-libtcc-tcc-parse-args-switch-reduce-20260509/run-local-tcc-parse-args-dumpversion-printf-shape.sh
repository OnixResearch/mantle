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
sed -i '/sprintf(buf,/c\    buf[0] = 34; strcpy(buf + 1, file->filename); strcat(buf, "\"");' tccpp.c
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
def find_idx(needle,start=0):
    for i in range(start,len(src)):
        if needle in src[i]: return i
    raise SystemExit('missing '+needle)
parse_start=find_idx('PUB_FUNC int tcc_parse_args')
switch_start=find_idx('switch(popt->index)', parse_start)
prefix=src[:parse_start]
pre_switch_func=src[parse_start:switch_start]
body="""        case TCC_OPTION_HELP:
            return OPT_HELP;
        case TCC_OPTION_HELP2:
            return OPT_HELP2;
        case TCC_OPTION_I:
            tcc_add_include_path(s, optarg);
            break;
        case TCC_OPTION_D:
            parse_option_D(s, optarg);
            break;
        case TCC_OPTION_U:
            tcc_undefine_symbol(s, optarg);
            break;
        case TCC_OPTION_L:
            tcc_add_library_path(s, optarg);
            break;
        case TCC_OPTION_B:
            tcc_set_lib_path(s, optarg);
            break;
        case TCC_OPTION_l:
            args_parser_add_file(s, optarg, AFF_TYPE_LIB);
            s->nb_libraries++;
            break;
        case TCC_OPTION_pthread:
parse_option_D(s, "_REENTRANT");
            s->option_pthread = 1;
            break;
        case TCC_OPTION_bench:
            s->do_bench = 1;
            break;
#ifdef CONFIG_TCC_BACKTRACE
        case TCC_OPTION_bt:
            tcc_set_num_callers(atoi(optarg));
            break;
#endif
#ifdef CONFIG_TCC_BCHECK
        case TCC_OPTION_b:
            s->do_bounds_check = 1;
            s->do_debug = 1;
            break;
#endif
        case TCC_OPTION_g:
            s->do_debug = 1;
            break;
        case TCC_OPTION_c:
            x = TCC_OUTPUT_OBJ;
        set_output_type:
            if (s->output_type)
                tcc_warning("-%s: overriding compiler action already specified", popt->name);
            s->output_type = x;
            break;
        case TCC_OPTION_d:
            if (*optarg == 'D')
                s->dflag = 3;
            else if (*optarg == 'M')
                s->dflag = 7;
            else if (*optarg == 't')
                s->dflag = 16;
            else if (isnum(*optarg))
                g_debug = atoi(optarg);
            else
                goto unsupported_option;
            break;
        case TCC_OPTION_static:
            s->static_link = 1;
            break;
        case TCC_OPTION_std:
            break;
        case TCC_OPTION_shared:
            x = TCC_OUTPUT_DLL;
            goto set_output_type;
        case TCC_OPTION_soname:
            s->soname = tcc_strdup(optarg);
            break;
        case TCC_OPTION_o:
            if (s->outfile) {
                tcc_warning("multiple -o option");
                tcc_free(s->outfile);
            }
            s->outfile = tcc_strdup(optarg);
            break;
        case TCC_OPTION_r:
            s->option_r = 1;
            x = TCC_OUTPUT_OBJ;
            goto set_output_type;
        case TCC_OPTION_isystem:
            tcc_add_sysinclude_path(s, optarg);
            break;
        case TCC_OPTION_include:
            dynarray_add(&s->cmd_include_files, &s->nb_cmd_include_files, tcc_strdup(optarg));
            break;
        case TCC_OPTION_nostdinc:
            s->nostdinc = 1;
            break;
        case TCC_OPTION_nostdlib:
            s->nostdlib = 1;
            break;
        case TCC_OPTION_run:
#ifndef TCC_IS_NATIVE
            tcc_error("-run is not available in a cross compiler");
#endif
            run = optarg;
            x = TCC_OUTPUT_MEMORY;
            goto set_output_type;
        case TCC_OPTION_v:
            do ++s->verbose; while (*optarg++ == 'v');
            ++noaction;
            break;
        case TCC_OPTION_f:
            if (set_flag(s, options_f, optarg) < 0)
                goto unsupported_option;
            break;
        case TCC_OPTION_m:
            if (set_flag(s, options_m, optarg) < 0) {
                if (x = atoi(optarg), x != 32 && x != 64)
                    goto unsupported_option;
                if (PTR_SIZE != x/8)
                    return x;
                ++noaction;
            }
            break;""".splitlines()
chunks=[
('W', ['        case TCC_OPTION_W:', '            if (set_flag(s, options_W, optarg) < 0)', '                goto unsupported_option;', '            break;']),
('w', ['        case TCC_OPTION_w:', '            s->warn_none = 1;', '            break;']),
('rdynamic', ['        case TCC_OPTION_rdynamic:', '            s->rdynamic = 1;', '            break;']),
('Wl', ['        case TCC_OPTION_Wl:', '            if (linker_arg.size)', "                --linker_arg.size, cstr_ccat(&linker_arg, ',');", '            cstr_cat(&linker_arg, optarg, 0);', '            if (tcc_set_linker(s, linker_arg.data))', '                cstr_free(&linker_arg);', '            break;']),
('Wp', ['        case TCC_OPTION_Wp:', '            r = optarg;', '            goto reparse;']),
('E', ['        case TCC_OPTION_E:', '            x = TCC_OUTPUT_PREPROCESS;', '            goto set_output_type;']),
('P', ['        case TCC_OPTION_P:', '            s->Pflag = atoi(optarg) + 1;', '            break;']),
('MD', ['        case TCC_OPTION_MD:', '            s->gen_deps = 1;', '            break;']),
('MF', ['        case TCC_OPTION_MF:', '            s->deps_outfile = tcc_strdup(optarg);', '            break;']),
('dumpversion', ['        case TCC_OPTION_dumpversion:', '            printf ("%s\\n", TCC_VERSION);', '            exit(0);', '            break;']),
('x', ['        case TCC_OPTION_x:', "            if (*optarg == 'c')", '                s->filetype = AFF_TYPE_C;', "            else if (*optarg == 'a')", '                s->filetype = AFF_TYPE_ASMPP;', "            else if (*optarg == 'n')", '                s->filetype = AFF_TYPE_NONE;', '            else', '                tcc_warning("unsupported language \'%s\'", optarg);', '            break;']),
('O', ['        case TCC_OPTION_O:', '            last_o = atoi(optarg);', '            break;']),
('print_search_dirs', ['        case TCC_OPTION_print_search_dirs:', '            x = OPT_PRINT_DIRS;', '            goto extra_action;']),
('impdef', ['        case TCC_OPTION_impdef:', '            x = OPT_IMPDEF;', '            goto extra_action;']),
('ar', ['        case TCC_OPTION_ar:', '            x = OPT_AR;', '        extra_action:', '            arg_start = optind - 1;', '            if (arg_start != noaction)', '                tcc_error("cannot parse %s here", r);', '            tool = x;', '            break;']),
('ignored', ['        case TCC_OPTION_traditional:', '        case TCC_OPTION_pedantic:', '        case TCC_OPTION_pipe:', '        case TCC_OPTION_s:', '            break;']),
('default', ['        default:', 'unsupported_option:', '            if (s->warn_unsupported)', '                tcc_warning("unsupported option \'%s\'", r);', '            break;']),
]

def make(name,b):
    close_body = b if any(line.strip() == 'default:' for line in b) else b+['        default:','unsupported_option:','            break;']
    return name, prefix+pre_switch_func+['        switch(popt->index) {']+close_body+['        }','    }','    return 0;','}']
cur=body.copy()
for name, chunk in chunks:
    if name == 'dumpversion':
        break
    cur += chunk
# Sanity: cur is the known-good manual cumulative body through TCC_OPTION_MF.
printf_variants=[
    ('00_through_MF', []),
    ('01_printf_empty', ['        case TCC_OPTION_dumpversion:', '            printf("");', '            break;']),
    ('02_printf_literal_no_format', ['        case TCC_OPTION_dumpversion:', '            printf("0.9.27\\n");', '            break;']),
    ('03_printf_literal_format_s', ['        case TCC_OPTION_dumpversion:', '            printf("%s\\n", "0.9.27");', '            break;']),
    ('04_printf_macro_no_format', ['        case TCC_OPTION_dumpversion:', '            printf(TCC_VERSION);', '            break;']),
    ('05_printf_macro_format_s', ['        case TCC_OPTION_dumpversion:', '            printf("%s\\n", TCC_VERSION);', '            break;']),
    ('06_fputs_macro_stdout', ['        case TCC_OPTION_dumpversion:', '            fputs(TCC_VERSION, stdout);', '            break;']),
    ('07_fputs_macro_stderr', ['        case TCC_OPTION_dumpversion:', '            fputs(TCC_VERSION, stderr);', '            break;']),
    ('08_puts_macro', ['        case TCC_OPTION_dumpversion:', '            puts(TCC_VERSION);', '            break;']),
    ('09_tcc_warning_macro_format_s', ['        case TCC_OPTION_dumpversion:', '            tcc_warning("%s", TCC_VERSION);', '            break;']),
]
variants=[make(name, cur.copy()+extra) for name, extra in printf_variants]
out=Path('tcc_parse_args_dumpversion_printf_shape_variants'); out.mkdir(exist_ok=True)
for name,lines in variants:
    safe=''.join(c if c.isalnum() else '_' for c in name)
    (out/f'{safe}.c').write_text('\n'.join(lines)+'\n')
print('diag-libtcc-tcc-parse-args-dumpversion-printf-shape: generated_variants=' + str(len(variants)))
for name,lines in variants:
    print(f'diag-libtcc-tcc-parse-args-dumpversion-printf-shape: variant {name} lines={len(lines)}')
PY
for extra in '' '-D ONE_SOURCE=1'; do
  suffix=${extra//[^A-Za-z0-9]/_}
  for src in tcc_parse_args_dumpversion_printf_shape_variants/*.c; do
    label="tcc_parse_args_dumpversion_printf_shape_${src##*/}${suffix:-_common}"; label=${label%.c}
    compile_variant "$label" "$src" "$extra"
  done
done
