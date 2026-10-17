# Pin the current Picolibc release source for the StageX comparison diagnostic.
#
# Research-only input for the `evaluate-picolibc-stagex-runtime` change. This
# pin does not admit Picolibc into the StageX lineage, the normalized
# provider graph, or any parity row.
#
# Pin source: pins/picolibc.toml; checked derived reader: pins/generated/picolibc.json.
let crunch = import "../lib/lib.ncl" in
let pin = import "pins/generated/picolibc.json" in

{
  picolibc-src = crunch.fetchTarball {
    url = pin.artifacts.source.url,
    hash = pin.artifacts.source.hash,
    name = "picolibc-" ++ pin.version ++ "-src",
  },
}
