## Context

`crunch release witness-rebuild` now has explicit scratch-root validation and a
checked-in helper wrapper, and the current replayable-witness tests already
cover:

- helper `--check` preflight-only behavior,
- helper and CLI happy paths,
- root-level symlink rejection for the scratch root, and
- several request/output mismatch failures.

The remaining review findings are narrower but important:

1. helper-owned `tmp/` / `cargo-target/` entries have runtime validation logic
   for symlink rejection, but the unit rail does not isolate that exact branch,
   and
2. one no-launch negative test should depend on a seam that the fake driver
   visibly consumes, not only on an env var named by the test.

This change hardens the deterministic rails around the existing behavior. It
is not a trust-model or workflow redesign.

## Goals / Non-Goals

**Goals**
- Add direct, deterministic coverage for helper-owned scratch-entry branches.
- Make fake-driver launch observation explicit so no-launch assertions are not
  vacuous.
- Keep the witness helper / CLI contract stable while tightening the proof
  around it.

**Non-Goals**
- Changing published request layout, release evidence, or witness sidecar
  formats.
- Changing policy evaluation, trusted-key handling, or witness-import
  semantics.
- Solving broader concurrency races around scratch-root creation in this
  change.

## Decisions

### 1. Test helper-owned scratch entries at the validator seam

**Choice:** add direct unit coverage at the Rust validator seam for helper-
owned `tmp/` / `cargo-target/` entries, including accepted-directory,
symlink-rejection, and non-directory-rejection cases.

**Rationale:** the existing CLI tests prove the boundary from farther away, but
review feedback is correct that helper-owned entry rejection deserves one tight,
cheap, deterministic unit rail. This keeps branch coverage focused and makes it
clear which message belongs to helper-owned entries versus whole-root failures.

**Alternative:** rely only on the existing root-symlink integration test.

**Why not:** that leaves the helper-entry branch covered only indirectly and
makes later refactors easier to regress silently.

### 2. Make the fake-driver launch signal a consumed seam

**Choice:** keep the sentinel-style no-launch assertion pattern, but require the
fake witness-rebuild driver implementation to consume the configured launch
signal explicitly and write it as its first visible side effect.

**Rationale:** the review finding is about confidence, not the basic idea. A
launch-signal seam is still a good fit for preflight-failure tests as long as
both sides share the same contract: tests configure it, the fake driver writes
it, and negative tests assert absence.

**Alternative:** delete the sentinel assertion and rely only on stderr text.

**Why not:** that weakens the boundary claim. The change should make the
no-launch proof stronger, not remove it.

### 3. Keep scope on deterministic rails, not new runtime behavior

**Choice:** express the work as spec coverage plus test-rail hardening without
changing user-facing CLI flags, policy files, or workflow identities.

**Rationale:** the reported gaps are about confidence in the existing replayable
witness rail. The highest-value response is to harden that confidence cheaply,
not expand scope into unrelated runtime or protocol work.

## Verification strategy

- unit tests in `src/witness_rebuild.rs` for helper-owned directory, symlink,
  and non-directory cases,
- `tests/release_cli.rs` coverage that proves preflight failures keep the fake
  driver launch signal absent,
- targeted `cargo test` filters for the validator seam and witness rebuild CLI,
  plus `openspec validate harden-replayable-witness-test-rails` after the
  change is fully wired.

## Risks / Trade-offs

**More test-only seam surface.**
The fake-driver launch signal is intentionally test-only. That is acceptable as
long as it stays bounded to the fake driver helper and does not leak into
operator-facing behavior.

**Low implementation scope can look "spec-only."**
This is deliberate. The value here is preventing small regressions in a trust-
bearing rail, not changing the witness workflow itself.
