# Source-built BusyBox for the full-source Rust host-tool closure.
#
# The installed core-utility bundle is compiled by the admitted native
# provider with independently source-built GNU Make and Linux UAPI headers.

let crunch = import "lib.ncl" in
let native_provider = crunch.derivationFile "seed-full-toolchain.ncl" in
let gnumake = crunch.derivationFile "make-4.4.1-gcc10.ncl" in
let linux_headers = crunch.derivationFile "linux-headers-6.6-gcc10.ncl" in
# Upstream archive SHA-256: 3311dff32e746499f4df0d5df04d7eb396382d7e108bb9250e7b519b837043a4.
let busybox_src = crunch.fetchTarball {
  url = "https://busybox.net/downloads/busybox-1.37.0.tar.bz2",
  hash = "sha256-tDFhcU0GUX3/ct2Hw+Ad8iNyMl0GWovhQynLgooh+qU=",
  name = "busybox-1.37.0-full-source-src",
} in

{
  name = "busybox-1.37.0-full-source-gcc10-v1",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      JOB_COUNT=1
      WORK=/tmp/busybox-1.37.0-full-source
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
      HEADERS=$(find_input linux-headers-6.6-full-source-gcc10-v3)
      SRC=$(find_input busybox-1.37.0-full-source-src)
      test -x "$PROVIDER/bin/$TARGET-gcc"
      test -x "$MAKE/bin/make"
      test -s "$HEADERS/include/linux/capability.h"
      test -s "$SRC/Makefile"

      $BB rm -rf "$WORK"
      $BB mkdir -p "$WORK/tools" "$WORK/compat-libs"
      for applet in awk basename bzip2 bzcat bunzip2 cat chmod cmp comm cp cut date dd dirname echo env expr false find grep gunzip gzip head hostname id install ln ls mkdir mktemp mv od paste printenv printf pwd readlink realpath rm sed sh sleep sort tail tee test touch tr true uname uniq wc which whoami xargs zcat; do
        $BB ln -sf "$BB" "$WORK/tools/$applet"
      done
      $BB ln -sf "$MAKE/bin/make" "$WORK/tools/make"
      $BB ln -sf "$PROVIDER/bin/$TARGET-gcc" "$WORK/tools/cc"
      $BB ln -sf "$PROVIDER/bin/$TARGET-gcc" "$WORK/tools/gcc"
      for tool in ar as ld nm objcopy objdump ranlib readelf strip; do
        test -x "$PROVIDER/bin/$tool"
        $BB ln -sf "$PROVIDER/bin/$tool" "$WORK/tools/$tool"
      done

      export PATH="$WORK/tools"
      export CC="$PROVIDER/bin/$TARGET-gcc"
      export HOSTCC="$PROVIDER/bin/$TARGET-gcc -static"
      export AR="$PROVIDER/bin/ar"
      export RANLIB="$PROVIDER/bin/ranlib"
      export LD="$PROVIDER/bin/ld"
      "$PROVIDER/bin/ar" rcs "$WORK/compat-libs/libresolv.a"
      "$PROVIDER/bin/ranlib" "$WORK/compat-libs/libresolv.a"
      export LC_ALL=C
      export TZ=UTC
      export SOURCE_DATE_EPOCH=1
      export KCONFIG_NOTIMESTAMP=1
      export KBUILD_BUILD_TIMESTAMP='Thu, 01 Jan 1970 00:00:01 +0000'
      export KBUILD_BUILD_USER=mantle
      export KBUILD_BUILD_HOST=mantle
      export KBUILD_BUILD_VERSION=1

      $BB cp -R "$SRC" "$WORK/src"
      $BB chmod -R u+w "$WORK/src"
      cd "$WORK/src"
      "$MAKE/bin/make" defconfig CC="$CC" HOSTCC="$HOSTCC"
      $BB sed -i 's/# CONFIG_STATIC is not set/CONFIG_STATIC=y/' .config
      $BB sed -i 's/# CONFIG_INSTALL_NO_USR is not set/CONFIG_INSTALL_NO_USR=y/' .config
      $BB sed -i 's/CONFIG_TC=y/# CONFIG_TC is not set/' .config
      LINUX_INC="-I$HEADERS/include"
      "$MAKE/bin/make" -j"$JOB_COUNT" \
        CC="$CC" \
        HOSTCC="$PROVIDER/bin/$TARGET-gcc" \
        HOSTLDFLAGS="-static" \
        LDFLAGS="-static -L$WORK/compat-libs" \
        CFLAGS="-Os -pipe $LINUX_INC"
      test -x busybox

      $BB mkdir -p "$out/bin"
      $BB cp busybox "$out/bin/busybox"
      $BB chmod 755 "$out/bin/busybox"
      for applet in $("$out/bin/busybox" --list); do
        $BB ln -sf busybox "$out/bin/$applet"
      done
      "$out/bin/busybox" sh -c 'echo mantle-source-busybox-ok' \
        | "$out/bin/busybox" grep -q mantle-source-busybox-ok
      "$out/bin/busybox" grep --help > "$WORK/grep-help.stdout" 2> "$WORK/grep-help.stderr"
      if "$out/bin/busybox" grep '[' "$WORK/grep-help.stdout" > "$WORK/malformed.stdout" 2> "$WORK/malformed.stderr"; then
        echo "ERROR: source-built BusyBox grep accepted malformed pattern" >&2
        exit 1
      fi

      RELOCATED_BUSYBOX="$WORK/relocated-busybox"
      $BB cp -R "$out" "$RELOCATED_BUSYBOX"
      "$RELOCATED_BUSYBOX/bin/busybox" sh -c 'echo mantle-source-busybox-relocated-ok' \
        | "$RELOCATED_BUSYBOX/bin/busybox" grep -q mantle-source-busybox-relocated-ok

      $BB mkdir -p "$out/share/mantle-bootstrap"
      $BB cat > "$out/share/mantle-bootstrap/full-source-rust-host-busybox.txt" <<'EOF'
claim: bounded source-built BusyBox core-utility matrix passed
positive: static shell/core utility execution and copied-tree execution
negative: malformed grep pattern rejected
source_authority: authenticated BusyBox 1.37.0 and Linux 6.6 sources
native_authority: admitted full-source provider plus independently built GNU Make
local_runtime_normalization: empty libresolv.a compatibility archive because musl provides resolver symbols in libc
non_claims: BusyBox correctness, Rust provider completion, whole-bootstrap correctness
EOF
      echo "source-built BusyBox 1.37.0 host-tool matrix passed"
    "%
  ],
  inputs = [native_provider, gnumake, linux_headers, busybox_src],
} | crunch.Derivation
