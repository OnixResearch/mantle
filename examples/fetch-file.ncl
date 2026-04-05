# Fetch a single file by URL.
#
# Usage: crunch build examples/fetch-file.ncl
#
# Downloads the file and stores it as a fixed-output derivation.
# The hash pins the content — if the upstream changes, the build fails
# until the hash is updated (or use --fix to auto-update).

let crunch = import "lib.ncl" in

crunch.fetchurl {
  url = "https://raw.githubusercontent.com/nickel-lang/nickel/master/LICENSE",
  hash = "sha256-jOtLnHt0MU0Bfrekml4dJqHFdMOBkJMkbTcMmPRsaJo=",
  name = "nickel-license",
}
