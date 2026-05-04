# V2 build attempt: tcc musl v2

- Task-ID: V2
- Covers: `bootstrap.part.tcc.musl.v2.runtime-validation`
- Command: `nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json --store $PWD/.crunch-drain/tcc-musl-v2-store --state-dir $PWD/.crunch-drain/tcc-musl-v2-state bootstrap validate bootstrap/tcc-musl-v2.ncl --evidence-dir openspec/changes/live-part-tcc-musl-v2-runtime-validation/evidence --resume`
- Status: `build-failed`
- Exit code: `1`
- Doctor OK: `True`
- Output path: none; the target did not build.
- Failure class: `build`
- Blocker: prerequisite derivation `sed-4.0.9-tcc.drv` failed before `tcc-0.9.27-musl-v2` builder execution.

## Evidence files

- `doctor.json`
- `build.stdout.log`
- `build.stderr.log`
- `validation-summary.json`
- `validation-summary.md`
- `V2-tcc-musl-v2-root-derivation.log`

## Root derivation log excerpt

```text
# crunch build log
# derivation: tcc-0.9.27-musl-v2
# drv_path: 301hmgjshvqc71y8h5jx1i91iqx6wsqb-tcc-0.9.27-musl-v2.drv
# status: failure
# timestamp: 1777901450

dependency sed-4.0.9-tcc.drv failed
```
