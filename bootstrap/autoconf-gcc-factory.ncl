# Shared Autoconf release builder over the conventional GCC-built generator
# set. Consumers supply only authenticated source identity and the immediately
# preceding Autoconf output.
#
# Tool edges are lazy derivation-file references so this linear generator
# chain remains derivational without embedding recursive Nickel values.

fun spec =>
let crunch = import "lib.ncl" in
let autoconf_src = crunch.fetchTarball {
  url = spec.source_url,
  hash = spec.source_hash,
  name = spec.source_name,
} in

{
  name = spec.output_name,
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    BUILD_JOBS=1
    VERSION=%{spec.version}

    find_input() {
      for candidate in $NIX_STORE/*-"$1"; do
        if [ -d "$candidate" ]; then echo "$candidate"; return; fi
      done
      echo "ERROR: input $1 not found" >&2
      exit 1
    }

    STAGE0=$(find_input stage0-posix)
    MAKE=$(find_input make-3.82-tcc)
    PREVIOUS=$(find_input %{spec.previous_input_name})
    PERL=$(find_input perl-5.6.2-configured-gcc-v16)
    M4TOOL=$(find_input m4-1.4.7-gcc-v5)
    GAWK=$(find_input gawk-3.0.4-gcc-v4)
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input %{spec.source_name})

    WORK=/tmp/autoconf-$VERSION-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN" "$WORK/src"
    for applet in basename cat chmod cmp cp cut dirname echo expr false find head ln ls mkdir mv printf pwd rm rmdir sed sh sleep sort tail test touch tr true uname uniq wc; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
    $BB ln -sf "$GAWK/bin/gawk" "$TOOLBIN/awk"
    $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
    $BB ln -sf "$GREP/bin/egrep" "$TOOLBIN/egrep"
    $BB ln -sf "$GREP/bin/fgrep" "$TOOLBIN/fgrep"
    $BB cp -r "$SRC/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"
    for bootstrap_script in ./configure ./install-sh ./mkinstalldirs ./config/install-sh ./config/mkinstalldirs; do
      if test -f "$bootstrap_script"; then
        $BB chmod 755 "$bootstrap_script"
      fi
    done

    export PATH="$PREVIOUS/bin:$MAKE/bin:$PERL/bin:$M4TOOL/bin:$GAWK/bin:$GREP/bin:$TOOLBIN:$STAGE0/bin"
    export CONFIG_SHELL="$TOOLBIN/sh"
    export PERL="$PERL/bin/perl"
    export M4="$M4TOOL/bin/m4"
    export AWK="$GAWK/bin/gawk"
    export GREP="$GREP/bin/grep"

    ./configure --prefix="$out"
    "$MAKE/bin/make" -j "$BUILD_JOBS"
    "$MAKE/bin/make" -j "$BUILD_JOBS" install prefix="$out"

    for command_name in autoconf autoheader autom4te autoreconf autoscan autoupdate ifnames; do
      test -s "$out/bin/$command_name" || {
        echo "ERROR: installed command missing: $command_name" >&2
        exit 1
      }
      $BB chmod 755 "$out/bin/$command_name"
    done
    test -d "$out/share/autoconf"
    test -s "$out/share/autoconf/autom4te.cfg"
    export PATH="$out/bin:$PATH"

    "$out/bin/autoconf" --version > "$WORK/version.txt"
    $BB cat "$WORK/version.txt"
    $BB grep -q "$VERSION" "$WORK/version.txt"

    POSITIVE="$WORK/positive"
    $BB mkdir -p "$POSITIVE"
    $BB cat > "$POSITIVE/configure.ac" <<'EOF'
AC_INIT([mantle-autoconf-positive], [1.0])
AC_CONFIG_SRCDIR([configure.ac])
AC_OUTPUT
EOF
    cd "$POSITIVE"
    "$out/bin/autoconf"
    test -s configure
    $BB cp configure "$WORK/configure-first"
    $BB rm configure
    "$out/bin/autoconf"
    test -s configure
    $BB cmp "$WORK/configure-first" configure
    $BB chmod 755 configure
    ./configure > "$WORK/positive-configure.txt"
    $BB grep -q 'creating ./config.status' "$WORK/positive-configure.txt"

    NEGATIVE="$WORK/negative"
    $BB mkdir -p "$NEGATIVE"
    $BB cat > "$NEGATIVE/configure.ac" <<'EOF'
AC_INIT([mantle-autoconf-negative], [1.0])
m4_fatal([mantle-negative])
AC_OUTPUT
EOF
    cd "$NEGATIVE"
    if "$out/bin/autoconf" > "$WORK/negative.out" 2> "$WORK/negative.err"; then
      echo 'ERROR: autoconf accepted an explicit fatal input' >&2
      exit 1
    fi
    test ! -s configure
    test -s "$WORK/negative.err"

    echo "Autoconf $VERSION GCC generator and rejection checks passed; not provider admission"
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile spec.previous_derivation_file,
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    autoconf_src,
  ],
} | crunch.Derivation
