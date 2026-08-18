# Completion Status — Project lock importers

Date: 2026-07-01

## Question

Can `project-lock-importers` be honestly drained from the current tree without overclaiming freshness/fetch/trust/non-Git VCS support that belongs to other active changes?

## Inspected evidence

- `crates/crunch-project-core/src/importer.rs` now defines the normalized `ExternalPinSet`, external pin/patch/hash metadata, `PinImportPlan`, blocker taxonomy, planned file operations, mapped-input/mapped-patch reports, preserved/rewritten semantics, non-claims, and future adapter seams.
- The core maps supported Nixtamal file, tarball, and Git pins plus mirrors, patches, BLAKE3/SHA-256/SHA-512 hashes, frozen flags, lock identities, and bounded supported metadata values into Mantle project manifest, lock, and generated-input file operations.
- Unsupported or not-yet-owned semantics are fail-closed blockers: unsupported source kinds, unsupported hash algorithms, missing Git rev identity, unknown patches, command/composition semantics, future flake/npins/niv adapters, and existing-file conflicts.
- `src/pin_import.rs` is the imperative shell: it reads a bounded Nixtamal JSON fixture, records existing Mantle-owned files, renders no-mutate plans, and applies only the recomputed blocker-free planned files.
- `src/main.rs` exposes `mantle import pins plan|apply` alongside the existing Cargo importer.
- `tests/pin_import_cli.rs` proves no-mutate planning, reviewed apply writing only planned Mantle files, and blocker/conflict apply refusal.

## Decision

Resolved for this change. The importer is intentionally scoped to data Mantle can model today. Freshness, fetch policy, trust policy, and non-Git VCS values that are not implemented yet are preserved only when they are bounded labels already understood by the importer; otherwise they remain deterministic blockers. This avoids silently dropping semantics and avoids claiming the future active changes are complete.

## Owner

Mantle project workflow/importer owner.

## Next action

Archive after Cairn validation/gates pass. Future changes can widen the importer by replacing blocker-only labels with first-class project freshness, fetch-policy, trust-policy, and forge-agnostic VCS models.
