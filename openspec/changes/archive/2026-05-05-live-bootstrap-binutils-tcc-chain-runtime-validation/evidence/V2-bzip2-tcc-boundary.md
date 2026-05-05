# V2 bzip2-tcc prerequisite boundary

Task-ID: V2
Covers: `bootstrap.binutils.tcc.runtime-validation`

## Result

BLOCKED: the resumed `bootstrap/bzip2-tcc.ncl` validation reached the bzip2 epoch root but failed before building bzip2 because its dependency `patch-2.5.9-tcc.drv` failed.

This resolves the earlier ambiguous Mes-prerequisite boundary: the long run progressed past the Mes prerequisite far enough for Crunch to report the actual failed dependency. The next blocker is the already split part/runtime-validation scope for patch 2.5.9, not binutils source-chain logic.

Related active change: `live-part-patch-2-5-9-runtime-validation`.

## Command

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/binutils-tcc-chain-store" \
  --state-dir "$PWD/.crunch-drain/binutils-tcc-chain-state" \
  bootstrap validate bootstrap/bzip2-tcc.ncl \
  --evidence-dir openspec/changes/live-bootstrap-binutils-tcc-chain-runtime-validation/evidence \
  --resume
```

Exit status: 1

## Evidence files

- `evidence/doctor.json`
- `evidence/build.stdout.log`
- `evidence/build.stderr.log`
- `evidence/validation-summary.json`
- `evidence/validation-summary.md`
- `evidence/V2-bzip2-tcc-failed-derivation.log`

## Key failure

From `build.stdout.log`:

```json
{
  "root": "bzip2-1.0.8-tcc",
  "phase": "build",
  "error_class": "builder",
  "message": "dependency patch-2.5.9-tcc.drv failed"
}
```

From the copied saved derivation log:

```text
dependency patch-2.5.9-tcc.drv failed
```

## Next boundary

Do not continue V3 epoch-chain validation until `patch-2.5.9-tcc` has runtime evidence or a narrower patch-part blocker. The next OpenSpec to drain is `live-part-patch-2-5-9-runtime-validation`.
