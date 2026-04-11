# Fetch a real crates.io source tarball.
#
# Usage:
#   mkdir -p /tmp/crunch-examples-store /tmp/crunch-examples-state
#   crunch build examples/fetch-crate-crc64.ncl \
#     --store /tmp/crunch-examples-store \
#     --state-dir /tmp/crunch-examples-state
#
# This uses the published `crc64` crate source from crates.io. The hash
# is the recursive/NAR hash of the unpacked crate tree.

let crunch = import "lib.ncl" in

crunch.fetchTarball {
  url = "https://static.crates.io/crates/crc64/crc64-2.0.0.crate",
  hash = "sha256-G2t3VkrTsVRXQrfE4NYeYPnQUgnzxpGz2eI3QlVE/rA=",
  name = "crc64-src",
}
