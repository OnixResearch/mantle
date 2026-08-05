# Build the fix Nix evaluator from the pinned source.
#
# Toolchain inputs (Zig, libcurl, libgit2) are signed nixpkgs cache closures
# declared in nixpkgs-toolchain.ncl. They are binary trust inputs, not
# source-built artifacts. Build this file with the same --nix-compat store
# and state that pulled the toolchain closures:
#
#   mantle --nix-compat --store <store> --state-dir <state> build packages/fix/fix.ncl
#
# The output is the fix producer backend binary. It is a Mantle build
# product; this build says nothing about evaluator correctness.

let crunch = import "../../lib/lib.ncl" in
let src = import "fix-src.ncl" in
let tc = import "nixpkgs-toolchain.ncl" in

{
  fix = {
    name = "fix-0.3.0",
    builder = "/bin/sh",
    system = 'x86_64-linux,
    args = [
      "-c",
      m%"
        set -eu
        BB=/bin/busybox

        SRC=$(echo $NIX_STORE/*-fix-0.3.0-src)
        $BB mkdir -p work
        $BB cp -r "$SRC" work/src
        $BB chmod -R u+w work/src
        cd work/src

        export HOME=$PWD
        export XDG_CACHE_HOME=$PWD/.cache
        export PATH="%{tc.zig}/bin:%{tc.pkg_config}/bin"
        export PKG_CONFIG_PATH="%{tc.libgit2_dev}/lib/pkgconfig:%{tc.curl_dev}/lib/pkgconfig:%{tc.openssl_dev}/lib/pkgconfig:%{tc.zlib_dev}/lib/pkgconfig"

        %{tc.zig}/bin/zig build --release=fast --prefix "$out"

        "$out/bin/fix" --version || "$out/bin/fix" --help
      "%,
    ],
    outputs = ["out"],
    env = {},
    inputs = [
      src.fix-src,
      tc.zig,
      tc.libgit2,
      tc.libgit2_lib,
      tc.libgit2_dev,
      tc.curl,
      tc.curl_dev,
      tc.openssl,
      tc.openssl_dev,
      tc.zlib,
      tc.zlib_dev,
      tc.libssh2,
      tc.pkg_config,
    ],
    addressing_mode = 'input-addressed,
    sandbox = 'native,
  } | crunch.Derivation,
}
