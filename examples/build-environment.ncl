# Pass declarative environment values into a build and persist them as output.
#
# Usage:
#   mantle build examples/build-environment.ncl --no-substitute

let mantle = import "lib.ncl" in
{
  name = "build-environment",
  builder = "/bin/sh",
  env = {
    GREETING = "Hello",
    AUDIENCE = "from Mantle's build environment",
    BUILD_MODE = "tutorial",
  },
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox

      $BB mkdir -p "$out/metadata"
      $BB printf '%s %s!\n' "$GREETING" "$AUDIENCE" > "$out/message.txt"
      $BB printf '%s\n' "$BUILD_MODE" > "$out/metadata/mode.txt"
    "%
  ],
} | mantle.Derivation
