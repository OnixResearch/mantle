# Spike proof: create an i386 Mes runtime/header layout for tcc26 -> tcc27.
#
# This is intentionally a sibling diagnostic derivation. It imports the proven
# no-libc i386 TinyCC 0.9.26 handoff and records the first concrete blocker
# before any production make-tcc routing changes.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc26_i386 = import "spike-i386-tinycc26-cross-smoke.ncl" in

let mes_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/mes/mes-0.27.1.tar.gz",
  hash = "sha256-pWARnEISClTfRHAhu69MRaVilq2UFuWsBcKMREsixmk=",
  name = "mes-0.27.1-src",
} in

let tcc27_src = crunch.fetchTarball {
  url = "https://download-mirror.savannah.gnu.org/releases/tinycc/tcc-0.9.27.tar.bz2",
  hash = "sha256-J1qVIv8ti91hOO6Gp2Wqho3t8C7jQHcXpTI85IPwZn0=",
  name = "tcc-0.9.27-src",
} in

let make_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/make/make-3.82.tar.bz2",
  hash = "sha256-GwvLvcEtBMdaC/3YqmUkE9aoNTmrj9yQ39Vyd+Fw+M4=",
  name = "make-3.82-src",
} in

{
  name = "spike-i386-mes-runtime-layout",
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

      STAGE0=$(find_input stage0-posix)
      TCC26=$(find_input spike-i386-tinycc26-cross-smoke)
      MES_SRC=$(find_input mes-0.27.1-src)
      TCC27_SRC=$(find_input tcc-0.9.27-src)
      MAKE_SRC=$(find_input make-3.82-src)
      export PATH="$TCC26/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/spike-i386-mes-runtime-layout
      LOGS=$WORK/logs
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK" "$LOGS"
      cd "$WORK"

      run_step() {
        name="$1"
        shift
        echo "i386-tcc27-make: STEP $name: $*" >&2
        set +e
        "$@" >"$LOGS/$name.stdout" 2>"$LOGS/$name.stderr"
        rc=$?
        set -e
        echo "$rc" >"$LOGS/$name.rc"
        echo "i386-tcc27-make: RESULT $name rc=$rc" >&2
        return "$rc"
      }

      note_blocker() {
        step="$1"
        rc_file="$LOGS/$step.rc"
        rc=missing
        if [ -f "$rc_file" ]; then rc=$($BB cat "$rc_file"); fi
        $BB mkdir -p "$out/share/spike-i386-mes-runtime-layout"
        $BB cp -r "$LOGS" "$out/share/spike-i386-mes-runtime-layout/logs"
        {
          echo "status=blocked"
          echo "blocked_step=$step"
          echo "blocked_rc=$rc"
          echo "note=The i386 Mes runtime/header layout proof found the first concrete blocker before tcc27 can use the layout."
        } > "$out/share/spike-i386-mes-runtime-layout/summary.txt"
      }

      echo 'i386-mes-layout: predecessor version' >&2
      run_step tcc26_version tcc26-i386 -version

      RUNTIME="$WORK/i386-mes-runtime"
      MES_BUILD="$WORK/mes-0.27.1"
      $BB cp -r "$MES_SRC" "$MES_BUILD"
      $BB chmod -R u+w "$MES_BUILD"
      $BB mkdir -p "$RUNTIME/include/mes" "$RUNTIME/lib/mes/tcc"
      $BB cp -r "$MES_BUILD/include" "$RUNTIME/include/mes"
      $BB mkdir -p "$RUNTIME/include/mes/include/arch"
      $BB cp "$MES_BUILD/include/linux/x86/kernel-stat.h" "$RUNTIME/include/mes/include/arch/kernel-stat.h"
      $BB cp "$MES_BUILD/include/linux/x86/signal.h" "$RUNTIME/include/mes/include/arch/signal.h"
      $BB cp "$MES_BUILD/include/linux/x86/syscall.h" "$RUNTIME/include/mes/include/arch/syscall.h"
      $BB cat > "$RUNTIME/include/mes/include/mes/config.h" << 'EOF'
#undef SYSTEM_LIBC
#define MES_VERSION "0.27.1"
EOF

      run_step runtime_header_smoke test -f "$RUNTIME/include/mes/include/stdio.h"
      if ! run_step runtime_crt1_object tcc26-i386 -c -D HAVE_CONFIG_H=1 -I "$RUNTIME/include/mes/include" -I "$RUNTIME/include/mes/include/linux/x86" -o "$RUNTIME/lib/mes/crt1.o" "$MES_BUILD/lib/linux/x86-mes-gcc/crt1.c"; then
        note_blocker runtime_crt1_object
        exit 0
      fi
      : > "$RUNTIME/lib/mes/crti.o"
      : > "$RUNTIME/lib/mes/crtn.o"
      if ! run_step runtime_libtcc1_object tcc26-i386 -c -D HAVE_CONFIG_H=1 -D TCC_TARGET_I386=1 -D HAVE_FLOAT=1 -I "$RUNTIME/include/mes/include" -I "$RUNTIME/include/mes/include/linux/x86" -o "$WORK/libtcc1.o" "$MES_BUILD/lib/libtcc1.c"; then
        echo 'i386-mes-layout: libtcc1 object is blocked; create placeholder archives to continue header-only tcc27 object probe' >&2
        $BB cp "$RUNTIME/lib/mes/crt1.o" "$WORK/libtcc1.o"
      fi
      if ! run_step runtime_libtcc1_archive tcc26-i386 -ar cr "$RUNTIME/lib/mes/tcc/libtcc1.a" "$WORK/libtcc1.o"; then
        note_blocker runtime_libtcc1_archive
        exit 0
      fi
      $BB cp "$RUNTIME/lib/mes/tcc/libtcc1.a" "$RUNTIME/lib/mes/libc.a"

      $BB cp -r "$TCC27_SRC" tcc-0.9.27
      $BB chmod -R u+w tcc-0.9.27
      cd tcc-0.9.27
      : > config.h

      # Live-bootstrap source edits plus Crunch Mes/TCC normalization seams.
      $BB sed -i '/if ((fh = fopen(argv\[i_lib\], "wb")) == NULL)/,/    }/d' tcctools.c
      $BB sed -i '/^[[:space:]]*\/\/ write header/i\
    if ((fh = fopen(argv[i_lib], "wb")) == NULL)\
    {\
        fprintf(stderr, "tcc: ar: can'\''t open file %s \\n", argv[i_lib]);\
        goto the_end;\
    }\
' tcctools.c
      $BB sed -i '/if (ph->p_offset == 0) {/,/                    }/s/ph->p_paddr = ph->p_vaddr;/ph->p_paddr = ph->p_vaddr;\
                        if (s1->text_addr == 0x80100000)\
                            ph->p_paddr = ph->p_paddr - 0x80000000;/' tccelf.c
      $BB sed -i 's/size = size \* 2;/size = size + size;/g' tccpp.c tccelf.c
      $BB sed -i 's/nb_alloc = nb \* 2;/nb_alloc = nb + nb;/' libtcc.c
      $BB sed -i 's/next->size \* 2/next->size + next->size/' tccpp.c
      $BB sed -i 's/rebuild_hash(s, 2 \* nbuckets);/rebuild_hash(s, nbuckets + nbuckets);/' tccelf.c
      $BB sed -i 's/if (!is_compatible_types(\&sym->type, type))/if (0 \&\& !is_compatible_types(\&sym->type, type))/' tccgen.c
      $BB sed -i '/snprintf(buf, sizeof(buf),/s|.*|    strcpy(buf, name);|' tcc.c
      $BB sed -i '/snprintf(buf, sizeof(buf), fmt, paths\[i\], filename);/c\
        if (fmt[3] == '\''l'\'') {\
            strcpy(buf, paths[i]); strcat(buf, "/lib"); strcat(buf, filename); strcat(buf, ".a");\
        } else {\
            strcpy(buf, paths[i]); strcat(buf, "/"); strcat(buf, filename);\
        }' libtcc.c
      $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
      $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
      $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
      $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rel"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__start_%s", s->name);|strcpy(buf, "__start_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__stop_%s", s->name);|strcpy(buf, "__stop_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(sym_start, sizeof(sym_start), "__%s_start", section_name + 1);|strcpy(sym_start, "__"); strcat(sym_start, section_name + 1); strcat(sym_start, "_start");|' tccelf.c
      $BB sed -i 's|snprintf(sym_end, sizeof(sym_end), "__%s_end", section_name + 1);|strcpy(sym_end, "__"); strcat(sym_end, section_name + 1); strcat(sym_end, "_end");|' tccelf.c

      # First prove the predecessor can at least parse/compile 0.9.27 to an i386 object.
      if ! run_step tcc27_compile_object tcc26-i386 -c -o tcc27.o -I . -I "$RUNTIME/include/mes/include" -D BOOTSTRAP=1 -D HAVE_FLOAT=1 -D HAVE_BITFIELD=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_I386=1 -D CONFIG_TCCDIR=\"$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_CRTPREFIX=\"$RUNTIME/lib/mes\" -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" -D CONFIG_TCC_LIBPATHS=\"$RUNTIME/lib/mes:$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_SYSINCLUDEPATHS=\"$RUNTIME/include/mes/include\" -D TCC_LIBGCC=\"$RUNTIME/lib/mes/libc.a\" -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-i386-spike\" -D ONE_SOURCE=1 tcc.c; then
        cd "$WORK"
        note_blocker tcc27_compile_object
        exit 0
      fi

      if ! run_step tcc27_link tcc26-i386 -static -o tcc27-i386 -I . -I "$RUNTIME/include/mes/include" -D BOOTSTRAP=1 -D HAVE_FLOAT=1 -D HAVE_BITFIELD=1 -D HAVE_LONG_LONG=1 -D HAVE_SETJMP=1 -D TCC_TARGET_I386=1 -D CONFIG_TCCDIR=\"$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_CRTPREFIX=\"$RUNTIME/lib/mes\" -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" -D CONFIG_TCC_LIBPATHS=\"$RUNTIME/lib/mes:$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_SYSINCLUDEPATHS=\"$RUNTIME/include/mes/include\" -D TCC_LIBGCC=\"$RUNTIME/lib/mes/libc.a\" -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-i386-spike\" -D ONE_SOURCE=1 tcc.c; then
        cd "$WORK"
        note_blocker tcc27_link
        exit 0
      fi
      $BB chmod 755 tcc27-i386
      if ! run_step tcc27_version ./tcc27-i386 -version; then
        cd "$WORK"
        note_blocker tcc27_version
        exit 0
      fi

      cd "$WORK"
      $BB cp -r "$MAKE_SRC" make-3.82
      $BB chmod -R u+w make-3.82
      cd make-3.82
      : > config.h
      if ! run_step make_compile_getopt ../tcc-0.9.27/tcc27-i386 -c getopt.c; then
        cd "$WORK"
        note_blocker make_compile_getopt
        exit 0
      fi

      cd "$WORK"
      $BB mkdir -p "$out/bin" "$out/share/spike-i386-mes-runtime-layout"
      $BB cp tcc-0.9.27/tcc27-i386 "$out/bin/tcc27-i386" 2>/dev/null || true
      $BB cp -r "$LOGS" "$out/share/spike-i386-mes-runtime-layout/logs"
      printf 'status=partial\nblocked_step=make_full_smoke_not_yet_implemented\n' > "$out/share/spike-i386-mes-runtime-layout/summary.txt"
    "%,
  ],
  inputs = [stage0, tcc26_i386, mes_src, tcc27_src, make_src],
} | crunch.Derivation
