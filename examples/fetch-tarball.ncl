# Fetch and unpack a tarball.
#
# Usage: crunch build examples/fetch-tarball.ncl
#
# Downloads the tarball, decompresses (gzip/xz/bz2/zst), extracts,
# strips the top-level directory, and stores the unpacked tree.
# The hash is the NAR hash of the unpacked contents.
#
# To get the correct hash for a new URL:
#   1. Set hash to a dummy value: "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
#   2. Run: crunch build examples/fetch-tarball.ncl
#   3. The error message shows the actual hash — copy it
#   Or: crunch build --fix examples/fetch-tarball.ncl

let crunch = import "lib.ncl" in

crunch.fetchTarball {
  url = "https://github.com/nickel-lang/nickel/archive/refs/tags/1.9.0.tar.gz",
  hash = "sha256-TG6BuV1PotdbSxZECow4OPC1+MxvAX300PMuty5ibsM=",
  name = "nickel-src",
}
