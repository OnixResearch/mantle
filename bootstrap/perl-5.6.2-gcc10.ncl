# Source-built Perl for the full-source Rust provider route.
#
# Vendored OpenSSL uses Perl during Cargo construction. This interpreter is
# built from authenticated Perl source by the admitted normalized GCC/musl/
# binutils provider and declared source-built bootstrap tools.

let crunch = import "lib.ncl" in
let native_provider = import "seed-full-admitted.ncl" in
let admitted_host_tools = import "full-source-rust-host-tools-admitted.ncl" in
# Upstream archive SHA-256: a5e66f6ebf701b0567f569f57cae82abf5ce57af70a2b45ae71323b61f49134e.
let perl_src = crunch.fetchTarball {
  url = "https://www.cpan.org/src/5.0/perl-5.6.2.tar.gz",
  hash = "sha256-nkx05j4lJWByRFx60ySwH40RB0dj12+VplJyVzzIFQo=",
  name = "perl-5.6.2-src",
} in

{
  name = "perl-5.6.2-full-source-gcc10-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      JOB_COUNT=1
      TOOL_PROBE_TIMEOUT_SECONDS=30
      VERSION=5.6.2
      RUNTIME_LEFT=20
      RUNTIME_RIGHT=22
      EXPECTED_SUM=42
      WORK=/tmp/perl-5.6.2-full-source
      TARGET=x86_64-linux-musl

      find_input() {
        input_suffix=$1
        matched_input=
        match_count=0
        for input_path in $NIX_STORE/*-"$input_suffix"; do
          if [ -e "$input_path" ]; then
            matched_input=$input_path
            match_count=$((match_count + 1))
          fi
        done
        if [ "$match_count" -ne 1 ]; then
          echo "ERROR: expected exactly one declared input $input_suffix, got $match_count" >&2
          exit 1
        fi
        echo "$matched_input"
      }

      PROVIDER=$(find_input full-source-seed-toolchain)
      MAKE=$(find_input make-4.4.1-full-source-gcc10-v1)
      LINUX_HEADERS=$(find_input linux-headers-6.6-full-source-gcc10-v3)
      BUSYBOX=$(find_input busybox-1.37.0-full-source-gcc10-v1)
      SRC=$(find_input perl-5.6.2-src)
      SOURCE_ROOT="$SRC"
      if [ -f "$SRC/perl-5.6.2/Configure" ]; then
        SOURCE_ROOT="$SRC/perl-5.6.2"
      fi

      for required_executable in \
        "$PROVIDER/bin/$TARGET-gcc" \
        "$PROVIDER/bin/ar" \
        "$PROVIDER/bin/nm" \
        "$PROVIDER/bin/ranlib" \
        "$MAKE/bin/make" \
        "$BUSYBOX/bin/busybox"; do
        if [ ! -x "$required_executable" ]; then
          echo "ERROR: required Perl construction executable is missing: $required_executable" >&2
          exit 1
        fi
      done
      if [ ! -f "$LINUX_HEADERS/include/linux/fs.h" ]; then
        echo "ERROR: declared Linux headers lack include/linux/fs.h: $LINUX_HEADERS" >&2
        exit 1
      fi
      if [ ! -x "$SOURCE_ROOT/Configure" ]; then
        echo "ERROR: authenticated Perl source has no Configure script at either bounded root: $SRC" >&2
        exit 1
      fi

      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK/tools" "$WORK/home" "$WORK/src"
      for applet in basename cat chmod cmp comm cp cut date dirname echo env expr false find head hostname id install ln ls mkdir mv od paste printf pwd readlink realpath rm rmdir sed sh sleep tail tee test touch tr true uname uniq wc xargs; do
        $BB ln -sf "$BUSYBOX/bin/busybox" "$WORK/tools/$applet"
      done
      $BB cat > "$WORK/tools/split" <<EOF
#!/bin/sh
case "\${1-}" in
  -[0-9]*) lines=\${1#-}; shift; exec "$BUSYBOX/bin/busybox" split -l "\$lines" "\$@" ;;
  *) exec "$BUSYBOX/bin/busybox" split "\$@" ;;
esac
EOF
      $BB cat > "$WORK/tools/sort" <<EOF
#!/bin/sh
case "\${1-}:\${2-}" in
  -n:+0) shift 2; exec "$BUSYBOX/bin/busybox" sort -n -k1 "\$@" ;;
  -n:+1) shift 2; exec "$BUSYBOX/bin/busybox" sort -n -k2 "\$@" ;;
  -n:+2) shift 2; exec "$BUSYBOX/bin/busybox" sort -n -k3 "\$@" ;;
  -u:+0) shift 2; exec "$BUSYBOX/bin/busybox" sort -u -k1 "\$@" ;;
  -u:+1) shift 2; exec "$BUSYBOX/bin/busybox" sort -u -k2 "\$@" ;;
esac
case "\${1-}" in
  +0) shift; exec "$BUSYBOX/bin/busybox" sort -k1 "\$@" ;;
  +1) shift; exec "$BUSYBOX/bin/busybox" sort -k2 "\$@" ;;
  +2) shift; exec "$BUSYBOX/bin/busybox" sort -k3 "\$@" ;;
  +3) shift; exec "$BUSYBOX/bin/busybox" sort -k4 "\$@" ;;
  +4) shift; exec "$BUSYBOX/bin/busybox" sort -k5 "\$@" ;;
  +5) shift; exec "$BUSYBOX/bin/busybox" sort -k6 "\$@" ;;
  *) exec "$BUSYBOX/bin/busybox" sort "\$@" ;;
esac
EOF
      $BB chmod 755 "$WORK/tools/split" "$WORK/tools/sort"
      $BB ln -sf "$BUSYBOX/bin/awk" "$WORK/tools/awk"
      $BB ln -sf "$BUSYBOX/bin/grep" "$WORK/tools/grep"
      $BB ln -sf "$BUSYBOX/bin/grep" "$WORK/tools/egrep"
      $BB ln -sf "$BUSYBOX/bin/grep" "$WORK/tools/fgrep"
      $BB ln -sf "$MAKE/bin/make" "$WORK/tools/make"
      $BB ln -sf "$PROVIDER/bin/$TARGET-gcc" "$WORK/tools/cc"
      $BB ln -sf "$PROVIDER/bin/ar" "$WORK/tools/ar"
      $BB ln -sf "$PROVIDER/bin/nm" "$WORK/tools/nm"
      $BB ln -sf "$PROVIDER/bin/ranlib" "$WORK/tools/ranlib"
      $BB ln -sf "$PROVIDER/bin/ld" "$WORK/tools/ld"

      export PATH="$WORK/tools"
      export HOME="$WORK/home"
      export CONFIG_SHELL="$BUSYBOX/bin/sh"
      export SHELL="$BUSYBOX/bin/sh"
      export MAKE="$MAKE/bin/make"
      export CC="$PROVIDER/bin/$TARGET-gcc"
      export AR="$PROVIDER/bin/ar"
      export NM="$PROVIDER/bin/nm"
      export RANLIB="$PROVIDER/bin/ranlib"
      export LD="$PROVIDER/bin/ld"
      export SOURCE_DATE_EPOCH=1
      unset MAKEFLAGS MFLAGS CDPATH ENV BASH_ENV

      $BB cp -R "$SOURCE_ROOT/." "$WORK/src/"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"

      "$BUSYBOX/bin/awk" '
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

      "$BUSYBOX/bin/sh" Configure -des \
        -Dprefix="$out" \
        -Dinstallprefix="$out" \
        -Dcc="$CC" \
        -Dld="$CC" \
        -Dar="$AR" \
        -Dranlib="$RANLIB" \
        -Dnm="$NM" \
        -Dlibc="$PROVIDER/$TARGET/lib/libc.a" \
        -Dsort="$WORK/tools/sort" \
        -Dsplit="$WORK/tools/split" \
        -Doptimize='-O2' \
        -Dccflags="-O2 -I$LINUX_HEADERS/include" \
        -Dldflags='-static' \
        -Dlibs='-lm' \
        -Uusedl \
        -Uuseshrplib \
        -Dstatic_ext='Data/Dumper Fcntl File/Glob IO POSIX' \
        -Udynamic_ext \
        -Dman1dir=none \
        -Dman3dir=none \
        -Dinstallusrbinperl=n

      test -s config.sh
      $BB grep -q "^cc='$CC'" config.sh
      $BB grep -q "^ptrsize='8'" config.sh
      $BB grep -q "^ivsize='8'" config.sh

      $BB sed -i "s|/usr/include/errno.h|$PROVIDER/$TARGET/include/errno.h|" ext/Errno/Errno_pm.PL
      $BB grep -q "$PROVIDER/$TARGET/include/errno.h" ext/Errno/Errno_pm.PL
      $BB grep -q '^public = .*translators' Makefile
      $BB sed -i 's/ translators$//' Makefile
      if $BB grep -q '^public = .*translators' Makefile; then
        echo "ERROR: x2p translators remain in the bounded public target" >&2
        exit 1
      fi
      $BB cp Makefile makefile
      $BB touch makefile

      "$MAKE" -f Makefile -j "$JOB_COUNT" perl utilities
      test -x ./perl
      (
        cd ext/Errno
        ../../perl -I../../lib Errno_pm.PL
      )
      test -s ext/Errno/Errno.pm

      $BB mkdir -p "$out/bin" "$out/lib/$VERSION"
      $BB cp perl "$out/bin/perl"
      $BB chmod 755 "$out/bin/perl"
      $BB cp -R lib/. "$out/lib/$VERSION/"
      $BB cp ext/Errno/Errno.pm "$out/lib/$VERSION/Errno.pm"

      "$out/bin/perl" -v | "$BUSYBOX/bin/grep" -q 'v5.6.2'
      runtime_output=$("$out/bin/perl" -e "print(($RUNTIME_LEFT+$RUNTIME_RIGHT)==$EXPECTED_SUM ? \"full-source-perl-ok\\n\" : \"bad\\n\")")
      test "$runtime_output" = full-source-perl-ok
      "$out/bin/perl" -MConfig -MFile::Basename -MFile::Spec::Functions -MFindBin -MGetopt::Long -e 'print "openssl-configure-modules-ok\n"' \
        | "$BUSYBOX/bin/grep" -q openssl-configure-modules-ok

      if "$BUSYBOX/bin/timeout" "$TOOL_PROBE_TIMEOUT_SECONDS" "$out/bin/perl" -e 'sub {' > "$WORK/malformed.stdout" 2> "$WORK/malformed.stderr"; then
        echo "ERROR: source-built Perl accepted malformed input" >&2
        exit 1
      fi
      test ! -s "$WORK/malformed.stdout"
      test -s "$WORK/malformed.stderr"

      RELOCATED_PERL="$WORK/relocated-perl"
      $BB cp -R "$out" "$RELOCATED_PERL"
      PERL5LIB="$RELOCATED_PERL/lib/$VERSION" "$RELOCATED_PERL/bin/perl" -MConfig -e 'print "mantle-source-perl-relocated-ok\n"' \
        | "$BUSYBOX/bin/grep" -q mantle-source-perl-relocated-ok

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/full-source-rust-host-perl.txt" <<'EOF'
claim: bounded source-built Perl host-tool matrix passed
positive: version, OpenSSL Configure core-module imports, copied-tree execution
negative: malformed Perl source rejected with nonempty stderr
source_authority: authenticated Perl 5.6.2 source
native_authority: admitted full-source GCC/musl/binutils provider
non_claims: Perl correctness, OpenSSL correctness, Rust provider completion, whole-bootstrap correctness
EOF
      echo "source-built Perl 5.6.2 host-tool matrix passed"
    "%
  ],
  inputs = [
    native_provider,
    admitted_host_tools.make,
    admitted_host_tools.linux_headers,
    admitted_host_tools.busybox,
    perl_src,
  ],
} | crunch.Derivation
