# Standalone Nickel export core cutover evidence

## Success contract

Goal: adopt `nickel-export-core` at exact revision
`257fafc1c746f1faf156207043a4c826bfb16d49` while retaining every evaluator,
filesystem, destination, build, and release decision in Mantle shells.

Completion evidence requires exact Cargo/Nix/lock/release pins; pure adapter
checks; dual-run canonical identity and Mantle projection agreement; positive and
all requested negative fixtures; focused tests; strict touched-package Clippy;
lock/dependency checks; machine-contract freshness; relevant Nix and release
checks; and passing Cairn validation/gates.

False completion includes a path/floating override, invoking the external CLI,
moving I/O into the shared core, comparing different evaluator observations,
accepting output-byte equality as evaluator equivalence, marking a task from a
planned rather than executed check, or claiming build/release correctness from
an export receipt.

Audit risks are symlink/path escape, source mutation during evaluation, stale or
tampered evidence, mixed evaluator descriptors, receipt emission on evaluator
error, conservative secret markers, weakened non-claims, lock drift, and stale
machine contracts.

Allowed outcomes are validated, exact blocked rail, exhausted bounded review,
or user-decision-required. The final result must not synthesize success around a
failed gate.

Budget: repository sources plus the exact sibling Git revision are the only
implementation sources; Mantle tests, the pinned core API, Nix, machine-contract
checks, and Cairn are authoritative; retrieval is bounded to the change, repo
docs, touched code/contracts, and exact pinned core; tools are local file tools,
Cargo/Nix/Cairn, and one advisory VibeThinker pass; the working context budget is
this isolated session; and the search budget is two implementation lenses, one
adversarial audit, and one final validation round.

## Approach registry

| Family | Mechanism | Artifact/evidence | State |
|---|---|---|---|
| direct replacement | Replace Mantle evaluator/export shell with external CLI | Rejected by authority requirement | falsified |
| copied local core | Vendor/path-copy standalone logic into Mantle | Rejected by immutable release identity and duplicate-core requirement | falsified |
| pinned pure adapter | Exact Git/Nix pin, explicit observations, core admission/projection, Mantle shell authority | Cargo/Nix/config pins; adapter/shell tests | active |
| adversarial rollback audit | Legacy/canonical dual-run, core-owned one-receipt manifest identity, exact v1 projection, classified drift | Positive/negative fixture cycle and rollback selector tests | active |

The surviving mechanism is the pinned pure adapter. VibeThinker supplied an
advisory adversarial pass; its useful source-tamper concern is covered by
post-evaluation recapture, while its suggestions to resolve/follow symlinks or
replace Mantle authority were rejected because the requirement is no-follow
admission and an embedded evaluator shell. Deterministic repository checks remain
authoritative.

## Authority and non-claims

`src/nickel_export_core_adapter.rs` contains no filesystem, environment,
process, network, evaluator, destination, build, or release I/O. It delegates to
the exact standalone core. `src/nickel_export.rs` retains no-follow capture,
`crunch_eval::evaluate_to_json`, post-evaluation source recapture, destination
writes, rendering, and authority selection. Build and release policy remain
outside both modules.

The canonical identity comparison uses the pinned core's `build_manifest` over
one receipt, not an adapter-invented serialization identity. Projection digest
comparison retains the existing Mantle v1 JSON/BLAKE3 compatibility surface.
Passing evidence does not prove evaluator equivalence, complete observed import
closure, semantic/build correctness, deployability, or release eligibility.

## Validation transcript summary

Exact command results are appended only after each command has completed and its
current pueue log has been inspected. No sync or archive command is permitted in
this worktree.
