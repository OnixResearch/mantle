# Evidence-driven remote resource policy

Mantle can compare declared resource requirements with bounded prior
observations. This feature is off by default. The default mode records a policy
decision but keeps the static scheduler choice.

## Ownership

`crunch-resource-policy-core` owns deterministic policy meaning. It uses only
supplied data and supports `no_std + alloc`.

`crunch-resource-policy` owns application ports for usage-ledger commits and
Valence publication. `src/resource_policy.rs` maps accepted decisions to the
existing coordinator, fenced-attempt, and strong action-result APIs.

The existing components keep their authority:

- The remote coordinator owns hard worker eligibility and assignment.
- The resource lease core owns current capacity and fenced reservations.
- The attempt core owns attempt identity and fence advancement.
- The action-result core owns signature, policy, output, identity, and CAS fact
  admission.
- Valence owns evidence linkage and verification roles.
- OnixOS owns machine declarations.
- ChaosControl owns fault execution.

## Action-family identity

An action-family identity contains only public request features:

- request kind and system;
- builder class;
- sandbox and network modes;
- required platform features;
- semantic accelerator classes.

The identity excludes environment values, token claims, credentials, and log
content. The contract is
`schemas/machine-contracts/resource-action-family.schema.json`.

## Resource observations

A terminal observation records declared and selected resources, bounded timing
and measurements, terminal outcome, collector version, and optional OOM facts.
It binds the public attempt, action family, platform, and machine class.

Selection rejects observations that are missing, stale, future-dated,
untrusted, malformed, duplicated, incompatible, or outside measurement
bounds. Every exclusion has a stable reason code. The contract is
`schemas/machine-contracts/resource-observation.schema.json`.

An observation is not a resource promise. It cannot reduce a declared minimum
or remove architecture, platform, KVM, trust, isolation, or feature needs.

## Selection

The selector performs these steps:

1. Validate the policy, request, class declarations, quotas, and observations.
2. Select the smallest statically eligible class by ordinal and class ID.
3. Filter history with explicit reasons.
4. If enough compatible samples exist, apply named memory and scratch margins.
5. Select the smallest class that meets the effective requirements.
6. Apply project and account quota facts when quota enforcement is enabled.
7. In observe-only mode, schedule the static class and record the alternative.

The coordinator intersects the selected class with its own eligible worker set.
It checks the effective quantities against worker capacity and reserves those
same quantities in the fenced lease.

## OOM escalation

OOM retry is a separate feature. A retry requires:

- an opted-in project and enforcement mode;
- the OOM retry control;
- trusted positive platform-specific OOM evidence;
- a strictly larger eligible class;
- remaining retry, ordinal, cumulative-charge, wall-time, and quota limits.

An ambiguous exit or generic failure does not authorize escalation. The
adapter passes an accepted retry plan to the existing attempt core. That core
creates a different attempt identity and advances the fence.

## Usage accounting

Admission creates project and account reservations with explicit units,
windows, schedules, policy IDs, and attempt IDs. Completion reconciles one
reservation into a bounded charge and release.

Identical duplicate operations reuse the existing record. A conflicting
operation fails. Cancellation, worker loss, and partial observations have
separate outcomes. A compare-and-commit failure returns no quota grant and
leaves the loaded state unchanged.

## Result sharing

Private project scope is the default. Named cross-project sharing requires an
explicit producer and consumer list. Reuse also requires all existing strong
result checks. The policy adapter cannot turn a rejected or conflicting strong
result plan into a usable result.

A successful decision records a producer-to-consumer evidence link. A denied
consumer does not mutate or delete the producer result.

## Evidence

The application adapter emits the accepted
`valence.build-service-evidence.v1` records from Valence revision
`e40c76b4d2070a29636e00c85c0dff93f03dba2f`. It preserves field roles:
selection is verified, observations and usage are recorded-only, retries are
linked, and reuse is verified.

Valence validates identity and linkage. It does not prove resource policy,
build output behavior, accounting fairness, or release eligibility.

## Benchmarks and faults

`fixtures/resource-policy/benchmark-corpus.json` contains fixed-input bounded
facts. Its first workload adapts the shape of
`nixbuild/nixbench` `write-one-file` at revision
`b256cd275d8c79ba485be8d317005f973879825a`. The source is Apache-2.0. Mantle
copies no implementation or hosted-service policy.

`fixtures/resource-policy/chaoscontrol-campaign.valid.json` binds a reviewed
ChaosControl runtime-capacity source and covers positive OOM, ambiguous
failure, worker loss, duplicate completion, accounting interruption, and CAS
unavailability.

Reports keep correctness, compatibility, throughput, latency, memory,
transfer, and usage facts separate.

## Rollout

`config/resource-policy.ncl` is the typed source of policy defaults. Its
generated JSON is `config/generated/resource-policy.json`.

All feature controls default to false:

- historical selection;
- OOM retry;
- quota enforcement;
- result sharing.

Operators can enable each control independently for selected projects. A
rollback can disable one control without disabling observation collection.

## Validation

Run the focused checks:

```text
cargo test -p crunch-resource-policy-core
cargo check -p crunch-resource-policy-core --target wasm32-unknown-unknown
cargo test -p crunch-resource-policy
cargo test -p mantle --test resource_policy_evidence
cargo test -p mantle --bin mantle resource_policy
cargo -Zscript scripts/check-resource-policy-architecture.rs --root .
cargo -Zscript scripts/check-machine-schema-contracts.rs
nickel typecheck config/resource-policy.ncl
```

## Non-claims

This feature does not prove future resource sufficiency, build correctness,
host isolation, fair billing, production performance, source trust, or release
eligibility.
