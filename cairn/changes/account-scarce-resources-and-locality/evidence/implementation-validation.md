# Resource accounting and locality implementation evidence

## Scope

This evidence covers the active Cairn change `account-scarce-resources-and-locality`.
The implementation is in commits:

- `45dbac0c` — pure bounded resource/locality core, initial ADR, architecture inventory.
- `ab687980` — durable production leases, receiver-verified locality placement, typed Nickel policy, bounded reports, and production-path tests.

A later evidence-only checkpoint may include lint cleanup, task markers, and this transcript. No sync or archive operation was run.

## Implementation map

- `crates/crunch-build/src/distributed/remote_resources.rs`
  - bounded provider-neutral CPU, memory-byte, scratch-byte, accelerator, and named-token DTOs;
  - canonical validation, checked arithmetic, deterministic fit and reservation planning;
  - job/attempt/fence/worker-generation-bound durable lease identity;
  - exact stale-fence authorization, release, resize, snapshot validation, and restart recovery plans;
  - receiver-verified locality normalization bound to worker generation, canonical manifest digest, and policy digest;
  - deterministic resource/locality placement over eligible workers only.
- `src/remote_build.rs`
  - clone-plan-persist-publish coordinator mutations for registration, dispatch, reassignment, terminal completion/failure, cancellation, timeout, and worker loss;
  - cross-process coordinator mutation lock remains the production serialization boundary;
  - restart reconciliation preserves only exact current live leases, releases stale/non-live leases, and fail-closes quantified live jobs missing a lease;
  - worker generation changes invalidate locality and cannot inherit a prior-generation active lease;
  - locality is publicly recorded only through the receiver filesystem probe shell;
  - status, dispatch decisions, and build observability expose bounded requirements, lease scope/digests, remaining capacity, fit, locality counts/bytes/classes, stable reasons, and non-claims.
- `src/main.rs` and `src/remote_farm_config.rs`
  - production CLI selection threads configured worker generation, concurrency, and explicit resource inventory into coordinator registration;
  - absent inventory remains unknown and is never inferred from concurrency.
- `lib/remote-builders.ncl`, `lib/scheduling.ncl`, and `tests/scheduling_policy.rs`
  - typed inventory, requirement, named-token, semantic accelerator, locality-policy, and bounded scheduling contracts;
  - Rust/Nickel limit and default parity checks.
- `adr/0025-reserve-remote-resources-with-fenced-leases.md`
  - records hard-admission-before-preference, fenced durability, receiver verification, worker-generation freshness, identity treatment, and report non-claims.

## Functional-core / imperative-shell boundary

The core consumes explicit in-memory inventories, requirements, active leases, transfer manifests, receiver facts, and scheduling policy. It performs no filesystem, process, clock, network, lock, or persistence operation. The root-package shell owns receiver probing, cross-process locking, state loading, atomic temp-file replacement, and presentation.

The production shell does not fabricate known-graph pressure or provider order. Existing ready-goal ranking remains authoritative for root selection. Worker placement filters every system, semantic feature, sandbox, network, store-prefix, output-key, concurrency, and quantified-resource blocker before applying resource/locality preference.

## Validation evidence

### Focused current-tree tests and format

Pueue task `2947` ran the required isolated environment:

```text
CARGO_TARGET_DIR=/home/brittonr/.cache/mantle-drain-targets/resource-locality
TMPDIR=/home/brittonr/.cache/mantle-drain-targets/resource-locality/tmp
```

It ran these exact commands:

```text
nix develop -c cargo test -q -p crunch-build --lib distributed::tests
nix develop -c cargo test -q -p crunch-build --lib scheduling::tests
nix develop -c cargo test -q -p mantle --bin mantle remote_build::tests
nix develop -c cargo test -q -p mantle --bin mantle remote_farm_config::tests
nix develop -c cargo test -q -p mantle --test scheduling_policy
nix develop -c rustfmt --edition 2024 --check crates/crunch-build/src/distributed/remote_resources.rs src/remote_build.rs src/remote_farm_config.rs tests/scheduling_policy.rs
```

Observed results:

```text
crunch-build distributed: 43 passed; 0 failed
crunch-build scheduling: 29 passed; 0 failed
mantle remote_build: 123 passed; 0 failed
mantle remote_farm_config: 20 passed; 0 failed
scheduling_policy: 3 passed; 0 failed
format: PASS
```

The suites include both positive and negative cases for every changed boundary: valid reserve/release/replay, every individual and aggregate capacity blocker, zero/oversized/duplicate/mismatched shapes, arithmetic safety, stale mutation, snapshot corruption, active and terminal restart recovery, receiver full/partial/missing facts, wrong scope/generation/unverified hints, hard-blocker precedence, stable worker ordering, starvation authority, concurrent coordinator locking, registration shrink, worker-generation rollover, all terminal release causes, scheduling-only identity invariance, semantic identity change, and Nickel type/runtime parity.

### Lint evidence and bounded blocker

Pueue task `2913` ran the checked-in `./scripts/check-first-party-clippy.sh`. It stopped in pre-existing `crunch-project-core` strict warnings, including `large_enum_variant`; it did not reach a repository-wide pass.

Pueue task `2939` reran strict crate-scoped Clippy:

```text
nix develop -c cargo clippy -p crunch-build --lib --no-deps -- -D warnings
```

One new `remote_resources.rs` warning found by the first run was fixed. The rerun reported no `remote_resources.rs` diagnostic, then stopped on ten pre-existing diagnostics in other `crunch-build` modules (`derivable_impls`, needless explicit lifetimes, large `Err` variants, a large enum variant, and type complexity).

Pueue task `2943` ran:

```text
nix develop -c cargo clippy -p mantle --bin mantle --no-deps -- -D warnings
```

It stopped before checking the binary on two pre-existing `src/build_correctness.rs` diagnostics (`result_large_err` at line 495 and `collapsible_if` at line 543). No changed root-package file appeared in the diagnostic transcript. Therefore strict first-party lint is recorded as blocked, not passed.

### Cairn and Tracey evidence

Pueue task `2908` ran policy freshness, Cairn validation, proposal/design/tasks gates, then broad Tracey coverage with the checked-in generated policy. Policy freshness, validation, and all three gates completed before Tracey ran. Tracey returned `valid: false` with receipt hash `5ccd67f3a626523e835d60e0360c2aa2fba9fe8d125470bea469559be8b7d536`.

Pueue task `2912` captured the full Tracey receipt at:

```text
/home/brittonr/.cache/mantle-drain-targets/resource-locality/tmp/resource-locality-tracey.json
```

The checked-in `mantle-default` Tracey profile currently scans only `cairn/specs/release-provenance/spec.md` and seven release-evidence sources. Its broad failures are established unrelated release-provenance missing/dangling identifiers. None of these active change's three identifiers appears in the missing or dangling lists:

```text
build_scheduling.quantified_resource_admission
build_scheduling.verified_locality_placement
remote_builds.fenced_worker_resource_leases
```

Because the configured profile neither includes the active build-scheduling/remote-build delta as a requirement source nor passes broadly, this is not Tracey success evidence. Verification task V6 remains unchecked until the repository-wide policy/profile blocker is resolved or an authoritative profile covers these specs and passes.

## Adversarial oracle checkpoint

### Checkpoint: lease atomicity and restart ownership

- **Question:** Can concurrent dispatch, reassignment, restart, or a stale attempt double-allocate or release current capacity?
- **Inspected evidence:** clone-persist-publish mutations, cross-process mutation guard in the production CLI, exact lease/job/attempt/fence/generation linkage validation, pure recovery plans, concurrent lock/retry fixture, restart fixture, and stale-loss negative fixture.
- **Decision:** Accepted. One candidate state owns both attempt and lease mutation; stale scopes fail before mutation; restart preserves only an exact current live scope and otherwise releases/fail-closes.
- **Owner:** Mantle remote coordinator (`src/remote_build.rs`) with pure authorization in `remote_resources.rs`.
- **Next action:** Keep new production mutation entry points inside `acquire_remote_coordinator_mutation_guard`; add the same concurrent/restart fixture for any new mutation path.

### Checkpoint: locality freshness and honesty

- **Question:** Can stale or sender-advertised content become `FullyPresent` or zero-transfer evidence?
- **Inspected evidence:** receiver filesystem probe shell, canonical manifest/policy binding, worker endpoint/generation binding, unverified conservative normalization, generation invalidation test, and advisory VibeThinker audit.
- **Decision:** Accepted after hardening. Worker generation is now explicitly the incarnation and destructive-content freshness fence and is included in resource lease scope. Workers must advance it before restart, eviction, or another destructive cache transition; additive changes can only underclaim until reprobed.
- **Owner:** Remote worker registration protocol and receiver-probe shell.
- **Next action:** Any future cache eviction API must advance worker generation before publishing the changed receiver state.

### Checkpoint: advisory findings disposition

- **Question:** Did the secondary audit identify an unaddressed concrete invariant failure?
- **Inspected evidence:** VibeThinker findings compared against actual coordinator lock ownership, exact current-scope recovery, lack of lease expiry/candidate-lease concepts, hard-filter-before-rank control flow, and private direct locality recorder.
- **Decision:** The freshness concern was valid and hardened as above. Claims about absent locking, lease expiry, candidate lease reuse, and locality creating eligibility did not match the implementation and were rejected. No advisory output was treated as authoritative without repository tests.
- **Owner:** This change's implementation review.
- **Next action:** Re-run focused negative tests and final Cairn gates after evidence/task-marker edits.

## Final current-tree lifecycle gates

Pueue task `2965` ran the exact policy freshness command and returned:

```text
policy fresh: cairn-policy/generated/cairn-policy.json
```

Pueue task `2964` ran the exact final current-tree commands with `--policy cairn-policy/generated/cairn-policy.json`. Full JSON receipts are stored under the required isolated target directory as `resource-locality-cairn-final-{validate,proposal,design,tasks}.json`.

Results:

```text
validate: valid=true, changes=8, specs_validated=35, issues=[]
proposal: PASS, input=a8e29c6514b5d355db981e5ecf46a1052923bc5556b6470f64bd489fa96c636c, receipt=a09f3a070670f2bf9eb7926d6b9280310cd8d77f5b420be2c9b0ba251e4858a4
design: PASS, input=2bb400546ca2e9d9fc60fbbb8c401b231858998a69ffeac6469a2c98a8d1f82e, receipt=3a183ab9a2faa6a65c31c39be5a2fbde287c7043c74ba9a558aa47a3f7f9e137
tasks: PASS, input=5273b332a766b4656dc09fa900398b8d61c4eca4432abab23e602b409c8c4a52, receipt=767814215bf70f3052d85a76977a5e40df773ec8f83e034db8e99fd8f1c3fbc8
policy hash: d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c
```

No sync or archive command was run.

## Honest non-claims

This evidence does not prove global placement optimality, makespan improvement, provider autoscaling behavior, future cache retention, tool identity, license compliance, output trust, attestation validity, execution success, release reproducibility, or a repository-wide strict-Clippy/Tracey pass.
