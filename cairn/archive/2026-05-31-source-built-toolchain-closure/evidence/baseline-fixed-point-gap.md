# Baseline evidence: fixed-point succeeds, source-built toolchain closure remains unproven

Task-ID: Spec-1
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Question

What current evidence motivates the next Mantle source-built compiler/toolchain closure workstream?

## Inspected evidence

### Cargo-free self-build probe

Command was launched as pueue task `30` from `/home/brittonr/git/mantle` with a static busybox `SNIX_BUILD_SANDBOX_SHELL` and host rustup nightly/Cargo tool paths.

Pueue result:

```text
Id  Status   Elapsed
30  Success  3m 39s
```

Pueue log:

```text
cargo-free self-build proof OK: /tmp/mantle-no-cargo-self-build-probe-20260531T183224Z
```

Receipt inspection:

```json
{
  "status": "success",
  "blocker": null,
  "unit_count": 598,
  "cargo_mode": {
    "no_cargo_oracle": true,
    "compatibility_class": "cargo-free-native-path-topology-v1",
    "blockers": [],
    "non_claims": [
      "bounded-path-workspace-only",
      "not-full-cargo-feature-resolution",
      "declared-vendor-and-captured-git-only",
      "not-network-or-ambient-cargo-source-resolution"
    ]
  }
}
```

Cargo marker inspection:

```text
cargo-marker-absent
```

### First-class fixed-point command probe

Command was launched as pueue task `32` from `/home/brittonr/git/mantle`:

```text
/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --fixed-point --out /tmp/mantle-fixed-point-fresh-probe-20260531T183625Z
```

Pueue result:

```text
Id  Status   Elapsed
32  Success  7m 9s
```

Output bundle:

```text
/tmp/mantle-fixed-point-fresh-probe-20260531T183625Z
```

Summary JSON from `/tmp/mantle-fixed-point-fresh-probe-20260531T183625Z.stdout`:

```json
{
  "schema": "mantle-cargo-free-fixed-point-proof-v1",
  "status": "success",
  "fixed_point": true,
  "stage1": {
    "execution_status": "success",
    "cargo_marker_absent": true,
    "unit_count": 598,
    "failed_unit_count": 0,
    "binary_blake3": "4f88fea9390fd3a9aaf57765405f39b712d89fd6295dd1ecc53b72a5d362c431"
  },
  "stage2": {
    "execution_status": "success",
    "cargo_marker_absent": true,
    "unit_count": 598,
    "failed_unit_count": 0,
    "binary_blake3": "4f88fea9390fd3a9aaf57765405f39b712d89fd6295dd1ecc53b72a5d362c431"
  },
  "rustc_compatibility": {
    "requested_rustc": "/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc",
    "stage_rustc": "/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc",
    "normalization": "none",
    "wrapper": null,
    "wrapper_blake3": null
  },
  "non_claims": [
    "not-crunch-bootstrap",
    "not-release-reproducibility",
    "not-source-built-toolchain-closure",
    "not-full-cargo-compatibility"
  ]
}
```

Additional inspection:

```text
stderr bytes: 0 /tmp/mantle-fixed-point-fresh-probe-20260531T183625Z.stderr
stage1-no-cargo-marker
stage2-no-cargo-marker
```

### Cairn validation before this change

Command:

```text
/home/brittonr/.cargo-target/debug/cairn validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 3,
  "valid": true
}
```

## Decision

Create a Cairn change for source-built toolchain closure. Current evidence proves the narrower bounded fixed-point command only. It does not prove source-built compiler/toolchain closure because the proof uses the host rustup `rustc` path and explicitly records `not-source-built-toolchain-closure`.

## Owner

Mantle owner / future implementation agent.

## Next action

Implement the smallest source-built toolchain closure slice only after proposal/design/tasks gates pass. Do not remove the current non-claim until a real end-to-end proof bundle records receipt-bound toolchain closure provenance.
