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

set +e


python3 - <<'PY'
from pathlib import Path
src=Path('libtcc.asm_simplified.c').read_text().splitlines()
prefix=src[:1674]
header=['PUB_FUNC int tcc_parse_args(TCCState *s, int *pargc, char ***pargv, int optind)','{','    const TCCOption *popt;','    const char *optarg, *r;','    const char *run = NULL;','    int last_o = -1;','    int x;','    CString linker_arg;','    int tool = 0, arg_start = 0, noaction = optind;','    char **argv = *pargv;','    int argc = *pargc;']
variants=[]
def add(name, body): variants.append((name,prefix+header+body+['}']))
add('decls_return',['    return 0;'])
add('cstr_new',['    cstr_new(&linker_arg);','    return 0;'])
add('while_min',['    cstr_new(&linker_arg);','    while (optind < argc) {','        r = argv[optind];','        optind++;','    }','    return 0;'])
add('while_file_branch',['    cstr_new(&linker_arg);','    while (optind < argc) {','        r = argv[optind];',"        if (r[0] == '@' && r[1] != '\\0') {",'            args_parser_listfile(s, r + 1, optind, &argc, &argv);','            continue;','        }','        optind++;',"        if (r[0] != '-' || r[1] == '\\0') {",'            args_parser_add_file(s, r, s->filetype);','            continue;','        }','    }','    return 0;'])
add('lookup_loop',['    cstr_new(&linker_arg);','    while (optind < argc) {','        r = argv[optind];','        optind++;',"        if (r[0] != '-' || r[1] == '\\0') continue;",'        for(popt = tcc_options; ; ++popt) {','            const char *p1 = popt->name;','            const char *r1 = r + 1;','            if (p1 == NULL) tcc_error("invalid option -- \'%s\'", r);','            if (!strstart(p1, &r1)) continue;','            optarg = r1;','            break;','        }','    }','    return 0;'])
# exact full function body from after decls to closing brace, using normalized source lines 1676..1955 1-indexed => indexes 1676:1955? But append without original header.
variants.append(('full_exact_body', prefix + header + src[1685:1965]))
out=Path('tcc_parse_args_body_variants'); out.mkdir(exist_ok=True)
for i,(name,lines) in enumerate(variants): (out/f'{i:02d}_{name}.c').write_text('\n'.join(lines)+'\n')
print('diag-libtcc-tcc-parse-args-body: generated_variants=' + str(len(variants)))
for i,(name,lines) in enumerate(variants): print(f'diag-libtcc-tcc-parse-args-body: variant {i:02d} {name} lines={len(lines)}')
PY
for extra in '' '-D ONE_SOURCE=1'; do
  suffix=${extra//[^A-Za-z0-9]/_}
  for src in tcc_parse_args_body_variants/*.c; do
    label="tcc_parse_args_body_${src##*/}${suffix:-_common}"; label=${label%.c}
    compile_variant "$label" "$src" "$extra"
  done
done
