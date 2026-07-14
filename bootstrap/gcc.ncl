# Build GCC from source using the bootstrap toolchain.
#
# Part of the Nix-free bootstrap chain (Phase 4):
#   seed → make → dash → binutils → musl → gcc (this file)
#
# This builds a minimal C-only GCC targeting musl libc. Combined with
# the from-source binutils and musl, this gives a fully from-source
# toolchain — the reduced bootstrap seed is no longer needed for
# subsequent builds.
#
# GCC needs three math libraries (GMP, MPFR, MPC) which are fetched
# separately and symlinked into the source tree for in-tree building.
#
# Usage:
#   crunch build bootstrap/gcc.ncl -I lib --store /tmp/crunch-store

let crunch = import "lib.ncl" in

let seed = import "seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in

let gnumake = (import "make.ncl") in
let dash = (import "dash.ncl") in
let binutils = (import "binutils.ncl") in
let musl = (import "musl.ncl") in

let gcc_src = crunch.fetchTarball {
  # ftpmirror.gnu.org returned repeated 502s during release proof replay;
  # use the stable kernel.org GNU mirror for the same fixed-output tarball.
  url = "https://mirrors.kernel.org/gnu/gcc/gcc-13.3.0/gcc-13.3.0.tar.gz",
  hash = "sha256-WWameciUJMmYPeqfDIPa3YmPZaZUuB6r6XpwlfjRXo8=",
  name = "gcc-src",
} in

let gmp_src = crunch.fetchTarball {
  # ftpmirror.gnu.org returned repeated 502s during release proof replay;
  # use the stable kernel.org GNU mirror for the same fixed-output tarball.
  url = "https://mirrors.kernel.org/gnu/gmp/gmp-6.2.1.tar.bz2",
  hash = "sha256-69A2yy9AY8/OFO5FfNmsvd01vTknitC/wrQSjaZq86A=",
  name = "gmp-src",
} in

let mpfr_src = crunch.fetchTarball {
  # ftpmirror.gnu.org returned repeated 502s during release proof replay;
  # use the stable kernel.org GNU mirror for the same fixed-output tarball.
  url = "https://mirrors.kernel.org/gnu/mpfr/mpfr-4.1.0.tar.bz2",
  hash = "sha256-Q8xRKVAvE1c0uPSWy/Kav0Hfw0UxlnWzePXE13u6sDw=",
  name = "mpfr-src",
} in

let mpc_src = crunch.fetchTarball {
  # ftpmirror.gnu.org returned repeated 502s during release proof replay;
  # use the stable kernel.org GNU mirror for the same fixed-output tarball.
  url = "https://mirrors.kernel.org/gnu/mpc/mpc-1.2.1.tar.gz",
  hash = "sha256-nyt5XHt58wWilbLRlIDuECmD3qBqmKTx4jiGL/7n5Ws=",
  name = "mpc-src",
} in

{
  name = "gcc",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      BB=/bin/busybox
      $BB mkdir -p /tmp/tools
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink sleep touch date od \
                 uname id hostname diff cmp true false tee nl \
                 cut printf dd env seq tar gzip gunzip bzip2 \
                 mktemp rmdir uniq; do
        $BB ln -sf $BB /tmp/tools/$cmd
      done

      # Find bootstrap tools.
      TOOLCHAIN=""
      for d in $NIX_STORE/*-%{seed_name}; do
        if [ -d "$d/bin" ]; then TOOLCHAIN="$d"; break; fi
      done
      MAKE=""
      for d in $NIX_STORE/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d/bin/make"; break; fi
      done
      DASH=""
      for d in $NIX_STORE/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d/bin/dash"; break; fi
      done
      BINUTILS=""
      for d in $NIX_STORE/*-binutils; do
        if [ -d "$d/bin" ]; then BINUTILS="$d"; break; fi
      done
      MUSL=""
      for d in $NIX_STORE/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done

      for tool in TOOLCHAIN MAKE DASH BINUTILS MUSL; do
        eval val=\$$tool
        if [ -z "$val" ]; then
          echo "ERROR: $tool not found" >&2; exit 1
        fi
        echo "$tool=$val"
      done

      $BB ln -sf "$DASH" /tmp/tools/sh

      DETERMINISTIC_EPOCH=1
      DETERMINISTIC_TOUCH_STAMP=197001010000.01
      DETERMINISTIC_UMASK=022
      export SOURCE_DATE_EPOCH="$DETERMINISTIC_EPOCH"
      export TZ=UTC
      export LANG=C
      export LC_ALL=C
      export ZERO_AR_DATE=1
      export ARFLAGS=crD
      export HOSTNAME=mantle-bootstrap
      export USER=mantle
      export LOGNAME=mantle
      umask "$DETERMINISTIC_UMASK"

      deterministic_touch() {
        path="$1"
        touch -d "@$SOURCE_DATE_EPOCH" "$path" 2>/dev/null || \
          touch -t "$DETERMINISTIC_TOUCH_STAMP" "$path"
      }

      deterministic_touch_tree() {
        root="$1"
        pattern="$2"
        find "$root" -name "$pattern" -exec touch -d "@$SOURCE_DATE_EPOCH" {} + 2>/dev/null || \
          find "$root" -name "$pattern" -exec touch -t "$DETERMINISTIC_TOUCH_STAMP" {} + 2>/dev/null
      }

      CONFIG_LOG_TAIL_LINES=400
      BUILD_JOBS=4

      dump_gcc_failure_logs() {
        phase="$1"
        echo "=== gcc failure diagnostics: $phase ===" >&2
        for log in \
          /tmp/gcc-build/config.log \
          /tmp/gcc-build/gcc/config.log \
          /tmp/gcc-build/x86_64-linux-musl/libgcc/config.log; do
          if [ -f "$log" ]; then
            echo "--- $log (tail -$CONFIG_LOG_TAIL_LINES) ---" >&2
            tail -n "$CONFIG_LOG_TAIL_LINES" "$log" >&2 || true
          else
            echo "--- $log missing ---" >&2
          fi
        done
      }

      normalize_output_metadata() {
        root="$1"
        find "$root" -type d -exec chmod 755 {} + 2>/dev/null || true
        find "$root" -type f -exec chmod u+rw,go+r {} + 2>/dev/null || true
        find "$root" -exec touch -d "@$SOURCE_DATE_EPOCH" {} + 2>/dev/null || \
          find "$root" -exec touch -t "$DETERMINISTIC_TOUCH_STAMP" {} + 2>/dev/null || true
      }

      remove_generated_manpages() {
        # Generated documentation is outside the bootstrap compiler contract.
        # Exclude it rather than replacing nondeterministic pages with content
        # that could be mistaken for real GCC documentation.
        $BB rm -rf "$out/share/man"
      }

      # egrep/fgrep wrappers. Must use /tmp/tools/sh as interpreter.
      echo '#!/tmp/tools/sh' > /tmp/tools/egrep
      echo 'exec /tmp/tools/grep -E "$@"' >> /tmp/tools/egrep
      $BB chmod +x /tmp/tools/egrep
      echo '#!/tmp/tools/sh' > /tmp/tools/fgrep
      echo 'exec /tmp/tools/grep -F "$@"' >> /tmp/tools/fgrep
      $BB chmod +x /tmp/tools/fgrep

      # GCC's release tarball carries pre-generated flex output for
      # gengtype. Copy that deterministic source instead of requiring flex in
      # the stage0 sandbox.
      $BB cat > /tmp/tools/flex << 'FLEXEOF'
#!/tmp/tools/sh
out=""
input=""
for arg in "$@"; do
  case "$arg" in
    -o*) out="${arg#-o}" ;;
    *) input="$arg" ;;
  esac
done
if [ -z "$out" ]; then
  echo "flex shim: missing -o output" >&2
  exit 1
fi
if [ -z "$input" ]; then
  echo "flex shim: missing input" >&2
  exit 1
fi
case "$(basename "$input")" in
  gengtype-lex.l)
    cp "$(dirname "$input")/gengtype-lex.cc" "$out"
    ;;
  *)
    echo "flex shim: unsupported input $input" >&2
    exit 1
    ;;
esac
FLEXEOF
      $BB chmod +x /tmp/tools/flex

      # Find source tarballs.
      GCC_SRC="" GMP_SRC="" MPFR_SRC="" MPC_SRC=""
      for d in $NIX_STORE/*-gcc-src; do
        if [ -d "$d" ]; then GCC_SRC="$d"; fi
      done
      for d in $NIX_STORE/*-gmp-src; do
        if [ -d "$d" ]; then GMP_SRC="$d"; fi
      done
      for d in $NIX_STORE/*-mpfr-src; do
        if [ -d "$d" ]; then MPFR_SRC="$d"; fi
      done
      for d in $NIX_STORE/*-mpc-src; do
        if [ -d "$d" ]; then MPC_SRC="$d"; fi
      done

      for src in GCC_SRC GMP_SRC MPFR_SRC MPC_SRC; do
        eval val=\$$src
        if [ -z "$val" ]; then
          echo "ERROR: $src not found" >&2; exit 1
        fi
      done

      # The stage-built binutils are musl-dynamic and reference
      # /lib/ld-musl-x86_64.so.1, which is not mounted in the sandbox.
      # Expose them through stable target-prefixed wrappers that launch them
      # with the stage-built musl loader instead of falling back to seed tools.
      REQUIRED_STAGE_BINUTILS="ar as ld nm objcopy objdump ranlib readelf strip"
      OPTIONAL_STAGE_BINUTILS="ld.bfd size strings"
      STAGE_BINUTILS_LOADER="$MUSL/lib/libc.so"
      for tool in $REQUIRED_STAGE_BINUTILS $OPTIONAL_STAGE_BINUTILS; do
        stage_tool="$BINUTILS/bin/%{seed_target}-$tool"
        wrapper="/tmp/tools/%{seed_target}-$tool"
        if [ ! -x "$stage_tool" ]; then
          case " $REQUIRED_STAGE_BINUTILS " in
            *" $tool "*) echo "ERROR: required stage binutils tool missing: $stage_tool" >&2; exit 1 ;;
            *) continue ;;
          esac
        fi
        $BB cat > "$wrapper" << WRAPEOF
#!/tmp/tools/sh
exec "$STAGE_BINUTILS_LOADER" "$stage_tool" "\$@"
WRAPEOF
        $BB chmod +x "$wrapper"
      done

      export PATH="/tmp/tools:$TOOLCHAIN/bin:$BINUTILS/bin"
      export SHELL="$DASH"
      export CONFIG_SHELL="$DASH"

      # Copy gcc source to writable /tmp and symlink prerequisites.
      $BB cp -r "$GCC_SRC" /tmp/gcc-src 2>/dev/null
      $BB chmod -R u+w /tmp/gcc-src

      # Handle fetchTarball not stripping top-level dir.
      if [ -f /tmp/gcc-src/configure ]; then
        GSRC=/tmp/gcc-src
      else
        for sub in /tmp/gcc-src/*/configure; do
          GSRC=$(dirname "$sub"); break
        done
      fi

      # Copy in-tree prerequisites (same layout as download_prerequisites),
      # but keep them writable so the sandbox can remove unavailable demo
      # subdirectories without mutating read-only store inputs.
      for lib in gmp mpfr mpc; do
        eval srcdir=\$$(echo $lib | tr a-z A-Z)_SRC
        prereq_src=""
        # Find the actual source dir (may have version subdir)
        if [ -f "$srcdir/configure" ]; then
          prereq_src="$srcdir"
        else
          for sub in $srcdir/*/configure; do
            prereq_src="$(dirname "$sub")"
            break
          done
        fi
        if [ -z "$prereq_src" ]; then
          echo "ERROR: prerequisite source not found for $lib" >&2
          exit 1
        fi
        $BB rm -rf "$GSRC/$lib"
        $BB cp -r "$prereq_src" "$GSRC/$lib" 2>/dev/null
        $BB chmod -R u+w "$GSRC/$lib"
        deterministic_touch_tree "$GSRC/$lib" '*'
        echo "Copied $lib -> $prereq_src"
      done

      # Touch pre-generated flex/yacc/gperf files so make doesn't
      # try to regenerate them (we have no flex/bison/gperf), using a
      # fixed timestamp so source freshness does not leak host time.
      deterministic_touch_tree "$GSRC" '*.cc'
      deterministic_touch_tree "$GSRC" '*.c'
      deterministic_touch_tree "$GSRC" '*.h'

      mkdir -p /tmp/gcc-build
      deterministic_touch /tmp/gcc-build
      cd /tmp/gcc-build

      # Create a CC wrapper that always passes -static. Inside the
      # bwrap sandbox, there's no dynamic linker at /lib/ so all
      # configure test binaries and build tools must be static.
      cat > /tmp/tools/seed-cc-static << WRAPEOF
#!/bin/sh
exec $TOOLCHAIN/bin/%{seed_target}-gcc -static "\$@"
WRAPEOF
      chmod +x /tmp/tools/seed-cc-static

      cat > /tmp/tools/seed-cxx-static << WRAPEOF
#!/bin/sh
exec $TOOLCHAIN/bin/%{seed_target}-g++ -static "\$@"
WRAPEOF
      chmod +x /tmp/tools/seed-cxx-static

      echo "=== Configuring gcc ==="
      "$DASH" "$GSRC/configure" \
        --prefix=$out \
        --build=%{seed_target} \
        --host=%{seed_target} \
        --target=%{seed_target} \
        --enable-languages=c \
        --disable-bootstrap \
        --disable-multilib \
        --disable-nls \
        --disable-werror \
        --disable-libssp \
        --disable-libgomp \
        --disable-libquadmath \
        --disable-libstdcxx \
        --disable-libatomic \
        --disable-libvtv \
        --disable-libsanitizer \
        --disable-shared \
        --disable-threads \
        --disable-plugin \
        --disable-lto \
        --disable-decimal-float \
        --disable-fixed-point \
        --with-sysroot="$MUSL" \
        --with-native-system-header-dir=/include \
        --with-build-sysroot="$MUSL" \
        CC="/tmp/tools/seed-cc-static" \
        CXX="/tmp/tools/seed-cxx-static" \
        AR="/tmp/tools/%{seed_target}-ar" \
        RANLIB="/tmp/tools/%{seed_target}-ranlib" \
        AS="/tmp/tools/%{seed_target}-as" \
        LD="/tmp/tools/%{seed_target}-ld" \
        NM="/tmp/tools/%{seed_target}-nm" \
        CFLAGS="-O2" \
        CXXFLAGS="-O2" \
        LDFLAGS="-static" \
        GREP="/tmp/tools/grep" \
        EGREP="/tmp/tools/grep -E" \
        FGREP="/tmp/tools/grep -F" \
        SED="/tmp/tools/sed" \
        AWK="/tmp/tools/awk" \
        am_cv_ar_interface=ar \
        2>&1 || { dump_gcc_failure_logs configure; exit 1; }

      # gmp demos need bison/yacc which are unavailable in the sandbox;
      # remove demos from gmp's SUBDIRS in Makefile.in so configure never
      # creates the demos build directory (make re-runs sub-configures)
      if [ -f /tmp/gcc-src/gmp/Makefile.in ]; then
        $BB sed -i 's/demos//g' /tmp/gcc-src/gmp/Makefile.in
      fi
      if [ -f /tmp/gcc-src/gmp/Makefile.am ]; then
        $BB sed -i 's/demos//g' /tmp/gcc-src/gmp/Makefile.am
      fi

      echo "=== Building gcc ==="
      "$MAKE" -j"$BUILD_JOBS" SHELL="$DASH" MAKEINFO=true MISSING=true ARFLAGS="$ARFLAGS" \
        2>&1 || { dump_gcc_failure_logs build; exit 1; }

      echo "=== Installing gcc ==="
      "$MAKE" install SHELL="$DASH" MAKEINFO=true MISSING=true ARFLAGS="$ARFLAGS" \
        2>&1 || { dump_gcc_failure_logs install; exit 1; }

      remove_generated_manpages
      normalize_output_metadata "$out"
      mkdir -p "$out/share/mantle"
      cat > "$out/share/mantle/bootstrap-determinism.json" << DETERMINISMEOF
{"schema":"mantle-bootstrap-determinism-v1","source_date_epoch":"$SOURCE_DATE_EPOCH","timezone":"$TZ","locale":"$LC_ALL","umask":"$DETERMINISTIC_UMASK","archive_flags":"$ARFLAGS","metadata_normalized":true}
DETERMINISMEOF
      deterministic_touch "$out/share/mantle/bootstrap-determinism.json"

      echo "=== Verify ==="
      $out/bin/%{seed_target}-gcc --version | head -1
      ls $out/bin/
    "%,
  ],
  inputs = [toolchain, gnumake, dash, binutils, musl, gcc_src, gmp_src, mpfr_src, mpc_src],
} | crunch.Derivation
