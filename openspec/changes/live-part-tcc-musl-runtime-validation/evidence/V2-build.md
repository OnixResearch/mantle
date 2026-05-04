# V2 build attempt: tcc musl

- Task-ID: V2
- Covers: `bootstrap.part.tcc.musl.runtime-validation`
- Command: `nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json --store $PWD/.crunch-drain/tcc-musl-store --state-dir $PWD/.crunch-drain/tcc-musl-state bootstrap validate bootstrap/tcc-musl.ncl --evidence-dir openspec/changes/live-part-tcc-musl-runtime-validation/evidence --resume`
- Status: `build-failed`
- Exit code: `1`
- Doctor OK: `True`
- Output path: none; the target did not produce a runtime-valid output.
- Failure class: `build`
- Root: `tcc-0.9.27-musl`
- Derivation key: `/crunch/store/5k4dnz4wkxlv3sys10yab6g0wcmi5jzv-tcc-0.9.27-musl.drv`
- Blocker: dependency tcc-0.9.27-musl-prep.drv failed

## Evidence files

- `doctor.json`
- `build.stdout.log`
- `build.stderr.log`
- `validation-summary.json`
- `validation-summary.md`
- `V2-tcc-0.9.27-musl-root-derivation.log`

## Root derivation message excerpt

```text
dependency tcc-0.9.27-musl-prep.drv failed
```
