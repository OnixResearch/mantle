# Self-host TinyCC 0.9.27 with the current musl-targeting predecessor.
#
# The resulting compiler is still linked to the declared Mes runtime, but its
# parser and code generator are rebuilt from TinyCC source by TinyCC rather
# than emitted by TinyCC 0.9.26.  This is an isolated compiler
# candidate for building the upstream-faithful static musl candidate.

let crunch = import "lib.ncl" in

let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let mes = crunch.derivationFile "mes.ncl" in
let tcc26 = crunch.derivationFile "tinycc-mes.ncl" in
let host_tcc = crunch.derivationFile "tcc-musl-v2.ncl" in
let target_musl = crunch.derivationFile "musl-1.1.24-tcc-musl.ncl" in

{
  name = "tcc-0.9.27-musl-selfhost",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      find_input() {
        for candidate in $NIX_STORE/*-"$1"; do
          if [ -d "$candidate" ]; then
            echo "$candidate"
            return
          fi
        done
        echo "ERROR: input $1 not found" >&2
        exit 1
      }

      HOST_TCC=$(find_input tcc-0.9.27-musl-v2)
      TCC26=$(find_input tinycc-0.9.26)
      MUSL=$(find_input musl-1.1.24-tcc-musl)
      SOURCE="$HOST_TCC/share/tcc-source"
      STAGE0=$(find_input stage0-posix)
      MES_OUT=$(find_input mes)
      export PATH="$HOST_TCC/bin:$STAGE0/bin:$MES_OUT/bin:$PATH"

      WORK=/tmp/tcc-selfhost
      $BB mkdir -p "$WORK"
      test -f "$SOURCE/tcc.c" || { echo "ERROR: normalized TinyCC source missing" >&2; exit 1; }
      $BB cp -r "$SOURCE" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"
      : > config.h

      # Mes runtime formatting cannot construct deterministic target paths.
      # Keep the same direct-string path normalization used by the predecessor,
      # without carrying its predecessor-codegen rewrites into this self-host.
      $BB sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), fmt, paths.*|if (fmt[3] == '\''l'\'') { strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a"); } else { strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename); }|' libtcc.c
      $BB sed -i '/s1->tcc_lib_path, filename);/s|.*|    strcpy(buf, s1->tcc_lib_path); strcat(buf, "/"); strcat(buf, filename);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rela"); strcat(buf, s->name);|' tccelf.c
      # Weak public definitions are valid archive providers.  Include their
      # object/function symbols in the generated archive index so alacarte
      # linking can resolve musl aliases without whole-archive loading.
      $BB sed -i '/|| sym->st_info == 0x12/a\                    || sym->st_info == 0x20\
                    || sym->st_info == 0x21\
                    || sym->st_info == 0x22' tcctools.c
      # Avoid the predecessor's aggregate-copy codegen failure in the first
      # initializer-emission function reached during self-host compilation.
      $BB sed -i 's/dtype = \*type;/memcpy(\&dtype, type, sizeof(CType));/' tccgen.c
      # Imported static objects can carry local GOT relocations without a
      # self-host-created sym_attr.  The relocation itself is authoritative for
      # the GOT offset; validate it directly instead of dereferencing null.
      $BB sed -i '/ST_FUNC void fill_got_entry/,/^}/s/    unsigned offset = attr->got_offset;/    if (!attr) return;\
    unsigned offset = attr->got_offset;/' tccelf.c
      $BB sed -i '/ST_FUNC void relocate_syms/i\
static void bootstrap_undefined_symbol(const char *name)\
{\
    fputs("tcc: undefined symbol: ", stderr);\
    fputs(name, stderr);\
    fputc('\''\\n'\'', stderr);\
    tcc_error_noabort("undefined symbol");\
}\
' tccelf.c
      $BB sed -i 's/tcc_error_noabort("undefined symbol '\''%s'\''", name);/bootstrap_undefined_symbol(name);/g' tccelf.c
      $BB sed -i '/static void fill_local_got_entries/,/^}/c\
static void fill_local_got_entries(TCCState *s1)\
{\
    ElfW_Rel *rel;\
    if (!s1->got || !s1->got->reloc)\
        return;\
    for_each_elem(s1->got->reloc, 0, rel, ElfW_Rel) {\
        if (ELFW(R_TYPE)(rel->r_info) == R_RELATIVE) {\
            int sym_index = ELFW(R_SYM)(rel->r_info);\
            ElfW(Sym) *sym = &((ElfW(Sym) *)symtab_section->data)[sym_index];\
            unsigned long offset;\
            if (rel->r_offset < s1->got->sh_addr) {\
                tcc_error_noabort("local GOT relocation precedes GOT");\
                continue;\
            }\
            offset = rel->r_offset - s1->got->sh_addr;\
            if (offset > s1->got->data_offset ||\
                s1->got->data_offset - offset < PTR_SIZE) {\
                tcc_error_noabort("local GOT relocation exceeds GOT");\
                continue;\
            }\
            rel->r_info = ELFW(R_INFO)(0, R_RELATIVE);\
#if SHT_RELX == SHT_RELA\
            rel->r_addend = sym->st_value;\
#else\
            write32le(s1->got->data + offset, sym->st_value);\
#endif\
        }\
    }\
}' tccelf.c
      # The target is x86_64, where the preceding ten-byte long-double copy is
      # selected.  Remove the dead TinyCC-only x87 fallback whose memory-operand
      # inline assembly crashes this predecessor during self-host compilation.
      $BB sed -i '/else if (sizeof (long double) == sizeof (double))/{N;d;}' tccgen.c
      # The predecessor-specific direct va_arg diagnostic formatter cannot be
      # compiled by that same predecessor.  Restore upstream's bounded
      # vsnprintf forwarding for this self-host pass.
      $BB sed -i '/static void crunch_diag_putc/,/static void strcat_printf/c\
static void strcat_vprintf(char *buf, int buf_size, const char *fmt, va_list ap)\
{\
    int len = strlen(buf);\
    vsnprintf(buf + len, buf_size - len, fmt, ap);\
}\
\
static void strcat_printf(char *buf, int buf_size, const char *fmt, ...)' libtcc.c

      $BB mkdir -p "$out/share/tcc-source-patched" "$out/share/selfhost-link/objects"
      $BB cp -r . "$out/share/tcc-source-patched/"
      echo "tcc-selfhost: compile compiler sources separately"
      MAIN_OBJECT=""
      LIB_OBJECTS=""
      for compiler_source in tcc.c libtcc.c tccpp.c tccgen.c tccelf.c tccasm.c tccrun.c x86_64-gen.c x86_64-link.c i386-asm.c; do
        echo "tcc-selfhost: compile $compiler_source"
        object="${compiler_source%.c}.o"
        "$HOST_TCC/bin/tcc" \
          -c -I . \
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
          -D CONFIG_TCC_LIBPATHS=\"$MUSL/lib:$out/lib/tcc\" \
          -D 'CONFIG_TCC_SYSINCLUDEPATHS="'"$out/lib/tcc/include:$MUSL/include"'"' \
          -D CONFIG_SYSROOT=\"/\" \
          -D TCC_LIBGCC=\"$MUSL/lib/libc.a\" \
          -D TCC_LIBTCC1=\"libtcc1.a\" \
          -D CONFIG_TCCBOOT=1 \
          -D CONFIG_TCC_STATIC=1 \
          -D CONFIG_USE_LIBGCC=1 \
          -D TCC_VERSION=\"0.9.27\" \
          -D ONE_SOURCE=0 \
          -o "$object" \
          "$compiler_source"
        test -s "$object" || { echo "ERROR: self-hosted TinyCC object $object missing" >&2; exit 1; }
        $BB chmod u+rw "$object"
        $BB cp "$object" "$out/share/selfhost-link/objects/$object"
        if [ "$compiler_source" = tcc.c ]; then
          MAIN_OBJECT="$object"
        else
          LIB_OBJECTS="$LIB_OBJECTS $object"
        fi
      done
      test -n "$MAIN_OBJECT" || { echo "ERROR: self-hosted TinyCC main object missing" >&2; exit 1; }
      test -n "$LIB_OBJECTS" || { echo "ERROR: no self-hosted libtcc objects compiled" >&2; exit 1; }
      "$HOST_TCC/bin/tcc" -ar rc libtcc-selfhost.a $LIB_OBJECTS
      test -s libtcc-selfhost.a || { echo "ERROR: self-hosted libtcc archive missing" >&2; exit 1; }

      echo "tcc-selfhost: link source-built objects with declared predecessor linker and Mes runtime"
      $BB cp "$TCC26/lib/mes/crt1.o" mes-crt1.o
      $BB cp "$TCC26/lib/mes/crti.o" mes-crti.o
      $BB cp "$TCC26/lib/mes/crtn.o" mes-crtn.o
      $BB cp "$TCC26/lib/mes/libc.a" mes-libc.a
      $BB cp "$TCC26/lib/mes/tcc/libtcc1.a" mes-libtcc1.a
      "$TCC26/bin/tcc" \
        -nostdlib -static \
        -o tcc-selfhost \
        mes-crt1.o mes-crti.o "$MAIN_OBJECT" libtcc-selfhost.a mes-libc.a mes-libtcc1.a mes-crtn.o
      $BB chmod 755 tcc-selfhost

      $BB mkdir -p "$out/bin" "$out/lib/tcc/include" "$out/share/selfhost-link"
      $BB cp tcc-selfhost "$out/bin/tcc"
      $BB cp tcc-selfhost "$out/bin/tcc-0.9.27-musl-selfhost"
      $BB cp "$HOST_TCC/lib/tcc/libtcc1.a" "$out/lib/tcc/libtcc1.a"
      $BB cp "$MAIN_OBJECT" "$out/share/selfhost-link/tcc.o"
      $BB cp libtcc-selfhost.a "$out/share/selfhost-link/libtcc-selfhost.a"
      $BB cp -r "$SOURCE/include/." "$out/lib/tcc/include/"
      $BB chmod 755 "$out/bin/tcc" "$out/bin/tcc-0.9.27-musl-selfhost"
      test -f "$out/lib/tcc/include/stdarg.h" || { echo "ERROR: self-hosted TinyCC stdarg header missing" >&2; exit 1; }
      test -s "$out/share/selfhost-link/tcc.o" || { echo "ERROR: relinkable TinyCC main object missing" >&2; exit 1; }
      test -s "$out/share/selfhost-link/libtcc-selfhost.a" || { echo "ERROR: relinkable libtcc archive missing" >&2; exit 1; }
      test -s "$out/share/selfhost-link/objects/tccpp.o" || { echo "ERROR: relinkable TinyCC preprocessor object missing" >&2; exit 1; }
      test -f "$out/share/tcc-source-patched/tcc.c" || { echo "ERROR: patched TinyCC source export missing" >&2; exit 1; }

      $BB cat > syntax-smoke.c <<'EOF'
#include <stdarg.h>
struct item { int first; int second; };
static const struct item item = { .second = 2, .first = 1 };
static int first_variadic(int ignored, ...)
{
  va_list arguments;
  va_start(arguments, ignored);
  int value = va_arg(arguments, int);
  va_end(arguments);
  return value;
}
int smoke(void) { return item.first == 1 && item.second == 2 && first_variadic(0, 1) == 1 ? 0 : 1; }
EOF
      "$out/bin/tcc" -c syntax-smoke.c -o syntax-smoke.o
      test -s syntax-smoke.o || { echo "ERROR: self-hosted TinyCC syntax smoke missing" >&2; exit 1; }
    "%,
  ],
  inputs = [stage0, mes, tcc26, host_tcc, target_musl],
} | crunch.Derivation
