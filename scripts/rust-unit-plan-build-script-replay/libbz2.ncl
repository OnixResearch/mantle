# Mantle-built bzip2 1.0.8 static library for the unit-plan `-sys` fixture.
# Source pin shared with bootstrap/bzip2-1.0.8-musl.ncl; objects compiled with the
# lane's declared glibc C toolchain (gcc-wrapper) so they link into
# x86_64-unknown-linux-gnu Rust units. Installs lib/libbz2.a, include/bzlib.h
# and lib/pkgconfig/bzip2.pc for bzip2-sys's pkg-config probe.
let mantle = import "lib.ncl" in
fun { cc_toolchain, cflags_note } =>
  let bzip2_src = mantle.fetchTarball {
    url = "https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz",
    hash = "sha256-Uvi4JZPPERK3gym4yoaeTEJwKXF5brBAEN7GgF+iF6g=",
    name = "bzip2-1.0.8-src",
  }
  in
  {
    name = "libbz2-1.0.8",
    builder = "/bin/sh",
    args = [
      "-c",
      m%"
        set -eu
        BB=/bin/busybox
        SRC=""
        for candidate in "$NIX_STORE"/*-bzip2-1.0.8-src; do
          if [ -f "$candidate/bzlib.c" ]; then SRC="$candidate"; break; fi
        done
        test -n "$SRC"
        CC="%{cc_toolchain}/bin/cc"
        AR="%{cc_toolchain}/bin/ar"
        WORK=/tmp/libbz2-build
        $BB mkdir -p "$WORK"
        cd "$WORK"
        CFLAGS="-O2 -fPIC -D_FILE_OFFSET_BITS=64 -DMANTLE_LIBBZ2_NOTE=%{cflags_note}"
        for src in blocksort huffman crctable randtable compress decompress bzlib; do
          "$CC" -c $CFLAGS -I"$SRC" "$SRC/$src.c" -o "$src.o"
          test -s "$src.o"
        done
        "$AR" rcs libbz2.a blocksort.o huffman.o crctable.o randtable.o compress.o decompress.o bzlib.o
        $BB mkdir -p "$out/lib/pkgconfig" "$out/include"
        $BB cp libbz2.a "$out/lib/libbz2.a"
        $BB cp "$SRC/bzlib.h" "$out/include/bzlib.h"
        $BB cat > "$out/lib/pkgconfig/bzip2.pc" <<EOF
prefix=$out
libdir=$out/lib
includedir=$out/include

Name: bzip2
Description: Mantle-built bzip2 1.0.8 static library
Version: 1.0.8
Libs: -L$out/lib -lbz2
Cflags: -I$out/include
EOF
        test -s "$out/lib/libbz2.a"
      "%,
    ],
    inputs = [bzip2_src, cc_toolchain],
    addressing_mode = 'input-addressed,
  } | mantle.Derivation
