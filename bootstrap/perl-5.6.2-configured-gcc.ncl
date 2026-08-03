# Perl 5.6.2 configured and built conventionally with the closed GCC/Make
# cycle. This succeeds the bounded miniperl diagnostic by requiring the
# Data::Dumper and xsubpp behavior consumed by Autoconf 2.53.
#
# The build regenerates and compiles the Perl parser before running the
# conventional Configure/Makefile path required by later Autoconf stages.

let crunch = import "lib.ncl" in

let perl_src = crunch.fetchTarball {
  url = "https://www.cpan.org/src/5.0/perl-5.6.2.tar.gz",
  hash = "sha256-nkx05j4lJWByRFx60ySwH40RB0dj12+VplJyVzzIFQo=",
  name = "perl-5.6.2-src",
} in

{
  name = "perl-5.6.2-configured-gcc-v16",
  builder = "/bin/sh",
  args = ["-c", m%"
    set -eu
    BB=/bin/busybox
    BUILD_JOBS=1
    VERSION=5.6.2
    RUNTIME_LEFT=20
    RUNTIME_RIGHT=22
    EXPECTED_SUM=42
    POSIX_INPUT=4.75
    POSIX_EXPECTED=4
    GLOB_EXPECTED=2

    find_input() {
      for candidate in $NIX_STORE/*-"$1"; do
        if [ -d "$candidate" ]; then echo "$candidate"; return; fi
      done
      echo "ERROR: input $1 not found" >&2
      exit 1
    }

    STAGE0=$(find_input stage0-posix)
    GCC=$(find_input gcc-4.0.4-musl-pass6-v1)
    MUSL=$(find_input musl-1.1.24-gcc-pass5-v1)
    BINUTILS=$(find_input binutils-2.30-gcc-pass4-v17)
    MAKE=$(find_input make-3.82-tcc)
    GAWK=$(find_input gawk-3.0.4-gcc-v4)
    GREP=$(find_input grep-2.4-gcc-v10)
    BISON=$(find_input bison-2.3-gcc-v9)
    M4TOOL=$(find_input m4-1.4.7-gcc-v5)
    SRC=$(find_input perl-5.6.2-src)

    record_phase() {
      phase=$1
      $BB printf '%s\n' "$phase" > "$NIX_BUILD_TOP/mantle-perl-phase"
    }

    WORK=/tmp/perl-5.6.2-configured-gcc
    TOOLBIN="$WORK/tools"
    HOME_DIR="$WORK/home"
    $BB rm -rf "$WORK"
    $BB mkdir -p "$TOOLBIN" "$HOME_DIR" "$WORK/src"
    for applet in basename cat chmod cmp comm cp cut date dirname echo env expr false find head hostname id ln ls mkdir mv od printf pwd rm rmdir sed sh sleep tail tee test touch tr true uname uniq wc xargs; do
      $BB ln -sf "$BB" "$TOOLBIN/$applet"
    done
    $BB cat > "$TOOLBIN/split" <<EOF
#!/bin/sh
case "\${1-}" in
  -[0-9]*) lines=\${1#-}; shift; exec "$BB" split -l "\$lines" "\$@" ;;
  *) exec "$BB" split "\$@" ;;
esac
EOF
    $BB chmod 755 "$TOOLBIN/split"
    # Perl 5.6.2's Configure still uses the historical `sort +POSITION`
    # spelling. Translate that bounded compatibility form to BusyBox -k.
    $BB cat > "$TOOLBIN/sort" <<EOF
#!/bin/sh
case "\${1-}:\${2-}" in
  -n:+0) shift 2; exec "$BB" sort -n -k1 "\$@" ;;
  -n:+1) shift 2; exec "$BB" sort -n -k2 "\$@" ;;
  -n:+2) shift 2; exec "$BB" sort -n -k3 "\$@" ;;
  -u:+0) shift 2; exec "$BB" sort -u -k1 "\$@" ;;
  -u:+1) shift 2; exec "$BB" sort -u -k2 "\$@" ;;
esac
case "\${1-}" in
  +0) shift; exec "$BB" sort -k1 "\$@" ;;
  +1) shift; exec "$BB" sort -k2 "\$@" ;;
  +2) shift; exec "$BB" sort -k3 "\$@" ;;
  +3) shift; exec "$BB" sort -k4 "\$@" ;;
  +4) shift; exec "$BB" sort -k5 "\$@" ;;
  +5) shift; exec "$BB" sort -k6 "\$@" ;;
  *) exec "$BB" sort "\$@" ;;
esac
EOF
    $BB chmod 755 "$TOOLBIN/sort"
    $BB ln -sf "$GAWK/bin/gawk" "$TOOLBIN/awk"
    $BB ln -sf "$GREP/bin/grep" "$TOOLBIN/grep"
    $BB ln -sf "$GREP/bin/egrep" "$TOOLBIN/egrep"
    $BB ln -sf "$GREP/bin/fgrep" "$TOOLBIN/fgrep"
    for binary_name in ar as ld nm objdump ranlib readelf strip; do
      $BB ln -sf "$BINUTILS/bin/$binary_name" "$TOOLBIN/$binary_name"
    done

    CC="$TOOLBIN/gcc"
    $BB cat > "$CC" <<EOF
#!/bin/sh
MANTLE_GCC_ASSEMBLER="$BINUTILS/bin/as" \\
MANTLE_GCC_LINKER="$BINUTILS/bin/ld" \\
MANTLE_GCC_LINK_LIBC="$MUSL/lib/libc.a" \\
MANTLE_GCC_LINK_LIBM="$MUSL/lib/libm.a" \\
MANTLE_GCC_LINK_CRT="$MUSL/lib" \\
exec "$GCC/bin/gcc" \\
  -B"$GCC/bin/" \\
  -B"$GCC/libexec/gcc/x86_64-unknown-linux-musl/4.0.4/" \\
  -B"$GCC/lib/gcc/x86_64-unknown-linux-musl/4.0.4/" \\
  -static "\$@"
EOF
    $BB chmod 755 "$CC"

    $BB cp -r "$SRC/." "$WORK/src/"
    $BB chmod -R u+w "$WORK/src"
    cd "$WORK/src"

    export PATH="$TOOLBIN:$MAKE/bin:$GCC/bin:$BINUTILS/bin:$GAWK/bin:$GREP/bin:$BISON/bin:$M4TOOL/bin:$STAGE0/bin"
    export HOME="$HOME_DIR"
    export CONFIG_SHELL="$TOOLBIN/sh"
    export M4="$M4TOOL/bin/m4"
    export BISON_PKGDATADIR="$BISON/share/bison"
    unset MAKEFLAGS MFLAGS CDPATH ENV BASH_ENV

    test -s perly.y
    $BB grep -F -q 'static void yydestruct(pTHXo_ void *ptr);' perly.y
    $BB sed -i '/^static void yydestruct(pTHXo_ void \*ptr);$/d' perly.y
    if $BB grep -F -q 'static void yydestruct(pTHXo_ void *ptr);' perly.y; then
      echo 'ERROR: obsolete Perl/byacc yydestruct declaration remained' >&2
      exit 1
    fi
    # Perl 5.6.2 already carries a pure-Bison lexer path. Use it so recursive
    # parser entry keeps state local instead of requiring the byacc-1.8-only
    # perly_c.diff patch against a different generated skeleton.
    $BB sed -i '/^%start prog$/a %pure-parser' perly.y
    $BB grep -F -q '%pure-parser' perly.y
    $BB rm -f perly.c perly.h perly.tab.c perly.tab.h
    M4="$M4TOOL/bin/m4" BISON_PKGDATADIR="$BISON/share/bison" \
      "$BISON/bin/bison" -d perly.y
    test -s perly.tab.c
    test -s perly.tab.h
    $BB mv perly.tab.c perly.c
    $BB mv perly.tab.h perly.h
    $BB touch perly.c perly.h
    record_phase parser-regenerated

    # Dependency generation uses shell constructs outside this bounded BusyBox
    # compatibility surface. Configure may skip it; the explicit Makefile build
    # below exercises and links every core and requested extension object.
    "$GAWK/bin/gawk" '
      /if \$contains .*depend:/ { in_depend_prompt = 1 }
      in_depend_prompt && $0 == "\tdflt=y" {
        print "\tdflt=n"
        in_depend_prompt = 0
        replaced += 1
        next
      }
      { print }
      END { if (replaced != 1) exit 1 }
    ' Configure > "$WORK/Configure.no-depend"
    $BB mv "$WORK/Configure.no-depend" Configure
    $BB chmod 755 Configure

    "$TOOLBIN/sh" Configure -des \
      -Dprefix="$out" \
      -Dinstallprefix="$out" \
      -Dcc="$CC" \
      -Dld="$CC" \
      -Dar="$BINUTILS/bin/ar" \
      -Dranlib="$BINUTILS/bin/ranlib" \
      -Dnm="$BINUTILS/bin/nm" \
      -Dlibc="$MUSL/lib/libc.a" \
      -Dsort="$TOOLBIN/sort" \
      -Dsplit="$TOOLBIN/split" \
      -Doptimize='-O2' \
      -Dccflags='-O2 -DUSE_PURE_BISON' \
      -Dldflags='-static' \
      -Dlibs='-lm' \
      -Uusedl \
      -Uuseshrplib \
      -Dstatic_ext='Data/Dumper Fcntl File/Glob IO POSIX' \
      -Udynamic_ext \
      -Dman1dir=none \
      -Dman3dir=none \
      -Dinstallusrbinperl=n
    record_phase configure-complete

    test -s config.sh
    $BB grep -q "^cc='$CC'" config.sh
    $BB grep -q "^ptrsize='8'" config.sh
    $BB grep -q "^ivsize='8'" config.sh

    # Errno's Linux branch hard-codes the host include root. Point the
    # generator at the selected musl sysroot instead of ambient /usr/include.
    $BB sed -i "s|/usr/include/errno.h|$MUSL/include/errno.h|" ext/Errno/Errno_pm.PL
    $BB grep -q "$MUSL/include/errno.h" ext/Errno/Errno_pm.PL

    # x2p translators are not an Autoconf dependency and retain a separate
    # historical Makefile compatibility frontier. Keep the conventional core,
    # utilities, DynaLoader, and Data::Dumper build bounded to this consumer.
    $BB grep -q '^public = .*translators' Makefile
    $BB sed -i 's/ translators$//' Makefile
    if $BB grep -q '^public = .*translators' Makefile; then
      echo 'ERROR: x2p translators remain in the bounded public target' >&2
      exit 1
    fi
    # Configure was intentionally told not to run makedepend. Materialize the
    # selected first makefile from the generated Makefile so perlmain.c does
    # not re-enter the unsupported dependency scanner.
    $BB cp Makefile makefile
    $BB touch makefile

    record_phase compile-start
    "$MAKE/bin/make" -f Makefile -j "$BUILD_JOBS" perl utilities
    record_phase compile-complete
    test -x ./perl
    (
      cd ext/Errno
      ../../perl -I../../lib Errno_pm.PL
    )
    test -s ext/Errno/Errno.pm
    # Keep installation bounded to the selected static extension set rather
    # than entering installperl's broader optional-extension traversal.
    # Install the built interpreter and generated library tree directly at
    # Configure's selected privlib path.
    $BB mkdir -p "$out/bin" "$out/lib/$VERSION" "$out/share/bootstrap-regeneration"
    $BB cp perl "$out/bin/perl"
    $BB chmod 755 "$out/bin/perl"
    $BB cp -r lib/. "$out/lib/$VERSION/"
    $BB cp ext/Errno/Errno.pm "$out/lib/$VERSION/Errno.pm"
    $BB cp perly.c perly.h "$out/share/bootstrap-regeneration/"
    test -s "$out/lib/$VERSION/Errno.pm"
    test -s "$out/share/bootstrap-regeneration/perly.c"
    test -s "$out/share/bootstrap-regeneration/perly.h"

    test -x "$out/bin/perl"
    "$out/bin/perl" -v > "$WORK/version.txt"
    $BB grep -q 'v5.6.2' "$WORK/version.txt"

    runtime_output=$("$out/bin/perl" -e "print(($RUNTIME_LEFT+$RUNTIME_RIGHT)==$EXPECTED_SUM ? \"configured-perl-ok\\n\" : \"bad\\n\")")
    test "$runtime_output" = configured-perl-ok

    dumper_output=$("$out/bin/perl" -MData::Dumper -e "my \$d = Data::Dumper->new([{answer => $EXPECTED_SUM}]); \$d->Terse(1); \$d->Indent(0); print \$d->Dump")
    test -n "$dumper_output"
    echo "$dumper_output" | $BB grep -q "$EXPECTED_SUM"
    "$out/bin/perl" -MIO::File -e 'my $f = IO::File->new("/tmp/perl-io-positive", "w") or die "open"; print $f "io-ok\n"; $f->close or die "close"'
    $BB grep -q '^io-ok$' /tmp/perl-io-positive
    posix_output=$("$out/bin/perl" -MPOSIX -e "print POSIX::floor($POSIX_INPUT), \"\\n\"")
    test "$posix_output" = "$POSIX_EXPECTED"
    $BB mkdir -p /tmp/perl-glob-positive
    $BB touch /tmp/perl-glob-positive/one /tmp/perl-glob-positive/two
    glob_count=$("$out/bin/perl" -MFile::Glob -e 'my @paths = File::Glob::bsd_glob("/tmp/perl-glob-positive/*"); print scalar(@paths), "\n"')
    test "$glob_count" = "$GLOB_EXPECTED"
    errno_value=$("$out/bin/perl" -MErrno=EINVAL -e 'print EINVAL, "\n"')
    test -n "$errno_value"

    XSUBPP="$out/lib/$VERSION/ExtUtils/xsubpp"
    TYPEMAP="$out/lib/$VERSION/ExtUtils/typemap"
    if test ! -s "$XSUBPP"; then XSUBPP="$out/lib/perl5/$VERSION/ExtUtils/xsubpp"; fi
    if test ! -s "$TYPEMAP"; then TYPEMAP="$out/lib/perl5/$VERSION/ExtUtils/typemap"; fi
    test -s "$XSUBPP"
    test -s "$TYPEMAP"
    "$out/bin/perl" "$XSUBPP" -typemap "$TYPEMAP" ext/Data/Dumper/Dumper.xs > "$WORK/Dumper.c"
    test -s "$WORK/Dumper.c"
    $BB grep -q 'boot_Data__Dumper' "$WORK/Dumper.c"

    if "$out/bin/perl" -e 'sub {' > "$WORK/rejected.out" 2> "$WORK/rejected.err"; then
      echo 'ERROR: configured Perl accepted malformed source' >&2
      exit 1
    fi
    test ! -s "$WORK/rejected.out"
    test -s "$WORK/rejected.err"

    "$BINUTILS/bin/readelf" -h "$out/bin/perl" > "$WORK/readelf.txt"
    $BB grep -q 'ELF64' "$WORK/readelf.txt"
    record_phase runtime-complete

    echo 'Perl 5.6.2 configured GCC runtime, Data::Dumper, IO::File, POSIX, File::Glob, Errno, xsubpp, and rejection checks passed; not provider admission'
  "%],
  inputs = [
    crunch.derivationFile "stage0-posix.ncl",
    crunch.derivationFile "gcc-4.0-musl-pass6.ncl",
    crunch.derivationFile "musl-1.1.24-gcc-pass5.ncl",
    crunch.derivationFile "binutils-2.30-gcc.ncl",
    crunch.derivationFile "make-tcc.ncl",
    crunch.derivationFile "gawk-3.0.4-gcc.ncl",
    crunch.derivationFile "grep-2.4-gcc.ncl",
    crunch.derivationFile "bison-2.3-gcc.ncl",
    crunch.derivationFile "m4-1.4.7-gcc.ncl",
    perl_src,
  ],
} | crunch.Derivation
