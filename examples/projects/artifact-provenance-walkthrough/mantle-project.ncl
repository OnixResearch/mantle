# Artifact and runtime-closure provenance walkthrough.
let mantle = import "../../../lib/lib.ncl" in

let source_artifact = {
  name = "provenance-walkthrough-source",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      "$BB" mkdir -p "$out"
      "$BB" printf '%s\n' 'provenance-walkthrough: declared source' > "$out/source.txt"
    "%
  ],
} | mantle.Derivation in

let assembled_artifact = {
  name = "provenance-walkthrough-artifact",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  inputs = [source_artifact],
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      source_path=""
      for candidate in "$NIX_STORE"/*-provenance-walkthrough-source; do
        if [ -f "$candidate/source.txt" ]; then
          source_path="$candidate"
          break
        fi
      done
      if [ -z "$source_path" ]; then
        "$BB" printf '%s\n' 'declared source input was not mounted' >&2
        exit 1
      fi
      "$BB" mkdir -p "$out"
      "$BB" cp "$source_path/source.txt" "$out/artifact.txt"
      "$BB" printf '%s\n' 'provenance-walkthrough: assembled' >> "$out/artifact.txt"
      "$BB" printf '%s\n' "$source_path" > "$out/runtime-reference.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    source = source_artifact,
    artifact = assembled_artifact,
  },
  default = { package = "artifact" },
} | mantle.Project
