# Spike proof: try the i386 live-bootstrap sequence tcc-0.9.26 -> tcc-0.9.27 -> make-3.82.
#
# This is intentionally a sibling diagnostic derivation. It imports the proven
# i386 Mes runtime and TinyCC 0.9.27 object handoff, then records the first
# concrete link or GNU Make blocker without changing production make-tcc
# routing.

let crunch = import "lib.ncl" in

let stage0 = import "stage0-posix.ncl" in
let tcc26_i386 = import "spike-i386-tinycc26-cross-smoke.ncl" in
let mes_runtime = import "spike-i386-mes-runtime-layout.ncl" in

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
  name = "spike-i386-tcc27-make-pass1",
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
      MES_RUNTIME=$(find_input spike-i386-mes-runtime-layout)
      TCC27_SRC=$(find_input tcc-0.9.27-src)
      MAKE_SRC=$(find_input make-3.82-src)
      RUNTIME="$MES_RUNTIME/share/spike-i386-mes-runtime-layout/runtime"
      RUNTIME_SUMMARY="$MES_RUNTIME/share/spike-i386-mes-runtime-layout/summary.txt"
      export PATH="$TCC26/bin:$STAGE0/bin:$PATH"

      WORK=/tmp/spike-i386-tcc27-make-pass1
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
        $BB mkdir -p "$out/share/spike-i386-tcc27-make-pass1"
        $BB cp -r "$LOGS" "$out/share/spike-i386-tcc27-make-pass1/logs"
        {
          echo "status=blocked"
          echo "blocked_step=$step"
          echo "blocked_rc=$rc"
          echo "note=The validated i386 Mes runtime handoff was consumed; this is the first remaining tcc27 or GNU Make blocker."
        } > "$out/share/spike-i386-tcc27-make-pass1/summary.txt"
      }

      validate_runtime_handoff() {
        $BB grep -qx 'status=object-emission-success' "$RUNTIME_SUMMARY" &&
        $BB grep -qx 'runtime_libtcc1_object=real' "$RUNTIME_SUMMARY" &&
        $BB grep -qx 'runtime_libtcc1_archive=real' "$RUNTIME_SUMMARY" &&
        $BB grep -qx 'runtime_libc_archive=real' "$RUNTIME_SUMMARY" &&
        test -f "$RUNTIME/lib/mes/crt1.o" &&
        test -f "$RUNTIME/lib/mes/libc.a" &&
        test -f "$RUNTIME/lib/mes/tcc/libtcc1.a" &&
        test -f "$RUNTIME/include/mes/include/stdio.h"
      }

      if ! run_step runtime_handoff_validate validate_runtime_handoff; then
        note_blocker runtime_handoff_validate
        exit 0
      fi

      echo 'i386-tcc27-make: predecessor version' >&2
      run_step tcc26_version tcc26-i386 -version

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
      # live-bootstrap tcc-0.9.27/pass1 check-reloc-null patch.
      $BB sed -i '/static void fill_local_got_entries(TCCState \*s1)/,/for_each_elem(s1->got->reloc, 0, rel, ElfW_Rel) {/s/    ElfW_Rel \*rel;/    ElfW_Rel *rel;\
    if (!s1->got->reloc)\
        return;/' tccelf.c
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
      $BB sed -i '/vsnprintf(buf + len/s|.*|    strcat(buf, fmt);|' libtcc.c
      $BB sed -i '/default case: stderr/{n;n;n;n;s|.*|        fputs(buf, stderr); fputs("\\n", stderr);|;}' libtcc.c
      $BB sed -i '/default case: stderr/,/fflush(stderr)/s|^[[:space:]]*fprintf(stderr, .*|        fputs(buf, stderr); fputs("\\n", stderr);|' libtcc.c
      $BB sed -i '/s1->error_set_jmp_enabled = 0;/a\    if (s1->nb_errors != 0) return -1;' libtcc.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), REL_SECTION_FMT, s->name);|strcpy(buf, ".rel"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__start_%s", s->name);|strcpy(buf, "__start_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(buf, sizeof(buf), "__stop_%s", s->name);|strcpy(buf, "__stop_"); strcat(buf, s->name);|' tccelf.c
      $BB sed -i 's|snprintf(sym_start, sizeof(sym_start), "__%s_start", section_name + 1);|strcpy(sym_start, "__"); strcat(sym_start, section_name + 1); strcat(sym_start, "_start");|' tccelf.c
      $BB sed -i 's|snprintf(sym_end, sizeof(sym_end), "__%s_end", section_name + 1);|strcpy(sym_end, "__"); strcat(sym_end, section_name + 1); strcat(sym_end, "_end");|' tccelf.c
      $BB awk '/tcc_error_noabort\("undefined symbol/ { print "                fputs(\"undefined symbol \", stderr); fputs(name, stderr); fputs(\"\\\\n\", stderr); s1->nb_errors++;"; next } { print }' tccelf.c > tccelf.c.new
      $BB mv tccelf.c.new tccelf.c
      $BB awk 'done == 0 && $0 == "#if defined TCC_IS_NATIVE_387" { print "#if 0 /* CRUNCH: tcc26-i386 crashes compiling native x87 long-double initializer path */"; done = 1; next } { print }' tccgen.c > tccgen.c.new
      $BB mv tccgen.c.new tccgen.c

      # Reproduce the validated real-runtime object handoff with stable provider paths.
      if ! run_step tcc27_compile_object tcc26-i386 -c -I . -I "$RUNTIME/include/mes/include" -D TCC_TARGET_I386=1 -D CONFIG_TCCDIR=\"$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_CRTPREFIX=\"$RUNTIME/lib/mes\" -D CONFIG_TCC_ELFINTERP=\"/mes/loader\" -D CONFIG_TCC_LIBPATHS=\"$RUNTIME/lib/mes:$RUNTIME/lib/mes/tcc\" -D CONFIG_TCC_SYSINCLUDEPATHS=\"$RUNTIME/include/mes/include\" -D TCC_LIBGCC=\"$RUNTIME/lib/mes/libc.a\" -D CONFIG_TCC_STATIC=1 -D CONFIG_USE_LIBGCC=1 -D TCC_VERSION=\"0.9.27-i386-spike\" -D ONE_SOURCE=1 tcc.c -o tcc27.o; then
        cd "$WORK"
        note_blocker tcc27_compile_object
        exit 0
      fi
      TCC27_OBJECT="$WORK/tcc-0.9.27/tcc27.o"
      if ! run_step tcc27_object_exists test -f "$TCC27_OBJECT"; then
        cd "$WORK"
        note_blocker tcc27_object_exists
        exit 0
      fi

      if ! run_step tcc27_link tcc26-i386 -nostdlib -static -o tcc27-i386 "$RUNTIME/lib/mes/crt1.o" "$TCC27_OBJECT" "$RUNTIME/lib/mes/tcc/libtcc1.a" "$RUNTIME/lib/mes/libc.a"; then
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
      $BB mkdir -p "$out/bin" "$out/share/spike-i386-tcc27-make-pass1"
      $BB cp tcc-0.9.27/tcc27-i386 "$out/bin/tcc27-i386" 2>/dev/null || true
      $BB cp -r "$LOGS" "$out/share/spike-i386-tcc27-make-pass1/logs"
      printf 'status=partial\nblocked_step=make_full_smoke_not_yet_implemented\n' > "$out/share/spike-i386-tcc27-make-pass1/summary.txt"
    "%,
  ],
  inputs = [stage0, tcc26_i386, mes_runtime, tcc27_src, make_src],
} | crunch.Derivation
