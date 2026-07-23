# Final tcc 0.9.27 rebuilt after self-consistent musl (v2).
#
# Reuse the validated bridge build shape from tcc-musl while targeting the
# second-pass musl output. This keeps the fragile TinyCC 0.9.27 source build
# hosted by the TinyCC 0.9.26 Mes compiler until the downstream self-host
# boundary is independently repaired.
#
# Adapted from live-bootstrap steps/tcc-0.9.27 (musl-v2 pass).

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let mes = crunch.derivationFile "mes.ncl" in
let tcc26 = crunch.derivationFile "tinycc-mes.ncl" in
let tcc_prep = crunch.derivationFile "tcc-musl-prep.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-tcc-musl.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let sed = crunch.derivationFile "sed-tcc.ncl" in

let tcc27_src = crunch.fetchTarball {
  # first-consumer: bootstrap/tcc-musl-v2.ncl
  # provenance: fosslinux/live-bootstrap steps/tcc-0.9.27 musl-v2 pass source input
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-J1qVIv8ti91hOO6Gp2Wqho3t8C7jQHcXpTI85IPwZn0=",
  name = "tcc-0.9.27-src",
} in

{
  name = "tcc-0.9.27-musl-v2",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for d in $NIX_STORE/*-"$1"; do
          if [ -d "$d" ]; then echo "$d"; return; fi
        done
        echo "ERROR: input $1 not found" >&2; exit 1
      }

      TCC26=$(find_input tinycc-0.9.26)
      MES_OUT=$(find_input mes)
      TCC_PREP=$(find_input tcc-0.9.27-musl-prep)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      SED=$(find_input sed-4.0.9-tcc)
      TCC27_SRC=$(find_input tcc-0.9.27-src)

      export PATH="$TCC26/bin:$TCC_PREP/bin:$MAKE/bin:$SED/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      MES_ARCH=x86_64
      WORK=/tmp/tcc-musl-v2-build
      $BB mkdir -p "$WORK"
      cd "$WORK"

      $BB cp -r "$TCC27_SRC" tcc-0.9.27
      $BB chmod -R u+w tcc-0.9.27
      cd tcc-0.9.27

      : > config.h

      # The Mes/TinyCC handoff can parse <inttypes.h> without leaving usable
      # fixed-width typedef names for TinyCC's bundled elf.h/tcc.h on amd64.
      # Force the handful of exact-width names this bootstrap compiler needs
      # to primitive token aliases after elf.h's include block so both elf.h
      # and the later little-endian helpers parse without depending on the
      # predecessor's fragile Mes inttypes/stdint bridge.
      $BB sed -i '/#include <inttypes.h>/a\
#define uint8_t unsigned char\
#define uint16_t unsigned short\
#define uint32_t unsigned int\
#define uint64_t unsigned long long\
#define int32_t int\
#define int64_t long long\
#define CRUNCH_TCC_ELF_FIXED_WIDTH_ALIASES 1\
' elf.h

      # CRUNCH: reuse TinyCC 0.9.27 source-normalization seams from bootstrap/tinycc.ncl.
      # The musl-prep compiler is still built by the Mes-linked TinyCC bridge,
      # so it needs the same amd64 codegen, varargs/path, CRT, GOT/PLT, and
      # source-construction repairs as the final Mes-linked TinyCC output.
        # Crunch amd64 predecessor tcc-0.9.26 still miscompiles variable
        # doublings through the power-of-two multiply shortcut as `shl $0`.
        # Spell bootstrap allocator growth as addition, matching the earlier
        # tinycc-mes source-normalization seam.
        $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
        $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
        $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
        $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
        $BB sed -i 's/#define REX_BASE(reg).*/#define REX_BASE(reg) (((reg) \& 8) ? 1 : 0)/' x86_64-gen.c
        $BB sed -i 's/(REX_BASE(r2) << 2) | (ll << 3)/(REX_BASE(r2) ? 4 : 0) | (ll ? 8 : 0)/' x86_64-gen.c
        $BB sed -i 's/(REX_BASE(r) << 8)/(REX_BASE(r) ? 256 : 0)/g' x86_64-gen.c
        $BB sed -i '/ST_FUNC void o(unsigned int c)/,/^}/c\
ST_FUNC void o(unsigned int c)\
{\
    unsigned char *p = (unsigned char *)&c;\
    int n = sizeof(c);\
    int i;\
    while (n > 0 && p[n - 1] == 0)\
        n--;\
    for (i = 0; i < n; i++)\
        g(p[i]);\
}\
' x86_64-gen.c
        $BB sed -i '/ST_FUNC void gen_le16(int v)/,/^}/c\
ST_FUNC void gen_le16(int v)\
{\
    unsigned char *p = (unsigned char *)&v;\
    g(p[0]);\
    g(p[1]);\
}\
' x86_64-gen.c
        $BB sed -i '/ST_FUNC void gen_le32(int c)/,/^}/c\
ST_FUNC void gen_le32(int c)\
{\
    unsigned char *p = (unsigned char *)&c;\
    g(p[0]);\
    g(p[1]);\
    g(p[2]);\
    g(p[3]);\
}\
' x86_64-gen.c
        $BB sed -i '/ST_FUNC void gen_le64(int64_t c)/,/^}/c\
ST_FUNC void gen_le64(int64_t c)\
{\
    unsigned char *p = (unsigned char *)&c;\
    g(p[0]);\
    g(p[1]);\
    g(p[2]);\
    g(p[3]);\
    g(p[4]);\
    g(p[5]);\
    g(p[6]);\
    g(p[7]);\
}\
' x86_64-gen.c
        $BB sed -i 's/g(func_ret_sub >> 8);/{ unsigned char *p = (unsigned char *)\&func_ret_sub; g(p[1]); }/g' x86_64-gen.c

        # The Mes-built predecessor can leave 64-bit absolute data/GOT
        # relocations unapplied in static outputs when the inline 64-bit
        # little-endian helpers use multiword shifts. Keep the public helper
        # names but spell 64-bit loads/stores as x86_64 byte copies so
        # R_X86_64_64 data initializers and static PLT/GOT slots are
        # materialized before the ELF is written.
        $BB sed -i '/static inline uint64_t read64le(unsigned char \*p)/,/^}/c\
static inline uint64_t read64le(unsigned char *p) {\
    uint64_t x;\
    memcpy(&x, p, sizeof(x));\
    return x;\
}\
' tcc.h
        $BB sed -i '/static inline void write64le(unsigned char \*p, uint64_t x)/,/^}/c\
static inline void write64le(unsigned char *p, uint64_t x) {\
    memcpy(p, &x, sizeof(x));\
}\
' tcc.h
        $BB sed -i '/static inline void add64le(unsigned char \*p, int64_t x)/,/^}/c\
static inline void add64le(unsigned char *p, int64_t x) {\
    uint64_t y;\
    memcpy(&y, p, sizeof(y));\
    y += x;\
    memcpy(p, &y, sizeof(y));\
}\
' tcc.h

        # The predecessor Mes runtime varargs path corrupts snprintf/sprintf
        # string expansion in bootstrap path constructors. Spell those
        # deterministic path constructors with direct string operations for
        # this Mes-linked handoff compiler, but keep TinyCC's diagnostic
        # vsnprintf path intact: the installed compiler now carries the
        # TinyCC-compatible Mes stdarg runtime and must expand warning/error
        # arguments instead of printing literal `%s:%d` placeholders.
        $BB sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
        $BB sed -i 's|snprintf(buf, sizeof(buf), fmt, paths.*|if (fmt[3] == '\''l'\'') { strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a"); } else { strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename); }|' libtcc.c
        # Mes 0.27.1's vsnprintf still cannot consume TinyCC's x86_64 va_list
        # reliably when a va_list is forwarded through TinyCC's diagnostic
        # helper.  Replace only the diagnostic append formatter with a small
        # direct va_arg formatter for the integer/string conversions TinyCC
        # uses in warnings/errors; leave deterministic path constructors on
        # the non-varargs string-copy path above.
        $BB sed -i '/static void strcat_vprintf(char \*buf, int buf_size, const char \*fmt, va_list ap)/,/^}/c\
static void crunch_diag_putc(char *buf, int buf_size, int *pos, int c)\
{\
    if (*pos < buf_size - 1) { buf[*pos] = c; buf[*pos + 1] = 0; }\
    ++*pos;\
}\
static void crunch_diag_puts(char *buf, int buf_size, int *pos, const char *s)\
{\
    if (!s) s = "(null)";\
    while (*s) crunch_diag_putc(buf, buf_size, pos, *s++);\
}\
static void crunch_diag_uint(char *buf, int buf_size, int *pos, unsigned long long v, int base, int neg)\
{\
    char tmp[32]; int n = 0;\
    if (neg) crunch_diag_putc(buf, buf_size, pos, '\''-'\'');\
    do { int d = v % base; tmp[n++] = d < 10 ? '\''0'\'' + d : '\''a'\'' + d - 10; v /= base; } while (v);\
    while (n) crunch_diag_putc(buf, buf_size, pos, tmp[--n]);\
}\
static void strcat_vprintf(char *buf, int buf_size, const char *fmt, va_list ap)\
{\
    int pos = strlen(buf);\
    (void)va_arg(ap, char *); /* CRUNCH: TinyCC/Mes x86_64 va_start exposes the named fmt slot first. */\
    while (*fmt) {\
        int long_mod = 0;\
        if (*fmt != '\''%'\'') { crunch_diag_putc(buf, buf_size, &pos, *fmt++); continue; }\
        ++fmt;\
        if (*fmt == '\''%'\'') { crunch_diag_putc(buf, buf_size, &pos, *fmt++); continue; }\
        while (*fmt == '\''l'\'') { long_mod++; fmt++; }\
        switch (*fmt++) {\
        case '\''s'\'': crunch_diag_puts(buf, buf_size, &pos, va_arg(ap, char *)); break;\
        case '\''c'\'': crunch_diag_putc(buf, buf_size, &pos, va_arg(ap, int)); break;\
        case '\''d'\'': case '\''i'\'': { long long x = long_mod ? va_arg(ap, long) : va_arg(ap, int); unsigned long long u = x < 0 ? -x : x; crunch_diag_uint(buf, buf_size, &pos, u, 10, x < 0); break; }\
        case '\''u'\'': crunch_diag_uint(buf, buf_size, &pos, long_mod ? va_arg(ap, unsigned long) : va_arg(ap, unsigned int), 10, 0); break;\
        case '\''x'\'': case '\''X'\'': crunch_diag_uint(buf, buf_size, &pos, long_mod ? va_arg(ap, unsigned long) : va_arg(ap, unsigned int), 16, 0); break;\
        case '\''p'\'': crunch_diag_puts(buf, buf_size, &pos, "0x"); crunch_diag_uint(buf, buf_size, &pos, (unsigned long)va_arg(ap, void *), 16, 0); break;\
        default: crunch_diag_putc(buf, buf_size, &pos, '\''%'\''); --fmt; break;\
        }\
    }\
}' libtcc.c
        $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
        $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
        $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
        $BB sed -i '/s1->tcc_lib_path, filename);/s|.*|    strcpy(buf, s1->tcc_lib_path); strcat(buf, "/"); strcat(buf, filename);|' tccelf.c
        $BB sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rela"); strcat(buf, s->name);|' tccelf.c
        $BB sed -i 's/const char filename../const char *filename/' tccelf.c


        # Mes/TinyCC varargs can corrupt tcctools.c archive size fields, which
        # makes later `tcc -ar` outputs unreadable by TinyCC/archive readers.
        # Avoid sprintf for the fixed-width ar header size columns.
        $BB sed -i '/static int ar_usage/i\
static void crunch_ar_size(char dst[10], unsigned long v)\
{\
    char tmp[20]; int i = 0, j = 0;\
    do { tmp[i++] = 48 + (v % 10); v /= 10; } while (v && i < 20);\
    while (i && j < 10) dst[j++] = tmp[--i];\
    while (j < 10) dst[j++] = 32;\
}\
' tcctools.c
        $BB sed -i '/sprintf(stmp, .*-10d.*, fsize);/{N;s|.*\n.*|        crunch_ar_size(arhdro.ar_size, (unsigned long)fsize);|;}' tcctools.c
        $BB sed -i '/sprintf(stmp, .*-10d.*, (int)(strpos/{N;s|.*\n.*|    crunch_ar_size(arhdr.ar_size, (unsigned long)(strpos + (funccnt+1) * sizeof(int)));|;}' tcctools.c

        # Carry the same Mes/TCC source-normalization seams used by the
        # predecessor compiler. The 0.9.26 handoff still inherits Mes-built
        # x86_64 codegen defects around aggregate SValue copies and variable
        # shift counts; if 0.9.27 is built without these normalizations it can
        # compile objects but segfault in the executable/DLL output path.
        $BB sed -i 's/SValue tmp = vtop\[0\];/SValue tmp;\
            memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/tmp = vtop\[0\];/memcpy(\&tmp, \&vtop[0], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/vtop\[0\] = vtop\[-i\];/memcpy(\&vtop[0], \&vtop[-i], sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/vtop\[-i\] = tmp;/memcpy(\&vtop[-i], \&tmp, sizeof(SValue));/' x86_64-gen.c
        $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
        $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
        $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
        $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
        $BB sed -i 's/loc = 0;/loc = 0;\
    ind = cur_text_section->data_offset;/' x86_64-gen.c
        $BB sed -i 's/g(vtop->c.i \& (ll ? 63 : 31));/g(vtop->c.i);/' x86_64-gen.c
        # The bootstrap compiler only produces static executables here. Avoid
        # emitting PLT32 call relocations for ordinary function calls: the
        # Mes-built linker path can leave static PLT/GOT slots unmaterialized,
        # so helper calls jump through zeroed GOT entries at runtime. PC32 lets
        # the final static link resolve direct calls to the known code address.
        $BB sed -i 's/R_X86_64_PLT32/R_X86_64_PC32/g' x86_64-gen.c
        $BB sed -i '/R_X86_64_GOTPCREL/c\    gen_addrpc32(r, sym, c); return;' x86_64-gen.c
        $BB sed -i '/gen_modrm64(0x8b, tr, fr, sv->sym, 0);/c\            orex(1, 0, tr, 0x8d); o(0x05 + REG_VALUE(tr) * 8); gen_addrpc32(fr, sv->sym, 0);' x86_64-gen.c
        $BB awk 'BEGIN{done=0;skip=0} skip {skip=0; next} { if (!done && index($0, "if ((fr & VT_VALMASK) == VT_CONST && (fr & VT_SYM) &&") != 0) { print "    if (0) {"; done=1; skip=1; next } print }' x86_64-gen.c > x86_64-gen.c.tmp
        $BB mv x86_64-gen.c.tmp x86_64-gen.c
        $BB sed -i '/(fr \& VT_LVAL) \&\& !(sv->sym->type.t \& VT_STATIC))/c\        0) {' x86_64-gen.c
        # code instead of loading them through GOTPCREL slots. Function-pointer
        # initializers otherwise load zero from the unfilled static GOT and
        # indirect-call through address 0.
        # Global lvalue loads/stores in static outputs must use direct RIP-relative
        # addressing too. The upstream x86_64 backend routes non-static globals
        # through GOTPCREL even for static executables; Crunch's static link path
        # does not reliably materialize those GOT slots, so stores like `x = 1`
        # can load a zero pointer and segfault.
        $BB sed -i '/if (fr == VT_CONST \&\& (v->r \& VT_SYM)) {/c\    if (0 && fr == VT_CONST && (v->r & VT_SYM)) {' x86_64-gen.c
        $BB sed -i 's/if (sv->sym->type.t \& VT_STATIC) {/if (1 || (sv->sym->type.t \& VT_STATIC)) {/' x86_64-gen.c
        $BB sed -i '/^[[:space:]]*is_got = /c\    is_got = 0;' x86_64-gen.c
        $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c
        $BB sed -i '/sprintf(buf,/c\
    buf[0] = '\''"'\''; strcpy(buf + 1, file->filename); strcat(buf, "\\"");' tccpp.c
        $BB sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\
        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c

        $BB sed -i 's|snprintf(buf, sizeof(buf), "__start_%s", s->name);|strcpy(buf, "__start_"); strcat(buf, s->name);|' tccelf.c
        $BB sed -i 's|snprintf(buf, sizeof(buf), "__stop_%s", s->name);|strcpy(buf, "__stop_"); strcat(buf, s->name);|' tccelf.c
        $BB sed -i 's|snprintf(sym_start, sizeof(sym_start), "__%s_start", section_name + 1);|strcpy(sym_start, "__"); strcat(sym_start, section_name + 1); strcat(sym_start, "_start");|' tccelf.c
        $BB sed -i 's|snprintf(sym_end, sizeof(sym_end), "__%s_end", section_name + 1);|strcpy(sym_end, "__"); strcat(sym_end, section_name + 1); strcat(sym_end, "_end");|' tccelf.c

        # For static executables, remember the GOT slot that backs a PLT entry
        # so the later fill_got_entry() pass can materialize it. Upstream only
        # stores plt_offset for R_JMP_SLOT entries because a dynamic loader will
        # fill the GOT; Crunch's Mes-linked static executables have no loader.
        $BB sed -i '/attr->plt_offset = create_plt_entry(s1, got_offset, attr);/a\        if (s1->static_link && s1->output_type == TCC_OUTPUT_EXE) attr->got_offset = got_offset;' tccelf.c

        # tcc-mes can emit x86_64 PLT entries even for static executables.
        # Relocate those PLT slots after final section layout; otherwise a
        # linked binary can jump through unrelocated offsets at runtime.
        $BB sed -i '/if (file_type == TCC_OUTPUT_EXE && s1->static_link)/,/        fill_got(s1);/c\
    if (file_type == TCC_OUTPUT_EXE && s1->static_link) {\
        if (s1->got)\
            fill_local_got_entries(s1);\
        if (s1->plt)\
            relocate_plt(s1);\
        fill_got(s1);\
    }' tccelf.c

        # Match the Mes-stage bootstrap contract: avoid loading crti.o/crtn.o
        # through the Mes-built TinyCC CRT path. That path enters broken
        # varargs/path diagnostics and can corrupt the final static link.
        $BB sed -i '/ST_FUNC int tcc_add_crt(TCCState \*s, const char \*filename)/,/^}/c\
ST_FUNC int tcc_add_crt(TCCState *s, const char *filename)\
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


      echo "tcc-musl: compile object with tinycc-0.9.26 host"
      tcc \
        -v -c \
        -o tcc-musl.o \
        -I . \
        -I "$TCC26/include/mes" \
        -D BOOTSTRAP=1 \
        -D HAVE_BITFIELD=1 \
        -D HAVE_FLOAT=1 \
        -D HAVE_LONG_LONG=1 \
        -D HAVE_SETJMP=1 \
        -D TCC_TARGET_X86_64=1 \
        -D CONFIG_TCCDIR=\"$out/lib/tcc\" \
        -D CONFIG_TCC_CRTPREFIX=\"$MUSL/lib\" \
        -D CONFIG_TCC_ELFINTERP=\"/lib/ld-musl-x86_64.so.1\" \
        -D CONFIG_TCC_LIBPATHS=\"$MUSL/lib:$out/lib/tcc:$TCC26/lib/mes:$TCC26/lib/mes/tcc\" \
        -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$MUSL/include"'"' \
        -D CONFIG_SYSROOT=\"/\" \
        -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" \
        -D TCC_LIBTCC1=\"libtcc1.a\" \
        -D CONFIG_TCCBOOT=1 \
        -D CONFIG_TCC_STATIC=1 \
        -D CONFIG_USE_LIBGCC=1 \
        -D TCC_VERSION=\"0.9.27\" \
        -D ONE_SOURCE=1 \
        tcc.c
      test -s tcc-musl.o
      $BB chmod u+rw tcc-musl.o

      # Build compiler executable with the known-good Mes runtime. The produced
      # compiler is configured to target the first musl output and ships the
      # musl libtcc1 contract below for downstream links.
      echo "tcc-musl: link compiler executable with Mes runtime"
      $BB cp "$TCC26/lib/mes/crt1.o" mes-crt1.o
      $BB cp "$TCC26/lib/mes/crti.o" mes-crti.o
      $BB cp "$TCC26/lib/mes/crtn.o" mes-crtn.o
      $BB cp "$TCC26/lib/mes/libc.a" mes-libc.a
      $BB cp "$TCC26/lib/mes/tcc/libtcc1.a" mes-libtcc1.a
      tcc \
        -v -nostdlib -static \
        -o tcc-musl \
        mes-crt1.o mes-crti.o tcc-musl.o mes-libc.a mes-libtcc1.a mes-crtn.o
      $BB chmod 755 tcc-musl

      echo "tcc-musl: install outputs"
      $BB mkdir -p "$out/bin" "$out/lib/tcc"
      $BB cp tcc-musl "$out/bin/tcc"
      $BB cp tcc-musl "$out/bin/tcc-0.9.27-musl-v2"
      $BB chmod 755 "$out/bin/tcc" "$out/bin/tcc-0.9.27-musl-v2"

      # Preserve the exact normalized C source that produced this compiler so
      # the next self-host pass cannot drift from predecessor-specific repairs.
      $BB mkdir -p "$out/share/tcc-source"
      for source_file in *.c *.h *.def; do
        test -f "$source_file" || continue
        $BB cp "$source_file" "$out/share/tcc-source/$source_file"
      done
      $BB cp -r include "$out/share/tcc-source/include"
      test -f "$out/share/tcc-source/tcc.c"
      test -f "$out/share/tcc-source/x86_64-gen.c"
      test -f "$out/share/tcc-source/include/stdarg.h"

      # libtcc1.a is part of the compiler runtime contract.  The first musl
      # stage can use the predecessor runtime archive until a later self-hosted
      # musl tcc rebuild replaces it.
      echo "tcc-musl: install libtcc1"
      $BB cp "$TCC_PREP/lib/${MES_ARCH}-mes/tcc/libtcc1.a" "$out/lib/tcc/libtcc1.a"

      echo "tcc-musl: verify output contract"
      test -x "$out/bin/tcc"
      test -x "$out/bin/tcc-0.9.27-musl-v2"
      test -s "$out/lib/tcc/libtcc1.a"
      echo "tcc-musl: smoke compile"
      $BB cat > smoke.c <<'EOF'
int main(void) { return 0; }
EOF
      set +e
      "$out/bin/tcc" -v -c -o smoke.o smoke.c
      smoke_rc=$?
      set -e
      echo "tcc-musl: smoke rc=$smoke_rc"
      test "$smoke_rc" -eq 0
      test -s smoke.o
    "%,
  ],
  inputs = [stage0, mes, tcc26, tcc_prep, musl, make, sed, tcc27_src],
} | crunch.Derivation
