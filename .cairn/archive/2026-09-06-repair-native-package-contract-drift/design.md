# Native package contract parity

## Context

The package check now continues after a failing target. Nine later targets expose drift that the former fail-fast run did not reach.
The failure inventory is evidence of symptoms, not a complete diagnosis.
ADR 0120 records the selected enumeration, diagnostic framing, generator, and scan-budget decisions.
Focused baselines must separate invalid fixtures from actual implementation defects before each repair.

## Decisions

### Decision: Accepted contracts govern each repair

**Choice:** Preserve accepted input, authority, output, and non-claim semantics. Correct fixtures only when their data or expected category contradicts those contracts.

**Rationale:** A failure message alone cannot justify a weaker check. Valid fixtures must reach the intended boundary. Negative fixtures must reject before protected effects.

The foreign import owner remains its typed admission core and parser adapter. Tests must maintain valid output/environment bindings except in explicit invalid-semantics cases.
The build-tool boundary must reject raw module inventory without prescribing one obsolete rendering path. Its source scan must remain bounded and cover all required files.

### Decision: Keep generated projections subordinate to their owners

**Choice:** Generate operator artifacts from the typed Nickel inventory and parser description. Derive machine cohort membership from the inventory, with uniqueness and exact membership checks.

**Rationale:** Updating a magic number or inserting an empty required field can conceal missing coverage. New example entries need honest prerequisites and support classifications.

### Decision: Preserve composed reads and local write authority

**Choice:** Inspect the base-only read path and its actual error before implementation. Correct only the adapter/service handoff that loses admitted data or layer facts.

**Rationale:** ADR 0012 and `store_lifecycle.overlay_sandbox_view` require one composed read view. `store_lifecycle.capability_only_access` forbids broad authority escape.

The functional core retains deterministic ordering, validation, and decisions over explicit domain facts.
`crunch-store` retains database, castore, signature, filesystem, generation, and effect observations.
Callers retain narrow capabilities. No new raw service accessor or mutable base capability is permitted.
Positive tests must exercise reads after reopening state and assert no backfill. Negative tests must retain trust, writable-base, prefix, generation, and shadow rejection.

### Decision: Gateway store authority is operation-specific

**Choice:** Replace the gateway's broad handle and raw services with a `crunch-store`-owned `GatewayStore` capability.
It exposes exact path/hash reads, bounded NAR ingest observations, and imported-output persistence. It cannot expose services, GC, source admission, signing, or root registration.

**Rationale:** The unchanged architecture guard finds six violations at the published prerequisite and in the current branch.
Existing borrowed transfer operations are reused inside the store shell. A new owned capability keeps their lifetime inside the gateway actor without giving it build or administration authority.

The existing remote core retains request authority and limits. The application retains explicit trusted keys and signature verification before imported-output persistence.
The store adapter forces non-root, castore-only persistence with no invented provenance or physical export.
It binds imported metadata to the observed NAR node, digest, size, and owning open store instance. An actor-local opaque token cannot cross stores or survive reopening as admission.
Fresh authority is checked again before final persistence. Rejection may leave unindexed received blobs, but cannot publish an admitted path or output.
Tests cover accepted import and reopened query, incorrect hash/size/signature, denied authority before reads, trust-override rejection, and compile-time capability denial.
The unchanged global architecture guard and its positive/negative self-tests must pass; no file or identifier exception is added.

### Decision: Worker and coordinator diagnostic facts remain distinct

**Choice:** Trace the existing worker bundle reference and capture status through the failure shell and status projection.

**Rationale:** Coordinator metadata-only capture cannot replace an observed worker capture. Conversely, a bundle reference alone cannot invent captured bytes.

Any changed deterministic selection rule belongs in the existing functional core, using explicit worker/coordinator facts.
Loading bundles, measuring content, cleanup, persistence, and JSON rendering remain shell effects.
Existing capture allowlists, sensitivity rules, quotas, build failure, and output non-admission remain unchanged.

## Verification strategy

First run focused baselines for `foreign_import_cli`, `integration_build`, and `store_gc_cli` with the pinned package development environment.
Then run each other target before changing its boundary. Add accepted and rejected controls for every changed policy or adapter boundary.
Run strict Clippy for changed first-party scope and the pinned Nix Tiger Style check.
Run generator freshness checks for each changed generated artifact.
The NAR guard must follow Rust-cache delegation through `RustCacheStore::render_nar` to the existing Snix writer, rather than require the removed direct call.
Test both missing delegation edges. Keep source commit and package checksum checks unchanged.
Prepare the exact locked `nix-archive` crate through Cargo vendoring for that source check; sparse guard input does not prove complete self-build vendoring.

Commit source before the exact native package run. Use `cargo test --release --locked --no-fail-fast` through the existing default package.
The operator cap is 60 minutes, two cores, one job, no remote builders, and a 30-second termination grace.
A timeout is not an assertion failure. Ignored cases and unrelated full-source proofs remain outside this acceptance claim.
Do not retry an unchanged failed package without a new explicit decision.

## Risks / Trade-offs

- A valid-looking fixture can fail before its intended trust boundary. Diagnostics and absence-of-effect assertions must expose that failure.
- A generated artifact can change public meaning. Review the producer and semantic diff, not just freshness success.
- A store fix can accidentally widen authority. Keep capability topology and hostile base/shadow controls.
- A debug status fix can conflate worker and coordinator observations. Test both present and unavailable worker capture.
- Full package success remains separate from consumer admission, reproducibility, deployment, and release authority.

## Rollback

The repair uses a dedicated branch and worktree. No schema migration or production state mutation is planned.
If a boundary needs new policy or an incompatible format, keep the task open and revise the design before implementation.
Only a passing, evidenced change can sync, archive, or integrate into `main`.
