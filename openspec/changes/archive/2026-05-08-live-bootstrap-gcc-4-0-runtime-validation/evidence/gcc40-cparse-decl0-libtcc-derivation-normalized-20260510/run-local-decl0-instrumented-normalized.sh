#!/usr/bin/env bash
set -euo pipefail

ROOT=${ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}
WORK=${WORK:-/tmp/tcc-decl0-normalized-local}
CACHE=${CACHE:-/tmp/crunch-tcc-version-faithful-cache}
RESTORE=${RESTORE:-/tmp/crunch-tcc-version-faithful-restore}
STATE_DIR=${STATE_DIR:-/home/brittonr/.local/state/crunch}
CRUNCH=${CRUNCH:-$ROOT/target/debug/crunch}

SOURCE_FRAGMENT=4jp67gx0k6h76np8wazalwlg5wyg5g3q-tcc-0.9.27-src
TCC_FRAGMENT=wl2qc53zbwka1qkq5kn1vk5lqzn6zh8l-tcc-0.9.27-musl-v2
MUSL_FRAGMENT=p1m8x9l2kbprlqbwy8xxahqjrkrb7v8i-musl-1.1.24-tcc-musl

restore_fragment() {
  local fragment=$1
  local out="$RESTORE/$fragment"
  if [ -e "$out" ]; then printf '%s\n' "$out"; return 0; fi
  mkdir -p "$CACHE" "$RESTORE"
  "$CRUNCH" store push --state-dir "$STATE_DIR" --store "$ROOT/.crunch-drain/store" --to "$CACHE" --trust-unsigned "$fragment" >/tmp/crunch-decl0-normalized-push.log
  local narinfo="$CACHE/${fragment%%-*}.narinfo"
  local url
  url=$(awk '/^URL:/{print $2}' "$narinfo")
  nix-store --restore "$out" < "$CACHE/$url"
  printf '%s\n' "$out"
}

TCC_SRC=$(restore_fragment "$SOURCE_FRAGMENT")
TCC=$(restore_fragment "$TCC_FRAGMENT")
MUSL=$(restore_fragment "$MUSL_FRAGMENT")

printf 'diag-decl0-normalized-local: mode=crunch-cache-restored\n'
printf 'diag-decl0-normalized-local: tcc=%s\n' "$TCC/bin/tcc"
printf 'diag-decl0-normalized-local: musl=%s\n' "$MUSL"
printf 'diag-decl0-normalized-local: source=%s\n' "$TCC_SRC"

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
    return 0; /* CRUNCH diag: suppress missing crt error in predecessor build */;\
    return 0;\
}' libtcc.c
sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
cp tccgen.c tccgen.c.with-native387
awk 'done == 0 && $0 == "#if defined TCC_IS_NATIVE_387" { print "#if 0 /* CRUNCH diag: predecessor tccgen native x87 compile boundary */"; done = 1; next } { print }' tccgen.c.with-native387 > tccgen.c
awk 'NR == 642 { print "            tcc_error_noabort(\"asm not supported\");"; skip = 1; next } skip && NR <= 646 { print "            /* CRUNCH diag: CONFIG_TCC_ASM island removed */"; next } { print }' libtcc.c > libtcc.tmp && mv libtcc.tmp libtcc.c
sed -i 's/^[[:space:]]*va_start(ap, fmt);/    \/\* CRUNCH diag: va_start removed with va_end pair \*\//g; s/^[[:space:]]*va_end(ap);/    \/\* CRUNCH diag: va_end removed with va_start pair \*\//g' libtcc.c
awk '{ if ($0 ~ /^[[:space:]]*sscanf\(TCC_VERSION,/) print "        a = 0; b = 9; c = 27; /* CRUNCH diag: sscanf(TCC_VERSION) bypassed */"; else print }' libtcc.c > libtcc.tmp && mv libtcc.tmp libtcc.c
sed -i '/copy_linker_arg(\&s->/c\            /* CRUNCH diag: copy_linker_arg field call removed */' libtcc.c
awk '{ if ($0 == "#ifdef TCC_TARGET_X86_64" && prev ~ /ms_bitfields/) { print "/* CRUNCH diag: options_m TCC_TARGET_X86_64 guard removed */"; while ((getline line) > 0) { if (line == "#endif") { print "/* CRUNCH diag: end removed options_m guard */"; break } print line } prev=""; next } print; prev=$0 }' libtcc.c > libtcc.tmp && mv libtcc.tmp libtcc.c
awk '
  /^static int decl0\(int l, int is_for_loop_init, Sym \*func_sym\)/ { in_decl0=1 }
  {
    print
    if (in_decl0 && $0 ~ /^[[:space:]]*while \(1\) \{/) print "        fputs(\"diag-tcc-decl0-runtime: enter\\n\", stderr);"
    if (in_decl0 && $0 ~ /type_decl\(&type, &ad, &v, TYPE_DIRECT\);/) print "            fputs(\"diag-tcc-decl0-runtime: after-type-decl\\n\", stderr);"
    if (in_decl0 && $0 ~ /^[[:space:]]*if .*tok == .*\{/) print "            fputs(\"diag-tcc-decl0-runtime: branch-function-body\\n\", stderr);"
    if (in_decl0 && $0 ~ /has_init = .*tok ==/) print "                    fputs(\"diag-tcc-decl0-runtime: decl-tail\\n\", stderr);"
    if (in_decl0 && $0 ~ /skip.*;/) print "                    fputs(\"diag-tcc-decl0-runtime: skip-semicolon\\n\", stderr);"
    if (in_decl0 && $0 ~ /^}/) in_decl0=0
  }
' tccgen.c > tccgen.tmp && mv tccgen.tmp tccgen.c

FLAGS='-D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_X86_64=1 -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION="0.9.27-decl0-diag" -D ONE_SOURCE=1'
printf 'diag-decl0-normalized-local: compile full libtcc\n'
set +e
"$TCC/bin/tcc" -c -I . -I "$MUSL/include" $FLAGS libtcc.c -o /tmp/decl0-normalized-libtcc.o 2> /tmp/decl0-normalized-libtcc.stderr
libtcc_rc=$?
set -e
printf 'diag-decl0-normalized-local: libtcc rc=%s\n' "$libtcc_rc"
sed 's/^/diag-decl0-normalized-local: libtcc stderr: /' /tmp/decl0-normalized-libtcc.stderr || true
BUILD_DEFS=(
  -D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1
  -D TCC_TARGET_X86_64=1
  -D CONFIG_TCCDIR="$TCC/lib/tcc"
  -D CONFIG_TCC_CRTPREFIX="$MUSL/lib"
  -D CONFIG_TCC_ELFINTERP="/lib/ld-musl-x86_64.so.1"
  -D CONFIG_TCC_LIBPATHS="$MUSL/lib:$TCC/lib/tcc"
  -D CONFIG_TCC_SYSINCLUDEPATHS="$MUSL/include"
  -D CONFIG_SYSROOT="/" -D TCC_LIBGCC="$MUSL/lib/libc.a" -D TCC_LIBTCC1="$TCC/lib/tcc/libtcc1.a"
  -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION="0.9.27-decl0-diag" -D ONE_SOURCE=1
)
run_phase() {
  phase=$1
  shift
  printf 'diag-decl0-normalized-local: phase %s\n' "$phase"
  set +e
  "$@" 2> "/tmp/decl0-normalized-${phase}.stderr"
  phase_rc=$?
  set -e
  printf 'diag-decl0-normalized-local: phase %s rc=%s\n' "$phase" "$phase_rc"
  sed 's/^/diag-decl0-normalized-local: phase '$phase' stderr: /' "/tmp/decl0-normalized-${phase}.stderr" || true
}

run_phase compile-only-quiet "$TCC/bin/tcc" -c -I . -I "$MUSL/include" "${BUILD_DEFS[@]}" tcc.c -o /tmp/tcc-decl0-normalized.o
run_phase compile-only-verbose "$TCC/bin/tcc" -c -v -I . -I "$MUSL/include" "${BUILD_DEFS[@]}" tcc.c -o /tmp/tcc-decl0-normalized.o
run_phase link-object-default "$TCC/bin/tcc" -v -static -o /tmp/tcc-decl0-normalized-from-object /tmp/tcc-decl0-normalized.o
run_phase link-object-explicit-archives "$TCC/bin/tcc" -v -static -o /tmp/tcc-decl0-normalized-from-object-explicit /tmp/tcc-decl0-normalized.o "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a"
for input in tcc.c /tmp/tcc-decl0-normalized.o "$TCC/lib/tcc/libtcc1.a" "$MUSL/lib/libc.a" "$MUSL/lib/crt1.o" "$MUSL/lib/crti.o" "$MUSL/lib/crtn.o"; do
  if [ -e "$input" ]; then
    printf 'diag-decl0-normalized-local: input-exists %s\n' "$input"
  else
    printf 'diag-decl0-normalized-local: input-missing %s\n' "$input"
  fi
done

printf 'diag-decl0-normalized-local: build instrumented compiler\n'
set +e
"$TCC/bin/tcc" -v -static -o /tmp/tcc-decl0-normalized \
  -I . -I "$MUSL/include" \
  -D BOOTSTRAP=1 -D HAVE_BITFIELD=1 -D HAVE_FLOAT=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 \
  -D TCC_TARGET_X86_64=1 \
  -D CONFIG_TCCDIR="$TCC/lib/tcc" \
  -D CONFIG_TCC_CRTPREFIX="$MUSL/lib" \
  -D CONFIG_TCC_ELFINTERP="/lib/ld-musl-x86_64.so.1" \
  -D CONFIG_TCC_LIBPATHS="$MUSL/lib:$TCC/lib/tcc" \
  -D CONFIG_TCC_SYSINCLUDEPATHS="$MUSL/include" \
  -D CONFIG_SYSROOT="/" -D TCC_LIBGCC="$MUSL/lib/libc.a" -D TCC_LIBTCC1="$TCC/lib/tcc/libtcc1.a" \
  -D CONFIG_TCCBOOT=1 -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION="0.9.27-decl0-diag" -D ONE_SOURCE=1 \
  tcc.c 2> /tmp/decl0-normalized-build.stderr
build_rc=$?
set -e
printf 'diag-decl0-normalized-local: build rc=%s\n' "$build_rc"
sed 's/^/diag-decl0-normalized-local: build stderr: /' /tmp/decl0-normalized-build.stderr || true
if [ "$build_rc" = 0 ]; then
  chmod 755 /tmp/tcc-decl0-normalized
  cat > /tmp/decl0-smoke.c <<'EOF_SMOKE'
int global_decl0_smoke;
int f(int x) { return x + 1; }
EOF_SMOKE
  set +e
  /tmp/tcc-decl0-normalized -c -I . -I "$MUSL/include" /tmp/decl0-smoke.c -o /tmp/decl0-smoke.o 2> /tmp/decl0-smoke.stderr
  smoke_rc=$?
  set -e
  printf 'diag-decl0-normalized-local: smoke rc=%s\n' "$smoke_rc"
  sed 's/^/diag-decl0-normalized-local: smoke stderr: /' /tmp/decl0-smoke.stderr || true
fi
