# Normalized direct-input bundle for conventional source generators.
#
# Mantle build sandboxes expose direct inputs, not arbitrary transitive inputs.
# This derivation copies the closed GCC 4.0/pass4, musl, binutils, Make,
# stage0, StageX sed/diffutils, and config.sub surfaces once so later generator
# stages can retain a linear derivation DAG without recursively flattening
# `.inputs`.

let crunch = import "lib.ncl" in
let stage0 = crunch.derivationFile "stage0-posix.ncl" in
let make = crunch.derivationFile "make-tcc.ncl" in
let gcc = crunch.derivationFile "gcc-4.0-musl-pass4.ncl" in
let musl = crunch.derivationFile "musl-1.1.24-gcc-pass4.ncl" in
let binutils = crunch.derivationFile "binutils-2.30-gcc.ncl" in
let stagex_transition = import "stagex-transition-proof-input.ncl" in
let automake115_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/automake/automake-1.15.1.tar.xz",
  hash = "sha256-/KHwBUWoEgUxjLuLFOpqM4c0UXUDWiyiBV4HBTwOcZU=",
  name = "automake-1.15.1-config-sub-src",
} in

{
  name = "gcc-generator-base-v4",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      TARGET=x86_64-unknown-linux-musl
      GCC_VERSION=4.0.4
      WORK=/tmp/gcc-generator-base
      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK"

      find_input() {
        input_suffix=$1
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            echo "$input_path"
            return 0
          fi
        done
        echo "ERROR: input $input_suffix not found" >&2
        exit 1
      }

      GCC=$(find_input gcc-4.0.4-musl-pass4-v5)
      MUSL=$(find_input musl-1.1.24-gcc-pass4-v2)
      BINUTILS=$(find_input binutils-2.30-gcc-pass4-v16)
      MAKE=$(find_input make-3.82-tcc)
      STAGE0=$(find_input stage0-posix)
      TRANSITION=$(find_input mantle-stagex-transition)
      SED="$TRANSITION/sed-stage/runtime/output"
      DIFFUTILS="$TRANSITION/diffutils-stage/runtime/output"
      AUTOMAKE115_SRC=$(find_input automake-1.15.1-config-sub-src)

      test -x "$GCC/bin/gcc"
      test -x "$GCC/libexec/gcc/$TARGET/$GCC_VERSION/cc1"
      test -s "$MUSL/lib/libc.a"
      for required_tool in as ld ar ranlib readelf; do
        test -x "$BINUTILS/bin/$required_tool"
      done
      test -x "$MAKE/bin/make"
      test -x "$SED/bin/sed"
      test -x "$DIFFUTILS/bin/diff"
      test -s "$AUTOMAKE115_SRC/lib/config.sub"

      $BB mkdir -p "$out"
      $BB cp -R "$GCC" "$out/gcc"
      $BB cp -R "$MUSL" "$out/musl"
      $BB cp -R "$BINUTILS" "$out/binutils"
      $BB cp -R "$MAKE" "$out/make"
      $BB cp -R "$STAGE0" "$out/stage0"
      $BB cp -R "$SED" "$out/sed"
      $BB cp -R "$DIFFUTILS" "$out/diffutils"
      $BB chmod -R u+w "$out/gcc"

      # Normalize the copied GCC assembler/linker seams. The GCC driver still
      # emits its configured predecessor libgcc path even with relocated -B
      # prefixes, so the provider-local linker must replace both `-lgcc` and
      # absolute predecessor archive arguments.
      $BB cat > "$out/gcc/bin/as" <<EOF
#!/bin/sh
exec "$out/binutils/bin/as" "\$@"
EOF
      $BB cat > "$out/gcc/bin/ld" <<EOF
#!/bin/sh
libc_archive="\${MANTLE_GCC_LINK_LIBC:-$out/musl/lib/libc.a}"
libm_archive="\${MANTLE_GCC_LINK_LIBM:-$out/musl/lib/libm.a}"
libgcc_archive="$out/gcc/lib/gcc/$TARGET/$GCC_VERSION/libgcc.a"
crt_dir="\${MANTLE_GCC_LINK_CRT:-$out/musl/lib}"
linker="\${MANTLE_GCC_LINKER:-$out/binutils/bin/ld}"
link_arguments=
library_dirs=
expect_library_dir=false
for link_argument in "\$@"; do
  if [ "\$expect_library_dir" = true ]; then
    library_dirs="\$library_dirs \$link_argument"
    expect_library_dir=false
    continue
  fi
  case "\$link_argument" in
    -L) expect_library_dir=true ;;
    -L*) library_dirs="\$library_dirs \${link_argument#-L}" ;;
    -lgcc|-lgcc_eh|*/libgcc.a|*/libgcc_eh.a) link_arguments="\$link_arguments \$libgcc_archive" ;;
    -lgcc_s|*/libgcc_s.so*) ;;
    crt1.o|crti.o|crtn.o|*/crt1.o|*/crti.o|*/crtn.o)
      crt_name="\${link_argument##*/}"
      link_arguments="\$link_arguments \$crt_dir/\$crt_name"
      ;;
    -lc) link_arguments="\$link_arguments \$libc_archive" ;;
    -lm) link_arguments="\$link_arguments \$libm_archive" ;;
    -l*)
      library_name="\${link_argument#-l}"
      library_archive=
      for library_dir in \$library_dirs; do
        if [ -f "\$library_dir/lib\$library_name.a" ]; then
          library_archive="\$library_dir/lib\$library_name.a"
          break
        fi
      done
      if [ -z "\$library_archive" ]; then
        echo "ERROR: relocated generator linker could not resolve \$link_argument" >&2
        exit 1
      fi
      link_arguments="\$link_arguments \$library_archive"
      ;;
    *) link_arguments="\$link_arguments \$link_argument" ;;
  esac
done
if [ "\$expect_library_dir" = true ]; then
  echo 'ERROR: relocated generator linker received -L without a directory' >&2
  exit 1
fi
exec "\$linker" \$link_arguments
EOF
      $BB chmod 755 "$out/gcc/bin/as" "$out/gcc/bin/ld"

      $BB mkdir -p "$out/share/automake"
      $BB cp "$AUTOMAKE115_SRC/lib/config.sub" "$out/share/automake/config.sub"
      $BB chmod 755 "$out/share/automake/config.sub"

      test -x "$out/gcc/bin/gcc"
      test -s "$out/musl/lib/libc.a"
      test -x "$out/binutils/bin/ld"
      test -x "$out/make/bin/make"
      $BB cat > "$WORK/relocated-smoke.c" <<'EOF'
int main(void) { return 0; }
EOF
      MANTLE_GCC_ASSEMBLER="$out/binutils/bin/as" \
      MANTLE_GCC_LINKER="$out/binutils/bin/ld" \
      MANTLE_GCC_LINK_LIBC="$out/musl/lib/libc.a" \
      MANTLE_GCC_LINK_LIBM="$out/musl/lib/libm.a" \
      MANTLE_GCC_LINK_CRT="$out/musl/lib" \
        "$out/gcc/bin/gcc" \
          -B"$out/gcc/bin/" \
          -B"$out/gcc/libexec/gcc/$TARGET/$GCC_VERSION/" \
          -B"$out/gcc/lib/gcc/$TARGET/$GCC_VERSION/" \
          -I"$out/musl/include" -static \
          "$WORK/relocated-smoke.c" -o "$WORK/relocated-smoke"
      "$WORK/relocated-smoke"
      echo "GCC generator base direct-input bundle passed"
    "%
  ],
  inputs = [stage0, make, gcc, musl, binutils, stagex_transition, automake115_src],
} | crunch.Derivation
