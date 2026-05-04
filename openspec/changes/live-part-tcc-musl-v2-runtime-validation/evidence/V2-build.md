# V2 build attempt: tcc musl v2

- Task-ID: V2
- Covers: `bootstrap.part.tcc.musl.v2.runtime-validation`
- Command: `nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json --store $PWD/.crunch-drain/tcc-musl-v2-store --state-dir $PWD/.crunch-drain/tcc-musl-v2-state bootstrap validate bootstrap/tcc-musl-v2.ncl --evidence-dir openspec/changes/live-part-tcc-musl-v2-runtime-validation/evidence --resume`
- Status: `build-failed`
- Exit code: `1`
- Doctor OK: `True`
- Output path: none; the target did not produce a runtime-valid output.
- Failure class: `build`
- Root: `tcc-0.9.27-musl-v2`
- Derivation key: `/crunch/store/mw16jzmvh5d17kc2y4z8z0sc04086f1r-tcc-0.9.27-musl-v2.drv`
- Blocker: dependency tcc-0.9.27-musl.drv failed

## Evidence files

- `doctor.json`
- `build.stdout.log`
- `build.stderr.log`
- `validation-summary.json`
- `validation-summary.md`
- `V2-tcc-0.9.27-musl-v2-root-derivation.log`

## Root derivation message excerpt

```text
dependency tcc-0.9.27-musl.drv failed
```
