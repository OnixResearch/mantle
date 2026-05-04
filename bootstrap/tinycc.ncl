# Build tinycc 0.9.27 from tinycc 0.9.26.
#
# tcc 0.9.27 adds features needed by early GNU software and the later musl
# bridge. This derivation mirrors the live-bootstrap part boundary: patch the
# upstream 0.9.27 source, build it with the final 0.9.26 compiler, and ship a
# compiler whose default runtime paths point at this output.
#
# Output: tcc 0.9.27 plus copied Mes libc/runtime from the predecessor.
#
# Adapted from live-bootstrap steps/tcc-0.9.27/pass1.kaem for Crunch's amd64
# bootstrap chain. The upstream x86 pass rebuilds Mes runtime with tcc 0.9.27;
# Crunch keeps that handoff for a later musl-oriented part because the amd64 Mes
# CRT sources still use inline assembly forms this compiler cannot parse.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let mes = import "mes.ncl" in
let tcc26 = import "tinycc-mes.ncl" in

let mes_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/mes/mes-0.27.1.tar.gz",
  hash = "sha256-pWARnEISClTfRHAhu69MRaVilq2UFuWsBcKMREsixmk=",
  name = "mes-0.27.1-src",
} in

let tcc27_src = crunch.fetchTarball {
  # download.savannah.gnu.org returned 502 during repair-make-tcc-amd64-varargs validation;
  # use the stable Savannah mirror for the same fixed-output tarball.
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-J1qVIv8ti91hOO6Gp2Wqho3t8C7jQHcXpTI85IPwZn0=",
  name = "tcc-0.9.27-src",
} in

{
  name = "tinycc-0.9.27",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
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

      apply_tcc27_live_bootstrap_patches() {
        # Mirror live-bootstrap simple-patches/{remove,addback}-fileopen.
        # The early ar path needs the output file opened immediately before
        # header emission, not before argument validation is fully complete.
        $BB sed -i '/if ((fh = fopen(argv\[i_lib\], "wb")) == NULL)/,/    }/d' tcctools.c
        $BB sed -i '/^[[:space:]]*\/\/ write header/i\
    if ((fh = fopen(argv[i_lib], "wb")) == NULL)\
    {\
        fprintf(stderr, "tcc: ar: can'\''t open file %s \\n", argv[i_lib]);\
        goto the_end;\
    }\
' tcctools.c

        # Mirror live-bootstrap simple-patches/fiwix-paddr for later Fiwix
        # bootability while staying harmless for normal Linux-linked tools.
        $BB sed -i '/if (ph->p_offset == 0) {/,/                    }/s/ph->p_paddr = ph->p_vaddr;/ph->p_paddr = ph->p_vaddr;\
                        if (s1->text_addr == 0x80100000)\
                            ph->p_paddr = ph->p_paddr - 0x80000000;/' tccelf.c

        # Mirror live-bootstrap simple-patches/check-reloc-null, and make
        # static executable GOT materialization explicit. TinyCC's original
        # local-GOT helper only records RELA addends for R_RELATIVE entries;
        # Mes-linked static outputs need the final .got bytes populated before
        # the ELF file is written, including GLOB_DAT/JUMP_SLOT slots used by
        # startup PLT/GOT paths. Keep non-static dynamic-link behavior scoped
        # to the original local R_RELATIVE rewrite.
        $BB sed -i '/static void fill_local_got_entries(TCCState \*s1)/,/^}/c\
static void fill_local_got_entries(TCCState *s1)\
{\
    Section *reloc;\
    ElfW_Rel *rel;\
    if (!s1->got || !s1->got->reloc)\
        return;\
    reloc = s1->got->reloc;\
    for_each_elem(reloc, 0, rel, ElfW_Rel) {\
        int type = ELFW(R_TYPE)(rel->r_info);\
        int sym_index = ELFW(R_SYM)(rel->r_info);\
        int materialize = (s1->static_link && s1->output_type == TCC_OUTPUT_EXE && (type == R_GLOB_DAT || type == R_JMP_SLOT));\
        if (type == R_RELATIVE || materialize) {\
            ElfW(Sym) *sym = &((ElfW(Sym) *) reloc->link->data)[sym_index];\
            struct sym_attr *attr = 0;\
            unsigned offset = rel->r_offset - s1->got->sh_addr;\
            if (reloc->link == symtab_section)\
                attr = get_sym_attr(s1, sym_index, 0);\
            else if (materialize) {\
                const char *name = (char *) reloc->link->link->data + sym->st_name;\
                int real_index = find_elf_sym(symtab_section, name);\
                if (real_index) {\
                    sym_index = real_index;\
                    sym = &((ElfW(Sym) *) symtab_section->data)[sym_index];\
                    attr = get_sym_attr(s1, sym_index, 0);\
                }\
            }\
            if (attr && attr->got_offset)\
                offset = attr->got_offset;\
            if (offset != rel->r_offset - s1->got->sh_addr && type == R_RELATIVE)\
                tcc_error_noabort("huh");\
            rel->r_info = ELFW(R_INFO)(0, type == R_RELATIVE ? R_RELATIVE : type);\
            if (materialize)\
                section_reserve(s1->got, offset + PTR_SIZE);\
# ifdef TCC_TARGET_X86_64\
            write64le(s1->got->data + offset, sym->st_value);\
# else\
            write32le(s1->got->data + offset, sym->st_value);\
# endif\
# if SHT_RELX == SHT_RELA\
            rel->r_addend = sym->st_value;\
# endif\
        }\
    }\
}' tccelf.c

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
      }

      write_tinycc_stdarg_header() {
        header=$1
        $BB cat > "$header" <<'EOF'
#ifndef __MES_STDARG_H
#define __MES_STDARG_H 1
#if defined(__TINYC__) && defined(__x86_64__)
typedef struct {
    unsigned int gp_offset;
    unsigned int fp_offset;
    union { unsigned int overflow_offset; char *overflow_arg_area; };
    char *reg_save_area;
} __va_list_struct;
typedef __va_list_struct va_list[1];
void __va_start(__va_list_struct *ap, void *fp);
void *__va_arg(__va_list_struct *ap, int arg_type, int size, int align);
#define va_start(ap, last) __va_start(ap, __builtin_frame_address(0))
#define va_arg(ap, type) (*(type *)(__va_arg(ap, __builtin_va_arg_types(type), sizeof(type), __alignof__(type))))
#define va_arg8(ap, type) va_arg(ap, type)
#define va_copy(dest, src) (*(dest) = *(src))
#define va_end(ap)
#else
#include_next <stdarg.h>
#define va_arg8(ap, type) va_arg(ap, type)
#endif
#endif
EOF
      }

      create_unified_mes_libc_source() {
        cd "$MES_BUILD/lib"
        $BB cat \
          ctype/isalnum.c ctype/isalpha.c ctype/isascii.c ctype/iscntrl.c \
          ctype/isdigit.c ctype/isgraph.c ctype/islower.c ctype/isnumber.c \
          ctype/isprint.c ctype/ispunct.c ctype/isspace.c ctype/isupper.c \
          ctype/isxdigit.c ctype/tolower.c ctype/toupper.c \
          dirent/closedir.c dirent/__getdirentries.c dirent/opendir.c \
          linux/readdir.c linux/access.c linux/brk.c linux/chdir.c \
          linux/chmod.c linux/clock_gettime.c linux/close.c linux/dup2.c \
          linux/dup.c linux/execve.c linux/fcntl.c linux/fork.c linux/fsync.c \
          linux/fstat.c linux/_getcwd.c linux/getdents.c linux/getegid.c \
          linux/geteuid.c linux/getgid.c linux/getpid.c linux/getppid.c \
          linux/getrusage.c linux/gettimeofday.c linux/getuid.c linux/ioctl.c \
          linux/ioctl3.c linux/kill.c linux/link.c linux/lseek.c linux/lstat.c \
          linux/malloc.c linux/mkdir.c linux/mknod.c linux/nanosleep.c \
          linux/_open3.c linux/pipe.c linux/_read.c linux/readlink.c \
          linux/rename.c linux/rmdir.c linux/setgid.c linux/settimer.c \
          linux/setuid.c linux/signal.c linux/sigprogmask.c linux/symlink.c \
          linux/stat.c linux/time.c linux/unlink.c linux/waitpid.c linux/wait4.c \
          linux/${MES_ARCH}-mes-gcc/_exit.c linux/${MES_ARCH}-mes-gcc/syscall.c \
          linux/${MES_ARCH}-mes-gcc/_write.c \
          math/ceil.c math/fabs.c math/floor.c \
          mes/abtod.c mes/abtol.c mes/__assert_fail.c mes/assert_msg.c \
          mes/__buffered_read.c mes/__init_io.c mes/cast.c mes/dtoab.c \
          mes/eputc.c mes/eputs.c mes/fdgetc.c mes/fdgets.c mes/fdputc.c \
          mes/fdputs.c mes/fdungetc.c mes/globals.c mes/itoa.c mes/ltoab.c \
          mes/ltoa.c mes/__mes_debug.c mes/mes_open.c mes/ntoab.c mes/oputc.c \
          mes/oputs.c mes/search-path.c mes/ultoa.c mes/utoa.c \
          posix/alarm.c posix/buffered-read.c posix/execl.c posix/execlp.c \
          posix/execv.c posix/execvp.c posix/getcwd.c posix/getenv.c \
          posix/isatty.c posix/mktemp.c posix/open.c posix/pathconf.c \
          posix/raise.c posix/sbrk.c posix/setenv.c posix/sleep.c \
          posix/unsetenv.c posix/wait.c posix/write.c \
          stdio/clearerr.c stdio/fclose.c stdio/fdopen.c stdio/feof.c \
          stdio/ferror.c stdio/fflush.c stdio/fgetc.c stdio/fgets.c \
          stdio/fileno.c stdio/fopen.c stdio/fprintf.c stdio/fputc.c \
          stdio/fputs.c stdio/fread.c stdio/freopen.c stdio/fscanf.c \
          stdio/fseek.c stdio/ftell.c stdio/fwrite.c stdio/getc.c \
          stdio/getchar.c stdio/perror.c stdio/printf.c stdio/putc.c \
          stdio/putchar.c stdio/remove.c stdio/snprintf.c stdio/sprintf.c \
          stdio/sscanf.c stdio/ungetc.c stdio/vfprintf.c stdio/vfscanf.c \
          stdio/vprintf.c stdio/vsnprintf.c stdio/vsprintf.c stdio/vsscanf.c \
          stdlib/abort.c stdlib/abs.c stdlib/alloca.c stdlib/atexit.c \
          stdlib/atof.c stdlib/atoi.c stdlib/atol.c stdlib/calloc.c \
          stdlib/__exit.c stdlib/exit.c stdlib/free.c stdlib/mbstowcs.c \
          stdlib/puts.c stdlib/qsort.c stdlib/realloc.c stdlib/strtod.c \
          stdlib/strtof.c stdlib/strtol.c stdlib/strtold.c stdlib/strtoll.c \
          stdlib/strtoul.c stdlib/strtoull.c \
          string/bcmp.c string/bcopy.c string/bzero.c string/index.c \
          string/memchr.c string/memcmp.c string/memcpy.c string/memmem.c \
          string/memmove.c string/memset.c string/rindex.c string/strcat.c \
          string/strchr.c string/strcmp.c string/strcpy.c string/strcspn.c \
          string/strdup.c string/strerror.c string/strlen.c string/strlwr.c \
          string/strncat.c string/strncmp.c string/strncpy.c string/strpbrk.c \
          string/strrchr.c string/strspn.c string/strstr.c string/strupr.c \
          stub/atan2.c stub/bsearch.c stub/chown.c stub/__cleanup.c \
          stub/cos.c stub/ctime.c stub/exp.c stub/fpurge.c stub/freadahead.c \
          stub/frexp.c stub/getgrgid.c stub/getgrnam.c stub/getlogin.c \
          stub/getpgid.c stub/getpgrp.c stub/getpwnam.c stub/getpwuid.c \
          stub/gmtime.c stub/ldexp.c stub/localtime.c stub/log.c stub/mktime.c \
          stub/modf.c stub/mprotect.c stub/pclose.c stub/popen.c stub/pow.c \
          stub/putenv.c stub/rand.c stub/realpath.c stub/rewind.c \
          stub/setbuf.c stub/setgrent.c stub/setlocale.c stub/setvbuf.c \
          stub/sigaction.c stub/sigaddset.c stub/sigblock.c stub/sigdelset.c \
          stub/sigemptyset.c stub/sigsetmask.c stub/sin.c stub/sys_siglist.c \
          stub/system.c stub/sqrt.c stub/strftime.c stub/times.c stub/ttyname.c \
          stub/umask.c stub/utime.c ${MES_ARCH}-mes-gcc/setjmp.c \
          > ../unified-libc.c
        cd "$MES_BUILD"
      }

      rebuild_mes_libc_for_tcc27_runtime() {
        echo "tinycc-0.9.27: rebuild Mes libc with TinyCC x86_64 stdarg ABI" >&2
        $BB cp -r "$MES_SRC" "$MES_BUILD"
        $BB chmod -R u+w "$MES_BUILD"
        $BB mkdir -p "$MES_BUILD/include/mes"
        $BB cat > "$MES_BUILD/include/mes/config.h" <<'EOF'
#undef SYSTEM_LIBC
#define MES_VERSION "0.27.1"
EOF
        write_tinycc_stdarg_header "$MES_BUILD/include/stdarg.h"
        write_tinycc_stdarg_header "$INCDIR/stdarg.h"
        # TinyCC's inline-asm register allocator can choose %rax for the
        # `_exit(int code)` input unless the syscall clobbers are explicit;
        # then the preceding `mov $SYS_exit,%rax` makes every exit status 60.
        $BB sed -i 's/: \/\/ no outputs "=" (r)/: \/\/ no outputs/' "$MES_BUILD/lib/linux/${MES_ARCH}-mes-gcc/_exit.c"
        $BB sed -i 's/: "rm" (code)/: "r" (code)\n       : "rax", "rdi"/' "$MES_BUILD/lib/linux/${MES_ARCH}-mes-gcc/_exit.c"
        create_unified_mes_libc_source
        cd "$MES_BUILD"
        $BB cat > va_list.c <<'EOF'
#if defined __x86_64__
extern void *memset(void *s, int c, unsigned long n);
extern void abort(void);
enum __va_arg_type { __va_gen_reg, __va_float_reg, __va_stack };
typedef struct {
    unsigned int gp_offset;
    unsigned int fp_offset;
    union { unsigned int overflow_offset; char *overflow_arg_area; };
    char *reg_save_area;
} __va_list_struct;
void __va_start(__va_list_struct *ap, void *fp) {
    memset(ap, 0, sizeof(__va_list_struct));
    *ap = *(__va_list_struct *)((char *)fp - 16);
    ap->overflow_arg_area = (char *)fp + ap->overflow_offset;
    ap->reg_save_area = (char *)fp - 176 - 16;
}
void *__va_arg(__va_list_struct *ap, enum __va_arg_type arg_type, int size, int align) {
    size = (size + 7) & ~7;
    align = (align + 7) & ~7;
    switch (arg_type) {
    case __va_gen_reg:
        if (ap->gp_offset + size <= 48) { ap->gp_offset += size; return ap->reg_save_area + ap->gp_offset - size; }
        goto use_overflow_area;
    case __va_float_reg:
        if (ap->fp_offset < 128 + 48) { ap->fp_offset += 16; return ap->reg_save_area + ap->fp_offset - 16; }
        size = 8;
        goto use_overflow_area;
    case __va_stack:
    use_overflow_area:
        ap->overflow_arg_area += size;
        ap->overflow_arg_area = (char*)((long long)(ap->overflow_arg_area + align - 1) & -align);
        return ap->overflow_arg_area - size;
    default:
        abort();
    }
}
#endif
EOF
        "$TCC26/bin/tcc" -c -D HAVE_CONFIG_H=1 -I "$MES_BUILD/include" -I "$MES_BUILD/include/linux/${MES_ARCH}" -o va_list.o va_list.c
        $BB cp va_list.o "$LIBDIR/va_list.o"
        "$TCC26/bin/tcc" -w -c -D HAVE_CONFIG_H=1 -I "$MES_BUILD/include" -I "$MES_BUILD/include/linux/${MES_ARCH}" -o unified-libc.o unified-libc.c
        $BB rm -f "$LIBDIR/libc.a"
        "$TCC26/bin/tcc" -ar cr "$LIBDIR/libc.a" va_list.o unified-libc.o
        cd "$TCC_BUILD"
      }

      compile_tcc27() {
        compiler=$1
        output=$2
        runtime_libdir=$3
        runtime_incdir=$4
        echo "tinycc-0.9.27: compile $output with $compiler" >&2
        "$compiler" \
          -v -static \
          -o "$output" \
          -I . \
          -I "$INCDIR" \
          -D BOOTSTRAP=1 \
          -D HAVE_BITFIELD=1 \
          -D HAVE_FLOAT=1 \
          -D HAVE_LONG_LONG=1 \
          -D HAVE_SETJMP=1 \
          -D TCC_TARGET_${TCC_TARGET_ARCH}=1 \
          -D CONFIG_TCCDIR=\"$runtime_libdir/tcc\" \
          -D CONFIG_TCC_CRTPREFIX=\"$runtime_libdir\" \
          -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" \
          -D CONFIG_TCC_LIBPATHS=\"$runtime_libdir:$runtime_libdir/tcc\" \
          -D CONFIG_TCC_SYSINCLUDEPATHS=\"$runtime_incdir\" \
          -D CONFIG_SYSROOT=\"/\" \
          -D TCC_LIBGCC=\"$runtime_libdir/libc.a\" \
          -D TCC_LIBTCC1=\"libtcc1.a\" \
          -D CONFIG_TCCBOOT=1 \
          -D CONFIG_TCC_STATIC=1 \
          -D CONFIG_USE_LIBGCC=1 \
          -D TCC_VERSION=\"0.9.27\" \
          -D ONE_SOURCE=1 \
          tcc.c \
          "$runtime_libdir/va_list.o"
        $BB chmod 755 "$output"
      }

      validate_mes_runtime_contract() {
        test -s "$LIBDIR/crt1.o"
        test -s "$LIBDIR/libc.a"
        test -s "$LIBDIR/libgetopt.a"
        test -s "$LIBDIR/va_list.o"
        test -s "$LIBDIR/tcc/libtcc1.a"
        test -d "$INCDIR"
      }

      STAGE0=$(find_input stage0-posix)
      MES_OUT=$(find_input mes)
      TCC26=$(find_input tinycc-0.9.26)
      MES_SRC=$(find_input mes-0.27.1-src)
      TCC27_SRC=$(find_input tcc-0.9.27-src)

      MES_ARCH=x86_64
      TCC_TARGET_ARCH=X86_64

      WORK=/build/tcc27-build
      PREFIX="$WORK/prefix"
      LIBDIR="$PREFIX/lib/mes"
      INCDIR="$PREFIX/include/mes"
      TCC_BUILD="$WORK/tcc-0.9.27"
      MES_BUILD="$WORK/mes-0.27.1"

      export PATH="$TCC26/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$LIBDIR/tcc" "$PREFIX/include"
      $BB cp -r "$TCC27_SRC" "$TCC_BUILD"
      $BB chmod -R u+w "$TCC_BUILD"
      $BB cp -r "$TCC26/include/mes" "$INCDIR"
      $BB chmod -R u+w "$INCDIR"
      $BB cp "$TCC26/lib/mes"/*.o "$LIBDIR/"
      $BB cp "$TCC26/lib/mes"/*.a "$LIBDIR/"
      $BB cp "$TCC26/lib/mes/tcc/libtcc1.a" "$LIBDIR/tcc/libtcc1.a"
      rebuild_mes_libc_for_tcc27_runtime

      cd "$TCC_BUILD"
      : > config.h
      apply_tcc27_live_bootstrap_patches

      validate_mes_runtime_contract

      $BB mkdir -p "$out/bin" "$out/lib/mes/tcc" "$out/include"
      $BB cp -r "$INCDIR" "$out/include/mes"
      $BB cp "$LIBDIR/crt1.o" "$out/lib/mes/crt1.o"
      $BB cp "$LIBDIR/crtn.o" "$out/lib/mes/crtn.o"
      $BB cp "$LIBDIR/crti.o" "$out/lib/mes/crti.o"
      $BB cp "$LIBDIR/libc.a" "$out/lib/mes/libc.a"
      $BB cp "$LIBDIR/va_list.o" "$out/lib/mes/va_list.o"
      $BB cp "$LIBDIR/libgetopt.a" "$out/lib/mes/libgetopt.a"
      $BB cp "$LIBDIR/tcc/libtcc1.a" "$out/lib/mes/tcc/libtcc1.a"

      compile_tcc27 "$TCC26/bin/tcc-0.9.26" tcc-final "$out/lib/mes" "$out/include/mes"
      $BB cp tcc-final "$out/bin/tcc"
      $BB cp tcc-final "$out/bin/tcc-0.9.27"
      $BB chmod 755 "$out/bin/tcc" "$out/bin/tcc-0.9.27"

      "$out/bin/tcc" -version

      test -x "$out/bin/tcc"
      test -x "$out/bin/tcc-0.9.27"
      test -s "$out/lib/mes/libc.a"
      test -s "$out/lib/mes/tcc/libtcc1.a"
    "%,
  ],
  inputs = [stage0, mes, tcc26, mes_src, tcc27_src],
} | crunch.Derivation
