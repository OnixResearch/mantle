# Locally regenerated GCC 10.5.0 source tree.
#
# This source derivation keeps generated-source provenance inspectable and lets
# both GCC compiler stages consume the same regenerated artifact.

let crunch = import "lib.ncl" in

let gcc10_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/gcc-10.5.0/sources
  url = "https://ftpmirror.gnu.org/gcc/gcc-10.5.0/gcc-10.5.0.tar.xz",
  hash = "sha256-t4vO308XvqCAgwrl2jS1llkIvHHfSV4wlkdLMkYfrkU=",
  name = "gcc-10.5.0-src",
} in

{
  name = "gcc-10.5.0-regenerated-source-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      WORK=/tmp/gcc-10.5.0-regeneration
      TOOLBIN="$WORK/tools"

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

      SRC=$(find_input gcc-10.5.0-src)
      AUTOCONF=$(find_input autoconf-2.69-gcc-v1)
      PERL=$(find_input perl-5.6.2-configured-gcc-v16)
      M4TOOL=$(find_input m4-1.4.7-gcc-v5)
      BISON=$(find_input bison-3.4.1-gcc-v2)
      FLEX=$(find_input flex-2.6.4-gcc-v1)
      GREP=$(find_input grep-2.4-gcc-v10)

      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in basename cat chmod cmp cp dirname echo env expr false find grep head ln ls mkdir mv printf pwd readlink rm sed sh sort tail test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
      export PATH="$AUTOCONF/bin:$PERL/bin:$M4TOOL/bin:$BISON/bin:$FLEX/bin:$TOOLBIN"
      export PERL="$PERL/bin/perl"
      export M4="$M4TOOL/bin/m4"
      export BISON_PKGDATADIR="$BISON/share/bison"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh

      $BB cp -R "$SRC" "$out"
      $BB chmod -R u+w "$out"
      cd "$out"
      for configure_dir in . fixincludes intl gcc libcpp libdecnumber libgcc libiberty libstdc++-v3 lto-plugin zlib; do
        test -s "$configure_dir/configure.ac"
        (cd "$configure_dir" && "$AUTOCONF/bin/autoconf" --force)
        test -s "$configure_dir/configure"
        if "$GREP/bin/grep" -q 'AC_CONFIG_HEADERS' "$configure_dir/configure.ac"; then
          (cd "$configure_dir" && "$AUTOCONF/bin/autoheader" --force)
        fi
      done
      M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$BISON/share/bison" \
        "$BISON/bin/bison" -y -o intl/plural.c intl/plural.y
      test -s intl/plural.c
      (cd gcc && "$FLEX/bin/flex" -o gengtype-lex.generated.c gengtype-lex.l)
      $BB cat > gcc/gengtype-lex.c <<'EOF'
#ifdef HOST_GENERATOR_FILE
#include "config.h"
#else
#include "bconfig.h"
#endif
EOF
      $BB cat gcc/gengtype-lex.generated.c >> gcc/gengtype-lex.c
      $BB rm gcc/gengtype-lex.generated.c
      test -s gcc/gengtype-lex.c

      $BB mkdir -p .mantle-regeneration
      $BB printf '%s\n' \
        configure fixincludes/configure intl/configure intl/plural.c gcc/configure gcc/config.in \
        gcc/gengtype-lex.c libcpp/configure libdecnumber/configure libgcc/configure \
        libiberty/configure libstdc++-v3/configure lto-plugin/configure zlib/configure \
        > .mantle-regeneration/gcc-10.5.0-files.txt
      for regenerated_file in $($BB cat .mantle-regeneration/gcc-10.5.0-files.txt); do
        test -s "$regenerated_file"
      done
      $BB printf '%s\n' \
        'source=gcc-10.5.0 authenticated release archive' \
        'autoconf=2.69 source-built derivation' \
        'bison=3.4.1 source-built derivation' \
        'flex=2.6.4 source-built derivation' \
        'status=bounded-regeneration-complete' \
        > .mantle-regeneration/provenance.txt
      echo "GCC 10.5.0 bounded source regeneration passed"
    "%
  ],
  inputs = [
    gcc10_src,
    crunch.derivationFile "autoconf-2.69-gcc.ncl",
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "bison-3.4.1-gcc.ncl",
    crunch.derivationFile "flex-2.6.4-gcc.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
  ],
} | crunch.Derivation
