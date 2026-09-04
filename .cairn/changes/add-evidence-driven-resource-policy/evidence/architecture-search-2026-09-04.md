# Resource-policy architecture search

Date: 2026-09-04

## Question

Where should Mantle own historical resource selection, OOM escalation,
accounting, sharing, and evidence without widening runtime authority?

## Inspected evidence

- `crates/crunch-build/src/distributed/remote_resources.rs` owns current
  quantified lease and locality decisions.
- `crates/crunch-remote-core/` and `crates/crunch-remote/` demonstrate the
  workspace functional-core and application-port pattern.
- `crates/crunch-action-result-core/` owns existing signature, output, policy,
  identity, and CAS fact admission for strong result reuse.
- Valence profile `valence.build-service-evidence.v1` at immutable revision
  `e40c76b4d2070a29636e00c85c0dff93f03dba2f` owns canonical evidence linkage,
  roles, and non-claims.
- ChaosControl `runtime_capacity` keeps bounded capacity admission and fault
  accounting in a deterministic core.
- `nixbuild/nixbench` revision
  `b256cd275d8c79ba485be8d317005f973879825a` is Apache-2.0. Its
  `write-one-file` workload separates rebuild seed, file seed, size,
  compressibility, and CPU request. Mantle can adapt this workload shape
  without copying the implementation or hosted-service policy.

Reference BLAKE3 values:

- `nixbuild/nixbench` README: `95bbe021e75cbd2caf651604fa9b70adb23ad27845d49bc3c68a56b8ac829279`.
- Apache-2.0 license: `14ec1590aae4c4e763d123c31085d5d37703f7a838e589b0d356058ad99177dd`.
- `nix/packages/write-one-file.nix`: `737a5a42181fdd59ba451159db7558c516f31ea82ffcc771f66a0f2d6421a5a8`.

## Candidate portfolio

### Extend `crunch-build::distributed::remote_resources`

Rejected. That module owns current lease capacity and locality. Historical
selection, quota ledgers, sharing policy, and evidence export would mix four
new authorities into an already large std crate.

### Extend `crunch-remote-core`

Rejected. The crate owns generic remote session transitions and effect plans.
Resource-policy evolution has an independent schema, rollout, and test matrix.

### Add a narrow resource-policy component

Selected. `crunch-resource-policy-core` owns deterministic policy meaning over
already supplied facts. `crunch-resource-policy` owns application ports and
Valence projection. Root adapters own files, clocks, existing coordinator
mapping, action-result probes, and output rendering.

## Decision

Use a separate `no_std + alloc` core. Keep observations non-authoritative,
declared minima hard, OOM evidence positive and platform-specific, accounting
mutations idempotent, and sharing private by default. Reuse existing lease,
fence, and strong-result admission through adapters instead of copying them.

## Owner

Mantle owns resource policy, retry, accounting, sharing, and rollout. Valence
owns evidence linkage semantics. OnixOS owns machine declarations.
ChaosControl owns fault execution. Existing Mantle components retain lease,
store, executor, and result-admission authority.

## Next action

Implement the core and ports, then add contracted fixtures, observe-only
integration, benchmark and fault evidence, and focused validation.
