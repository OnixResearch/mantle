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

        # Mirror live-bootstrap simple-patches/check-reloc-null. Some later
        # objects have no GOT relocation section; treat that as empty.
        $BB sed -i '/static void fill_local_got_entries(TCCState \*s1)/,/    for_each_elem(s1->got->reloc, 0, rel, ElfW_Rel) {/c\
static void fill_local_got_entries(TCCState *s1)\
{\
    ElfW_Rel *rel;\
    if (!s1->got->reloc)\
        return;\
    for_each_elem(s1->got->reloc, 0, rel, ElfW_Rel) {' tccelf.c

        # Crunch amd64 predecessor tcc-0.9.26 still miscompiles variable
        # doublings through the power-of-two multiply shortcut as `shl $0`.
        # Spell bootstrap allocator growth as addition, matching the earlier
        # tinycc-mes source-normalization seam.
        $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
        $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
        $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
        $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
        $BB sed -i 's/#define REX_BASE(reg).*/#define REX_BASE(reg) ((reg) >= 8 ? 1 : 0)/' x86_64-gen.c
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

        # The predecessor Mes runtime varargs path corrupts snprintf/sprintf
        # string expansion in the first tcc-0.9.27 binary. Spell bootstrap
        # path constructors with direct string operations for this Mes-linked
        # handoff compiler.
        $BB sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
        $BB sed -i 's|snprintf(buf, sizeof(buf), fmt, paths.*|if (fmt[3] == '\''l'\'') { strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a"); } else { strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename); }|' libtcc.c
        $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
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
        $BB sed -i 's/loc = 0;/loc = 0;\
    ind = cur_text_section->data_offset;/' x86_64-gen.c
        $BB sed -i 's/g(vtop->c.i \& (ll ? 63 : 31));/g(vtop->c.i);/' x86_64-gen.c
        $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c

        $BB sed -i 's|snprintf(buf, sizeof(buf), "__start_%s", s->name);|strcpy(buf, "__start_"); strcat(buf, s->name);|' tccelf.c
        $BB sed -i 's|snprintf(buf, sizeof(buf), "__stop_%s", s->name);|strcpy(buf, "__stop_"); strcat(buf, s->name);|' tccelf.c
        $BB sed -i 's|snprintf(sym_start, sizeof(sym_start), "__%s_start", section_name + 1);|strcpy(sym_start, "__"); strcat(sym_start, section_name + 1); strcat(sym_start, "_start");|' tccelf.c
        $BB sed -i 's|snprintf(sym_end, sizeof(sym_end), "__%s_end", section_name + 1);|strcpy(sym_end, "__"); strcat(sym_end, section_name + 1); strcat(sym_end, "_end");|' tccelf.c


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
          tcc.c
        $BB chmod 755 "$output"
      }

      validate_mes_runtime_contract() {
        test -s "$LIBDIR/crt1.o"
        test -s "$LIBDIR/libc.a"
        test -s "$LIBDIR/libgetopt.a"
        test -s "$LIBDIR/tcc/libtcc1.a"
        test -d "$INCDIR"
      }

      STAGE0=$(find_input stage0-posix)
      MES_OUT=$(find_input mes)
      TCC26=$(find_input tinycc-0.9.26)
      TCC27_SRC=$(find_input tcc-0.9.27-src)

      MES_ARCH=x86_64
      TCC_TARGET_ARCH=X86_64

      WORK=/build/tcc27-build
      PREFIX="$WORK/prefix"
      LIBDIR="$PREFIX/lib/mes"
      INCDIR="$PREFIX/include/mes"
      TCC_BUILD="$WORK/tcc-0.9.27"

      export PATH="$TCC26/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$LIBDIR/tcc" "$PREFIX/include"
      $BB cp -r "$TCC27_SRC" "$TCC_BUILD"
      $BB chmod -R u+w "$TCC_BUILD"
      $BB cp -r "$TCC26/include/mes" "$INCDIR"
      $BB cp "$TCC26/lib/mes"/*.o "$LIBDIR/"
      $BB cp "$TCC26/lib/mes"/*.a "$LIBDIR/"
      $BB cp "$TCC26/lib/mes/tcc/libtcc1.a" "$LIBDIR/tcc/libtcc1.a"

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
  inputs = [stage0, mes, tcc26, tcc27_src],
} | crunch.Derivation
