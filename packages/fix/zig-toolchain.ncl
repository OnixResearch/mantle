# Pin the Zig toolchain used to build the fix backend.
#
# This is a fixed-output binary distribution, not a source-built compiler.
# Receipts must name it as a trust input. Building Zig from source is a
# non-goal of the `add-nix-producer-adapter` change.
#
# Upstream: https://ziglang.org/download/0.16.0/zig-x86_64-linux-0.16.0.tar.xz
# Upstream flat sha256 (informational, not the pin):
#   70e49664a74374b48b51e6f3fdfbf437f6395d42509050588bd49abe52ba3d00
#
# To update the pin: set the new URL, use a dummy hash, run
# `mantle build packages/fix/zig-toolchain.ncl --fix`, and commit the result.

let crunch = import "../../lib/lib.ncl" in

{
  zig-toolchain = crunch.fetchTarball {
    url = "https://ziglang.org/download/0.16.0/zig-x86_64-linux-0.16.0.tar.xz",
    hash = "sha256-Bgb91FvN9FDW0bCoP3tdr+5XLZqd5LKoKlzv472t6s0=",
    name = "zig-0.16.0-toolchain",
  },
}
