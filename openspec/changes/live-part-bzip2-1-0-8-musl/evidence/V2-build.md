# V2 build attempt: bzip2 1.0.8 (musl)

- Task-ID: V2
- Covers: `bootstrap.part.bzip2.1.0.8.musl.evidence-isolated`
- Command: `nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json --store $PWD/.crunch-drain/bzip2-musl-store --state-dir $PWD/.crunch-drain/bzip2-musl-state bootstrap validate bootstrap/bzip2-1.0.8-musl.ncl --evidence-dir openspec/changes/live-part-bzip2-1-0-8-musl/evidence --resume`
- Status: `build-failed`
- Exit code: `1`
- Doctor OK: `True`
- Output path: none; the target did not build.
- Failure class: `build`
- Blocker: prerequisite derivation `tcc-0.9.27-musl-v2.drv` failed before `bzip2-1.0.8-musl` builder execution.
- Follow-up owner: existing OpenSpec change `live-part-tcc-musl-v2-runtime-validation`.

## Evidence files

- `doctor.json`
- `build.stdout.log`
- `build.stderr.log`
- `validation-summary.json`
- `validation-summary.md`
- `V2-bzip2-musl-root-derivation.log`

## Root derivation log excerpt

```text
# crunch build log
# derivation: bzip2-1.0.8-musl
# drv_path: vz6yg4d032cykbf274hggrmprb11bxpm-bzip2-1.0.8-musl.drv
# status: failure
# timestamp: 1777899769

dependency tcc-0.9.27-musl-v2.drv failed
```
