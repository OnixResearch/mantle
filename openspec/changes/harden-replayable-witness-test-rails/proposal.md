# Harden replayable witness test rails

## Why

Recent replayable-witness hardening closed the main helper/CLI scratch-root
mismatch, but review still found two deterministic-test gaps:

- `src/witness_rebuild.rs` rejects symlinked helper-owned scratch entries under
  `tmp/` or `cargo-target/`, but the rail only proves root-level symlink
  rejection explicitly.
- `tests/release_cli.rs` uses a sentinel-based "driver never launched"
  assertion for a negative witness-rebuild path, but that claim should be tied
  to a seam that the fake driver demonstrably consumes.

These are low-severity findings, but they sit on the boundary that makes the
replayable witness story believable. If the rails are ambiguous, future edits
can regress the helper/CLI contract or turn a no-launch assertion into a
vacuous pass.

## What Changes

- **Add direct scratch-boundary coverage.** Extend witness-rebuild unit and CLI
  coverage so helper-owned `tmp/` / `cargo-target/` entries are tested as real
  directories, rejected as symlinks, and rejected as non-directories.
- **Make no-launch assertions observable.** Tighten the fake witness-rebuild
  driver seam so tests that claim the workflow driver never launched depend on
  a signal that the driver implementation actually consumes.
- **Keep the scope on deterministic rails.** Strengthen regression coverage and
  boundary semantics without changing the witnessed-release trust model,
  protocol, or operator workflow.

## Non-Goals

- Changing release-verification policy semantics or trusted-key handling.
- Extending the witness workflow to new rebuild identities.
- Redesigning the helper wrapper or scratch layout beyond what the current rail
  already documents.
- Solving the small TOCTOU window around scratch-root creation in this change.

## Capabilities

### New Capabilities
- `witness-rebuild-scratch-boundary-rail`: deterministic proof that helper-
  owned scratch entries are accepted only as real directories and rejected as
  symlinks or other node kinds.
- `witness-rebuild-launch-observation-rail`: deterministic proof that tests
  claiming the workflow driver never launched use a launch signal the fake
  driver actually consumes.

### Modified Capabilities
- `witness-request-rebuild-runner`: stronger regression coverage for the helper
  wrapper / Rust scratch-validator boundary.

## Impact

- **Files**: `src/witness_rebuild.rs`, `tests/release_cli.rs`, and possibly the
  fake driver helper used by those tests.
- **APIs**: no user-facing CLI or protocol changes.
- **Dependencies**: none.
- **Testing**: add missing positive/negative coverage around helper-owned
  scratch entries and fake-driver no-launch observability.

## Constraints

- The change MUST keep the helper-owned scratch allowance bounded to `tmp/` and
  `cargo-target/` only.
- The change MUST prove helper-owned symlink rejection directly, not only by
  inference from root-level rejection.
- The change MUST keep no-launch assertions tied to an observable seam that the
  fake driver implementation consumes.
- The change MUST include both positive and negative regression coverage.

## Traceability

| Proposal slice | Delta spec |
|---|---|
| Scratch-boundary semantics for helper-owned entries | `specs/release-verification-tech/spec.md` |
| Deterministic driver-launch observation rail | `specs/release-verification-tech/spec.md` |

## How to validate

1. `openspec validate harden-replayable-witness-test-rails` succeeds.
2. `cargo test -p crunch --bin crunch validate_existing_scratch_root -- --nocapture`
   proves helper-owned directory, symlink, and non-directory cases fail or pass
   as intended.
3. `cargo test -p crunch --test release_cli witness_rebuild_ -- --nocapture`
   proves helper happy-path coverage remains green and negative no-launch
   assertions use a real launch-observation seam.
