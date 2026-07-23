# Autoconf 2.52 rebuilt with the GCC-built Perl/M4/Gawk/Grep generator set.
# Component frontier only: final provider admission must bind the authenticated
# derivational closure; component runtime evidence is not provider admission.

let crunch = import "lib.ncl" in

let autoconf_src = crunch.fetchTarball {
  url = "https://mirrors.kernel.org/gnu/autoconf/autoconf-2.52.tar.bz2",
  hash = "sha256-vCx4E2/YyJCkwgInpyyrGhJTYDyl70WVSkVHLeYr7Fg=",
  name = "autoconf-2.52-src",
} in

{
  name = "autoconf-2.52-gcc-v6",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    VERSION=2.52

    find_input() {
      for candidate in $NIX_STORE/*-"$1"; do
        if [ -d "$candidate" ]; then echo "$candidate"; return; fi
      done
      echo "ERROR: input $1 not found" >&2
      exit 1
    }

    STAGE0=$(find_input stage0-posix)
    PERL=$(find_input perl-5.6.2-configured-gcc-v16)
    M4TOOL=$(find_input m4-1.4.7-gcc-v5)
    GAWK=$(find_input gawk-3.0.4-gcc-v4)
    GREP=$(find_input grep-2.4-gcc-v10)
    SRC=$(find_input autoconf-2.52-src)

    WORK=/tmp/autoconf-2.52-gcc
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

    export PATH="$TOOLBIN:$PERL/bin:$M4TOOL/bin:$GAWK/bin:$GREP/bin:$STAGE0/bin"
    export CONFIG_SHELL="$TOOLBIN/sh"
    export PERL="$PERL/bin/perl"
    export M4="$M4TOOL/bin/m4"
    export AWK="$GAWK/bin/gawk"
    export GREP="$GREP/bin/grep"

    ./configure --prefix="$out"

    $BB mkdir -p "$out/bin" "$out/share/autoconf"
    echo 'autoconf252: generate-commands'
    for command_name in autoconf autoheader autoreconf autoscan autoupdate ifnames; do
      generated_path="$out/bin/$command_name"
      "$TOOLBIN/sed" \
        -e "s|@SHELL@|$TOOLBIN/sh|g" \
        -e "s|@PERL@|$PERL/bin/perl|g" \
        -e "s|@datadir@|$out/share/autoconf|g" \
        -e "s|@bindir@|$out/bin|g" \
        -e 's|@autoconf-name@|autoconf|g' \
        -e 's|@autoheader-name@|autoheader|g' \
        -e "s|@M4@|$M4TOOL/bin/m4|g" \
        -e "s|@AWK@|$GAWK/bin/gawk|g" \
        -e "s|@VERSION@|$VERSION|g" \
        -e 's|@PACKAGE_NAME@|Autoconf|g' \
        "$command_name.in" > "$generated_path"
      test -s "$generated_path" || {
        echo "ERROR: generated command missing: $generated_path" >&2
        exit 1
      }
      $BB chmod 755 "$generated_path"
    done

    echo 'autoconf252: copy-data'
    for data_name in acfunctions acheaders acidentifiers acmakevars acprograms aclibraries m4sugar.m4 m4sh.m4 acversion.m4 autoconf.m4 acgeneral.m4 acoldnames.m4 acspecific.m4 aclang.m4 acfunctions.m4 acheaders.m4 actypes.m4; do
      test -s "$data_name" || { echo "ERROR: data file missing: $data_name" >&2; exit 1; }
      $BB cp "$data_name" "$out/share/autoconf/$data_name"
    done
    echo 'autoconf252: freeze'
    "$M4TOOL/bin/m4" --include . --fatal-warning \
      --freeze-state="$out/share/autoconf/autoconf.m4f" autoconf.m4 \
      > "$WORK/freeze.log"
    test -s "$out/share/autoconf/autoconf.m4f"
    if $BB grep -v '^$' "$WORK/freeze.log" > "$WORK/freeze-nonempty.log"; then
      echo 'ERROR: freezing autoconf.m4 produced unexpected output' >&2
      exit 1
    fi

    echo 'autoconf252: runtime-version'
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
      echo 'ERROR: autoconf accepted malformed input' >&2
      exit 1
    fi
    test ! -s configure
    test -s "$WORK/negative.err"

    echo 'Autoconf 2.52 GCC generator and rejection checks passed; not provider admission'
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "perl-5.6.2-configured-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    autoconf_src,
  ],
} | crunch.Derivation
