# V2 Compile/link/execute smoke

Task-ID: V2
Covers: bootstrap.compiler.tinycc.0.9.27.amd64.static-link

## Command

```sh
timeout 1200 nix shell nixpkgs#bubblewrap -c "$PWD/target/debug/crunch" build --no-substitute \
  --store "$PWD/.crunch-drain/tcc-link-store-no-crti" \
  --state-dir "$PWD/.crunch-drain/tcc-link-state-no-crti" \
  bootstrap/diag-tcc-link-smoke.ncl
```

## Result

Status: FAIL / next blocker
Exit status: 1
Transcript: `evidence/V2-clean-link-smoke.log`
Saved derivation log: `.crunch-drain/tcc-link-state-no-crti/logs/gr46ci91cz76dcq11an1fp70fq0cdhb3-diag-tcc-link-smoke.drv.log`

The repaired TinyCC now compiles and links the explicit static executable, but executing `./hello` segfaults with exit 139. This moves the active blocker from TinyCC executable-output/link-time segfaults to runtime behavior of the produced static executable.
