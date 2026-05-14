# Fetch a single file by URL.
#
# Usage: mantle build examples/fetch-file.ncl
#
# Downloads the file and stores it as a fixed-output derivation.
# The hash pins the content — if the upstream changes, the build fails
# until the hash is updated (or use --fix to auto-update).

let mantle = import "lib.ncl" in

mantle.fetchurl {
  url = "https://raw.githubusercontent.com/nickel-lang/nickel/master/LICENSE",
  hash = "sha256-3jC9jts9aiU9fst7yGXpoRxcwMsB2CQ2EOnFjBhJ3CI=",
  name = "nickel-license",
}
