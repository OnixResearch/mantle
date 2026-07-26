# Binutils 2.41 source with bounded Autoconf regeneration and generated
# parser/lexer removal. The compiler derivation must regenerate and compile the
# removed Bison/Flex artifacts before this source can support provider admission.

let crunch = import "lib.ncl" in

let binutils_src = crunch.fetchTarball {
  # provenance: fosslinux/live-bootstrap steps/binutils-2.41/sources
  url = "https://mirrors.kernel.org/gnu/binutils/binutils-2.41.tar.xz",
  hash = "sha256-8dGjwSZTk28XKXwkODNezqFVLXDA6Ycf/dLF+RoYGWo=",
  name = "binutils-2.41-src",
} in

{
  name = "binutils-2.41-regeneration-ready-source-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      WORK=/tmp/binutils-2.41-source-regeneration
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

      SRC=$(find_input binutils-2.41-src)
      AUTOCONF=$(find_input autoconf-2.69-gcc-v1)
      PERL=$(find_input perl-5.6.2-configured-gcc-v16)
      M4TOOL=$(find_input m4-1.4.7-gcc-v5)
      GREP=$(find_input grep-2.4-gcc-v10)

      $BB rm -rf "$WORK"
      $BB mkdir -p "$TOOLBIN"
      for applet in basename cat chmod cmp cp dirname echo env expr false find grep head ln ls mkdir mv printf pwd readlink rm sed sh sort tail test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BB" "$TOOLBIN/$applet"
      done
      $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
      $BB ln -sf "$GREP/bin/egrep" "$TOOLBIN/egrep"
      $BB ln -sf "$GREP/bin/fgrep" "$TOOLBIN/fgrep"
      export PATH="$AUTOCONF/bin:$PERL/bin:$M4TOOL/bin:$TOOLBIN"
      export PERL="$PERL/bin/perl"
      export M4="$M4TOOL/bin/m4"
      export CONFIG_SHELL=/bin/sh
      export SHELL=/bin/sh

      $BB cp -R "$SRC" "$out"
      $BB chmod -R u+w "$out"
      cd "$out"

      $BB mkdir -p .mantle-regeneration
      : > .mantle-regeneration/binutils-2.41-configure-files.txt
      for configure_dir in . bfd binutils gas gprof intl ld libctf libiberty libsframe opcodes; do
        test -s "$configure_dir/configure.ac"
        (cd "$configure_dir" && "$AUTOCONF/bin/autoconf" --force)
        test -s "$configure_dir/configure"
        if "$GREP/bin/grep" -q 'AC_CONFIG_HEADERS' "$configure_dir/configure.ac"; then
          (cd "$configure_dir" && "$AUTOCONF/bin/autoheader" --force)
        fi
        if test "$configure_dir" = .; then
          $BB printf '%s\n' configure >> .mantle-regeneration/binutils-2.41-configure-files.txt
        else
          $BB printf '%s/configure\n' "$configure_dir" >> .mantle-regeneration/binutils-2.41-configure-files.txt
        fi
      done

      $BB cat > .mantle-regeneration/binutils-2.41-removed-generated-files.txt <<'EOF'
binutils/arparse.c
binutils/arparse.h
binutils/defparse.c
binutils/defparse.h
binutils/mcparse.c
binutils/mcparse.h
binutils/rcparse.c
binutils/rcparse.h
binutils/sysinfo.c
binutils/sysinfo.h
gas/config/bfin-parse.c
gas/config/bfin-parse.h
gas/config/loongarch-parse.c
gas/config/loongarch-parse.h
gas/config/m68k-parse.c
gas/config/rl78-parse.c
gas/config/rl78-parse.h
gas/config/rx-parse.c
gas/config/rx-parse.h
gas/itbl-parse.c
gas/itbl-parse.h
gold/yyscript.c
gold/yyscript.h
intl/plural.c
ld/deffilep.c
ld/deffilep.h
ld/ldgram.c
ld/ldgram.h
binutils/arlex.c
binutils/deflex.c
binutils/syslex.c
gas/config/bfin-lex.c
gas/config/loongarch-lex.c
gas/itbl-lex.c
ld/ldlex.c
EOF
      while IFS= read -r generated_file; do
        test -n "$generated_file"
        $BB rm -f "$generated_file"
        test ! -e "$generated_file"
      done < .mantle-regeneration/binutils-2.41-removed-generated-files.txt
      for grammar_file in $($BB find . -type f '(' -name '*.y' -o -name '*.l' ')'); do
        $BB touch "$grammar_file"
      done

      $BB printf '%s\n' \
        'source=binutils-2.41 authenticated release archive' \
        'autoconf=2.69 source-built derivation' \
        'parser_generator=consumer-bound bison-3.4.1' \
        'scanner_generator=consumer-bound flex-2.6.4' \
        'status=regeneration-ready-generated-files-removed' \
        > .mantle-regeneration/provenance.txt
      test -s .mantle-regeneration/binutils-2.41-configure-files.txt
      test -s .mantle-regeneration/binutils-2.41-removed-generated-files.txt
      echo 'Binutils 2.41 bounded Autoconf regeneration and parser/lexer source preparation passed'
    "%
  ],
  inputs = [
    binutils_src,
    crunch.derivationFile "autoconf-2.69-gcc.ncl",
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
  ],
} | crunch.Derivation
