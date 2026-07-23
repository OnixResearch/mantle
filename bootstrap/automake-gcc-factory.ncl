# Shared Automake release builder over the conventional GCC-built generator
# set. Each consumer binds an authenticated release and a tested Autoconf
# predecessor through a lazy derivation-file edge.

fun spec =>
let crunch = import "lib.ncl" in
let automake_src = crunch.fetchTarball {
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
    AUTOCONF=$(find_input %{spec.autoconf_input_name})
    PERL=$(find_input perl-5.6.2-configured-gcc-v16)
    M4TOOL=$(find_input m4-1.4.7-gcc-v5)
    GAWK=$(find_input gawk-3.0.4-gcc-v4)
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input %{spec.source_name})

    WORK=/tmp/automake-$VERSION-gcc
    TOOLBIN="$WORK/tools"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN" "$WORK/src"
    for applet in basename cat chmod cmp cp cut dirname echo expr false find head install ln ls mkdir mv printf pwd rm rmdir sed sh sleep sort tail test touch tr true uname uniq wc; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
    $BB ln -sf "$GAWK/bin/gawk" "$TOOLBIN/awk"
    $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
    $BB ln -sf "$GREP/bin/egrep" "$TOOLBIN/egrep"
    $BB ln -sf "$GREP/bin/fgrep" "$TOOLBIN/fgrep"
    for autoconf_command in autoconf autoheader autom4te autoreconf autoscan autoupdate ifnames; do
      if test -f "$AUTOCONF/bin/$autoconf_command"; then
        first_line=$($BB head -n 1 "$AUTOCONF/bin/$autoconf_command")
        case "$first_line" in
          *perl*) command_interpreter="$PERL/bin/perl" ;;
          *) command_interpreter=/bin/sh ;;
        esac
        $BB cat > "$TOOLBIN/$autoconf_command" <<EOF
#!/bin/sh
exec "$command_interpreter" "$AUTOCONF/bin/$autoconf_command" "\$@"
EOF
        $BB chmod 755 "$TOOLBIN/$autoconf_command"
      fi
    done
    $BB cp -r "$SRC/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"
    for bootstrap_script in ./configure ./install-sh ./mkinstalldirs ./lib/install-sh ./lib/mkinstalldirs; do
      if test -f "$bootstrap_script"; then
        $BB chmod 755 "$bootstrap_script"
      fi
    done

    export PATH="$TOOLBIN:$AUTOCONF/bin:$MAKE/bin:$PERL/bin:$M4TOOL/bin:$GAWK/bin:$GREP/bin:$STAGE0/bin"
    export CONFIG_SHELL="$TOOLBIN/sh"
    export PERL="$PERL/bin/perl"
    export M4="$M4TOOL/bin/m4"
    export AWK="$GAWK/bin/gawk"
    export GREP="$GREP/bin/grep"

    ./configure --prefix="$out"
    "$MAKE/bin/make" -j "$BUILD_JOBS"
    "$MAKE/bin/make" -j "$BUILD_JOBS" install prefix="$out"

    test -s "$out/bin/automake"
    test -s "$out/bin/aclocal"
    for installed_command in "$out"/bin/*; do
      test -f "$installed_command"
      $BB chmod 755 "$installed_command"
    done
    export PATH="$out/bin:$PATH"

    "$out/bin/automake" --version > "$WORK/version.txt"
    $BB cat "$WORK/version.txt"
    $BB grep -q "$VERSION" "$WORK/version.txt"

    POSITIVE="$WORK/positive"
    $BB mkdir -p "$POSITIVE"
    $BB cat > "$POSITIVE/configure.ac" <<'EOF'
AC_INIT([mantle-automake-positive], [1.0])
AM_INIT_AUTOMAKE([foreign])
AC_CONFIG_FILES([Makefile])
AC_OUTPUT
EOF
    $BB cat > "$POSITIVE/Makefile.am" <<'EOF'
EXTRA_DIST = marker.txt
EOF
    $BB printf '%s\n' 'marker' > "$POSITIVE/marker.txt"
    cd "$POSITIVE"
    "$out/bin/aclocal"
    "$TOOLBIN/autoconf"
    "$out/bin/automake" --add-missing --copy
    test -s Makefile.in
    $BB cp Makefile.in "$WORK/Makefile.in-first"
    $BB rm Makefile.in
    "$out/bin/automake" --add-missing --copy
    test -s Makefile.in
    $BB cmp "$WORK/Makefile.in-first" Makefile.in
    $BB chmod 755 configure
    ./configure > "$WORK/positive-configure.txt"
    test -s Makefile
    "$MAKE/bin/make" -j "$BUILD_JOBS"
    $BB grep -q 'creating ./config.status' "$WORK/positive-configure.txt"

    NEGATIVE="$WORK/negative"
    $BB mkdir -p "$NEGATIVE"
    $BB cat > "$NEGATIVE/configure.ac" <<'EOF'
AC_INIT([mantle-automake-negative], [1.0])
AM_INIT_AUTOMAKE([foreign])
AC_CONFIG_FILES([Makefile])
AC_OUTPUT
EOF
    $BB cat > "$NEGATIVE/Makefile.am" <<'EOF'
if MISSING_CONDITIONAL
EXTRA_DIST = unreachable.txt
endif
EOF
    cd "$NEGATIVE"
    "$out/bin/aclocal"
    "$TOOLBIN/autoconf"
    if "$out/bin/automake" --add-missing --copy > "$WORK/negative.out" 2> "$WORK/negative.err"; then
      echo 'ERROR: automake accepted an undefined conditional' >&2
      exit 1
    fi
    test -s "$WORK/negative.err"
    $BB grep -q 'MISSING_CONDITIONAL' "$WORK/negative.err"

    echo "Automake $VERSION GCC generation and rejection checks passed; not provider admission"
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile spec.autoconf_derivation_file,
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    automake_src,
  ],
} | crunch.Derivation
