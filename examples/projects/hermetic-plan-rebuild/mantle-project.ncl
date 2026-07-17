# Strict plan and clean-rebuild identity walkthrough.
let mantle = import "../../../lib/lib.ncl" in

let package = {
  name = "hermetic-plan-rebuild-payload",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      HOST_INPUT_REJECTED_EXIT=64
      if [ -n "${MANTLE_EXAMPLE_HOST_TOKEN:-}" ]; then
        "$BB" printf '%s\n' 'ambient host token leaked into the build' >&2
        exit "$HOST_INPUT_REJECTED_EXIT"
      fi
      "$BB" mkdir -p "$out"
      "$BB" printf '%s\n' 'hermetic-plan-rebuild: stable payload' > "$out/payload.txt"
      "$BB" printf 'HOME=%s\nLANG=%s\nLC_ALL=%s\nSOURCE_DATE_EPOCH=%s\nTZ=%s\n' \
        "$HOME" "$LANG" "$LC_ALL" "$SOURCE_DATE_EPOCH" "$TZ" > "$out/environment.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = { payload = package },
  default = { package = "payload" },
} | mantle.Project
