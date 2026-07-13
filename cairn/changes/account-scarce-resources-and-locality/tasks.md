## Implementation

- [x] [serial] I1 Inventory current scheduler classes, remote route gates, worker registration, coordinator leases, transfer missing-set facts, and typed Nickel profiles; record exact extension seams and compatibility treatment. r[build_scheduling.quantified_resource_admission]
- [x] [depends:I1] I2 Add bounded provider-neutral action requirements, worker capacities, named token pools, locality observations, and reservation-lease DTOs with checked arithmetic. r[build_scheduling.quantified_resource_admission]
- [x] [depends:I2] I3 Implement pure validation, semantic-versus-scheduling classification, resource fit, reservation/release/recovery, and stable rejection reasons. r[build_scheduling.quantified_resource_admission]
- [x] [depends:I2] I4 Implement pure locality normalization from receiver-verified object presence and demanded counts/bytes, including freshness and manifest/policy binding. r[build_scheduling.verified_locality_placement]
- [x] [depends:I3] I5 Persist job/attempt/fence-bound worker resource leases atomically in the coordinator and recover or release them conservatively across completion, cancellation, timeout, worker loss, and restart. r[remote_builds.fenced_worker_resource_leases]
- [x] [depends:I4,I5] I6 Feed derived hard eligibility and normalized preference classes into the existing lazy scheduler without adding provider ordering or bypassing trust/privacy/network/prefix gates. r[build_scheduling.verified_locality_placement]
- [x] [depends:I2] I7 Extend `lib/remote-builders.ncl` and scheduling policy with typed bounded resource/token/locality fields and Rust/Nickel parity checks. r[remote_builds.fenced_worker_resource_leases]
- [x] [depends:I6,I7] I8 Extend remote status, plan, scheduler-decision, and build reports with bounded requirement, reservation, locality, transfer, and stable rejection summaries. r[build_scheduling.verified_locality_placement]

## Verification

- [x] [depends:I3] V1 Positive: reserve and release compatible CPU/memory/scratch/accelerator/token vectors; prove equivalent snapshots produce identical decisions and checked arithmetic preserves capacity invariants. r[build_scheduling.quantified_resource_admission]
- [x] [depends:I3] V2 Negative: reject zero/oversized quantities, duplicate/conflicting token names, arithmetic overflow, unavailable capacities, semantic-classification drift, and aggregate overcommit before assignment. r[build_scheduling.quantified_resource_admission]
- [x] [depends:I5] V3 Positive and negative: run concurrent assignment and restart fixtures; prove leases prevent overcommit, current fences can recover/release, and stale attempts cannot mutate current capacity. r[remote_builds.fenced_worker_resource_leases]
- [x] [depends:I4] V4 Positive and negative: derive locality from verified full/partial/missing input sets, reject stale or wrong-manifest observations, and prove unverified worker hints cannot claim zero transfer. r[build_scheduling.verified_locality_placement]
- [x] [depends:I6] V5 Prove better verified locality/resource fit changes only eligible-worker preference while starvation and every hard blocker remain authoritative. r[build_scheduling.verified_locality_placement]
- [ ] [depends:I8] V6 Run focused scheduler/distributed/remote tests, Nickel contract tests, format and first-party lint checks, Cairn validate, Tracey coverage, and proposal/design/tasks gates with exact evidence. r[remote_builds.fenced_worker_resource_leases]
  - Current evidence: focused tests pass (`43 + 29 + 123 + 20 + 3`, zero failures), targeted format passes, policy freshness/validate/proposal/design/tasks pass, and changed `crunch-build` code has no strict-Clippy diagnostic. Exact commands and results are in `evidence/implementation-validation.md`.
  - Blocker: the checked-in first-party Clippy rail stops on pre-existing `crunch-project-core`/`crunch-build`/`src/build_correctness.rs` diagnostics, while the checked-in `mantle-default` Tracey profile scans only release-provenance and fails on unrelated historical missing/dangling identifiers. V6 remains unchecked; no broad lint or Tracey pass is claimed.
