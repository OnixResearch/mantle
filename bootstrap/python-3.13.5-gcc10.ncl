# Source-built Python for the full-source Rust provider route.
#
# Rust's x.py bootstrap driver requires Python. This host interpreter is built
# from authenticated CPython source by the admitted normalized GCC/musl/
# binutils provider, source-built make, Linux UAPI headers, and source-built
# BusyBox utilities.

let crunch = import "lib.ncl" in
let native_provider = crunch.derivationFile "seed-full-toolchain.ncl" in
let gnumake = crunch.derivationFile "make-4.4.1-gcc10.ncl" in
let linux_headers = crunch.derivationFile "linux-headers-6.6-gcc10.ncl" in
let busybox = crunch.derivationFile "busybox-1.37.0-gcc10.ncl" in
# Upstream archive SHA-256: 93e583f243454e6e9e4588ca2c2662206ad961659863277afcdb96801647d640.
let python_src = crunch.fetchTarball {
  url = "https://www.python.org/ftp/python/3.13.5/Python-3.13.5.tar.xz",
  hash = "sha256-CViHmIaqSfZ4TNLtDEGqlWrq1rLQgKwfZiRgh7ohehU=",
  name = "python-3.13.5-src",
} in

{
  name = "python-3.13.5-full-source-gcc10-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      JOB_COUNT=1
      TOOL_PROBE_TIMEOUT_SECONDS=30
      WORK=/tmp/python-3.13.5-full-source
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
      LINUX_HEADERS=$(find_input linux-headers-6.6-full-source-gcc10-v1)
      BUSYBOX=$(find_input busybox-1.37.0-full-source-gcc10-v1)
      SRC=$(find_input python-3.13.5-src)
      SOURCE_ROOT="$SRC"
      if [ -f "$SRC/Python-3.13.5/configure" ]; then
        SOURCE_ROOT="$SRC/Python-3.13.5"
      fi

      for required_executable in \
        "$PROVIDER/bin/$TARGET-gcc" \
        "$PROVIDER/bin/ar" \
        "$PROVIDER/bin/ranlib" \
        "$MAKE/bin/make" \
        "$BUSYBOX/bin/busybox"; do
        if [ ! -x "$required_executable" ]; then
          echo "ERROR: required Python construction executable is missing: $required_executable" >&2
          exit 1
        fi
      done
      if [ ! -f "$LINUX_HEADERS/include/linux/fs.h" ]; then
        echo "ERROR: declared Linux headers lack include/linux/fs.h: $LINUX_HEADERS" >&2
        exit 1
      fi
      if [ ! -x "$SOURCE_ROOT/configure" ]; then
        echo "ERROR: authenticated Python source has no configure script at either bounded root: $SRC" >&2
        exit 1
      fi

      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK/tools"
      for applet in awk basename cat chmod cmp cp cut date dirname echo env expr find grep head hostname id install ln ls mkdir mv od paste printf pwd readlink realpath rm rmdir sed sh sleep sort tail tee test touch tr uname uniq wc xargs; do
        $BB ln -sf "$BUSYBOX/bin/busybox" "$WORK/tools/$applet"
      done
      $BB ln -sf "$MAKE/bin/make" "$WORK/tools/make"
      $BB ln -sf "$PROVIDER/bin/$TARGET-gcc" "$WORK/tools/cc"
      $BB ln -sf "$PROVIDER/bin/ar" "$WORK/tools/ar"
      $BB ln -sf "$PROVIDER/bin/ranlib" "$WORK/tools/ranlib"
      $BB ln -sf "$PROVIDER/bin/ld" "$WORK/tools/ld"

      export PATH="$WORK/tools"
      export CONFIG_SHELL="$BUSYBOX/bin/sh"
      export SHELL="$BUSYBOX/bin/sh"
      export MAKE="$MAKE/bin/make"
      export CC="$PROVIDER/bin/$TARGET-gcc"
      export AR="$PROVIDER/bin/ar"
      export RANLIB="$PROVIDER/bin/ranlib"
      export LD="$PROVIDER/bin/ld"
      export GREP="$BUSYBOX/bin/grep"
      export EGREP="$BUSYBOX/bin/grep -E"
      export CFLAGS="-O2 -I$LINUX_HEADERS/include"
      export LDFLAGS="-static"
      export SOURCE_DATE_EPOCH=1
      export ac_cv_file__dev_ptmx=yes
      export ac_cv_file__dev_ptc=no

      $BB cp -R "$SOURCE_ROOT" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      MODULE_BUILDTYPE_TEMPLATE="$WORK/src/Modules/Setup.stdlib.in"
      module_buildtype_count=$($BUSYBOX/bin/grep -c '^\*@MODULE_BUILDTYPE@\*$' "$MODULE_BUILDTYPE_TEMPLATE" || true)
      if [ "$module_buildtype_count" -ne 1 ]; then
        echo "ERROR: CPython stdlib module build-type template shape changed: $module_buildtype_count" >&2
        exit 1
      fi
      $BUSYBOX/bin/sed -i 's/^\*@MODULE_BUILDTYPE@\*$/\*static\*/' "$MODULE_BUILDTYPE_TEMPLATE"
      $BUSYBOX/bin/grep -qx '\*static\*' "$MODULE_BUILDTYPE_TEMPLATE"
      $BB mkdir -p "$WORK/build"
      cd "$WORK/build"
      "$BUSYBOX/bin/sh" "$WORK/src/configure" \
        --prefix="$out" \
        --disable-shared \
        --disable-ipv6 \
        --disable-test-modules \
        --without-ensurepip \
        --without-readline
      PYTHON_FOR_REGEN="$BUSYBOX/bin/true" "$MAKE" -j"$JOB_COUNT"
      PYTHON_FOR_REGEN="$BUSYBOX/bin/true" "$MAKE" install

      test -x "$out/bin/python3.13"
      test -x "$out/bin/python3"
      "$out/bin/python3" --version | "$BUSYBOX/bin/grep" -q 'Python 3.13.5'
      "$out/bin/python3" -c 'import json, pathlib, shutil, subprocess, sys, tomllib; assert sys.version_info[:3] == (3, 13, 5); print("mantle-source-python-ok")' \
        | "$BUSYBOX/bin/grep" -q mantle-source-python-ok

      if "$BUSYBOX/bin/timeout" "$TOOL_PROBE_TIMEOUT_SECONDS" "$out/bin/python3" -c 'this is not valid Python syntax (' > "$WORK/malformed.stdout" 2> "$WORK/malformed.stderr"; then
        echo "ERROR: source-built Python accepted malformed input" >&2
        exit 1
      fi
      "$BUSYBOX/bin/grep" -q SyntaxError "$WORK/malformed.stderr"

      RELOCATED_PYTHON="$WORK/relocated-python"
      $BB cp -R "$out" "$RELOCATED_PYTHON"
      PYTHONHOME="$RELOCATED_PYTHON" "$RELOCATED_PYTHON/bin/python3" -c 'import json, pathlib, sys; assert sys.prefix == sys.exec_prefix; print("mantle-source-python-relocated-ok")' \
        | "$BUSYBOX/bin/grep" -q mantle-source-python-relocated-ok

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/full-source-rust-host-python.txt" <<'EOF'
claim: bounded source-built Python host-tool matrix passed
positive: version, Rust-bootstrap stdlib imports, copied-tree execution
negative: malformed Python source rejected with SyntaxError
source_authority: authenticated CPython 3.13.5 source
native_authority: admitted full-source GCC/musl/binutils provider
non_claims: Python correctness, Rust provider completion, whole-bootstrap correctness
EOF
      echo "source-built Python 3.13.5 host-tool matrix passed"
    "%
  ],
  inputs = [native_provider, gnumake, linux_headers, busybox, python_src],
} | crunch.Derivation
