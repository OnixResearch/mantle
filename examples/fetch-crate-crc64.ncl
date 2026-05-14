# Fetch a real crates.io source tarball.
#
# Usage:
#   mkdir -p /tmp/mantle-examples-store /tmp/mantle-examples-state
#   mantle build examples/fetch-crate-crc64.ncl \
#     --store /tmp/mantle-examples-store \
#     --state-dir /tmp/mantle-examples-state
#
# This uses the published `crc64` crate source from crates.io. The hash
# is the recursive/NAR hash of the unpacked crate tree.

let mantle = import "lib.ncl" in

mantle.fetchTarball {
  url = "https://static.crates.io/crates/crc64/crc64-2.0.0.crate",
  hash = "sha256-G2t3VkrTsVRXQrfE4NYeYPnQUgnzxpGz2eI3QlVE/rA=",
  name = "crc64-src",
}
