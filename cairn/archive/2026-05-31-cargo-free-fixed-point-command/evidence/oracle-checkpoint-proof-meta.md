# Oracle checkpoint: existing proof metadata for fixed-point command proposal

## Question

Can this change proposal cite the existing `/tmp` Cargo-free self-build and fixed-point proof metadata as motivating evidence for making the fixed-point rail a first-class command?

## Inspected evidence

Command run from repo root:

```sh
jq '{status, binary_blake3, unit_count, failed_unit_count, cargo_marker_absent, smoke_status_code}' \
  /tmp/mantle-cargo-free-next/meta.json
jq '{status, fixed_point, stage1_digest: .stage1.binary_blake3, stage2_digest: .stage2.binary_blake3, stage1_units: .stage1.unit_count, stage2_units: .stage2.unit_count, cargo_stage1_absent: .stage1.cargo_marker_absent, cargo_stage2_absent: .stage2.cargo_marker_absent}' \
  /tmp/mantle-cargo-free-fixed-point-next/meta.json
```

Observed output:

```json
{
  "status": "success",
  "binary_blake3": "e61c9464c62d15c4ac59e89a8c3ad0ca4f68081c3baf9e0b9ef9cff9be702a82",
  "unit_count": 599,
  "failed_unit_count": 0,
  "cargo_marker_absent": true,
  "smoke_status_code": 0
}
{
  "status": "success",
  "fixed_point": true,
  "stage1_digest": "3f446343d104469490b3aa9d86bd0e54260f5f5c0be453c90884b96a90ac5273",
  "stage2_digest": "3f446343d104469490b3aa9d86bd0e54260f5f5c0be453c90884b96a90ac5273",
  "stage1_units": 599,
  "stage2_units": 599,
  "cargo_stage1_absent": true,
  "cargo_stage2_absent": true
}
```

## Decision

The proposal may cite these `/tmp` metadata files as local motivating evidence only. They are not durable release evidence and do not complete any task in this active change.

## Owner

Mantle maintainer / implementation agent.

## Next action

Implement `mantle self-build --cargo-free --fixed-point --out <bundle-dir>` and record fresh repo-local evidence from the first-class command before checking verification tasks complete.
