# Use one cache-imported Nixpkgs shell as the executable builder.
let mantle = import "lib.ncl" in
let mksh = "/nix/store/baa9yl7sazygz4k4ma2v343n4aadjhlj-mksh-59c" in
let mksh_builder = mksh ++ "/bin/mksh" in

{
  name = "cache-imported-mksh-builder-use",
  builder = mksh_builder,
  args = [
    "-c",
    m%"
      set -eu
      print -r -- "builder=$0" > "$out"
      print -r -- "provenance=cache-imported" >> "$out"
      print -r -- "workflow=nixpkgs-tool-use" >> "$out"
      print -r -- "execution=imported-builder" >> "$out"
    "%
  ],
  inputs = [mksh],
  addressing_mode = 'input-addressed,
} | mantle.Derivation
