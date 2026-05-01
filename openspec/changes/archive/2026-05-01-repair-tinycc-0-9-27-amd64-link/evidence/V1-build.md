# V1 Build bootstrap/tinycc.ncl

Task-ID: V1
Covers: bootstrap.compiler.tinycc.0.9.27.amd64.static-link

## Command

```sh
timeout 1200 nix shell nixpkgs#bubblewrap -c "$PWD/target/debug/crunch" build --no-substitute \
  --store "$PWD/.crunch-drain/tcc-link-store-no-crti" \
  --state-dir "$PWD/.crunch-drain/tcc-link-state-no-crti" \
  bootstrap/tinycc.ncl
```

## Result

Status: PASS
Exit status: 0
Transcript: `evidence/V1-clean-tinycc-build.log`

The build used the explicit local Crunch store/state directories under `.crunch-drain/` and did not use host substitution.
