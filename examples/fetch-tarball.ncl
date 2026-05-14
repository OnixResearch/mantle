# Fetch and unpack a tarball.
#
# Usage: mantle build examples/fetch-tarball.ncl
#
# Downloads the tarball, decompresses (gzip/xz/bz2/zst), extracts,
# strips the top-level directory, and stores the unpacked tree.
# The hash is the NAR hash of the unpacked contents.
#
# To get the correct hash for a new URL:
#   1. Set hash to a dummy value: "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
#   2. Run: mantle build examples/fetch-tarball.ncl
#   3. The error message shows the actual hash — copy it
#   Or: mantle build --fix examples/fetch-tarball.ncl

let mantle = import "lib.ncl" in

mantle.fetchTarball {
  url = "https://github.com/nickel-lang/nickel/archive/refs/tags/1.9.0.tar.gz",
  hash = "sha256-TG6BuV1PotdbSxZECow4OPC1+MxvAX300PMuty5ibsM=",
  name = "nickel-src",
}
