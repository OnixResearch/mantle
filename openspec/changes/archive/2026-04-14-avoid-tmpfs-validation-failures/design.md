# Design: Avoid tmpfs validation failures

## Context

The checked-in self-hosting proof helper drives one of the heaviest local
validation workflows in the repo. It can produce GiBs of Cargo artifacts and
temporary files. If it inherits ambient `/tmp` on a host where `/tmp` is
backed by tmpfs, the proof can fail for environmental reasons even when the
main filesystem has plenty of free space.

The helper should own scratch selection explicitly instead of inheriting host
defaults blindly.

## Goals / Non-Goals

**Goals:**

- make `scripts/prove-self-hosting.sh` prefer disk-backed scratch by default
- keep scratch selection explicit and overridable through one named interface
- fail fast with actionable diagnostics when the chosen scratch filesystem is too small
- document direct heavyweight `cargo` guidance without replacing the proof helper

**Non-Goals:**

- wrap every possible manual Cargo invocation in a repo helper
- hide genuine out-of-space failures on the filesystem the operator explicitly selected
- change the canonical self-hosting proof entry point away from `scripts/prove-self-hosting.sh`

## Decisions

### 1. Use one named proof-scratch override

**Choice:** `scripts/prove-self-hosting.sh` MUST resolve its scratch root from
`CRUNCH_PROOF_SCRATCH_DIR` when that environment variable is set.

If `CRUNCH_PROOF_SCRATCH_DIR` is unset, the helper MUST fall back to the
repo-root anchored repo-local disk-backed path `target/self-hosting-proof/work/`.

A scratch root is usable only when the helper can create it if needed,
confirm it is a directory, and confirm it is writable.

If `CRUNCH_PROOF_SCRATCH_DIR` is set and that selected path is not usable, the
helper MUST fail immediately with no fallback to `target/self-hosting-proof/work/`.

If `CRUNCH_PROOF_SCRATCH_DIR` is unset and the repo-local fallback path is not
usable, the helper MUST fail before the proof starts.

**Rationale:** one named override makes diagnostics and regression tests
unambiguous.

**Implementation:** keep selection logic in a small helper that resolves the
chosen root relative to the repo root when needed, returns whether it came from
`CRUNCH_PROOF_SCRATCH_DIR` or the default repo-local policy, and enforces the
create-or-reject usable-path check before proof work starts.

### 2. Route both Cargo artifacts and temp files under one root

**Choice:** the selected scratch root MUST drive both `TMPDIR` and
`CARGO_TARGET_DIR` through predictable subdirectories.

`CRUNCH_PROOF_SCRATCH_DIR` is the only operator-facing override interface for
proof scratch selection in this change. `TMPDIR` and `CARGO_TARGET_DIR` remain
helper-managed outputs of that selected root, not additional public override
knobs.

**Rationale:** moving only one class of files off tmpfs still leaves the other
half able to fail mid-run, and multiple public override knobs would undermine
the single-interface contract.

**Implementation:** derive separate subdirectories under the selected root, for
example `tmp/` and `cargo-target/`, so the helper can report them and clean
them predictably.

### 3. Preflight uses a fixed 4 GiB threshold

**Choice:** the helper MUST fail before the long proof run begins when the
selected scratch filesystem has less than 4 GiB free.

**Rationale:** the repo already treats the proof as a multi-GiB workflow. A
fixed threshold makes behavior deterministic and testable.

**Implementation:** expose the free-space probe behind a narrow helper seam so
regression tests can inject values below and above the 4 GiB threshold. The
error must name the selected scratch root and `CRUNCH_PROOF_SCRATCH_DIR` as the
override interface.

### 4. Direct cargo guidance stays explicit and subordinate to the proof helper

**Choice:** docs MAY show disk-backed `TMPDIR` and `CARGO_TARGET_DIR` examples
for heavyweight direct `cargo` commands, but self-hosting proof docs MUST keep
`scripts/prove-self-hosting.sh` as the canonical proof entry point.

**Rationale:** contributors sometimes need direct `cargo` commands, but that
must not replace the checked-in proof helper for self-hosting evidence.

**Implementation:** document a direct `cargo` example for a heavyweight
non-proof validation command and a short note explaining why the proof helper
still owns the self-hosting workflow.

## Risks / Trade-offs

**[Repo-local scratch grows large]**

Proof artifacts move onto the main filesystem.

**Mitigation:** keep them under predictable repo-local directories so they are
inspectable and easy to remove.

**[4 GiB is conservative, not perfect]**

The proof could still fail later if a host needs more space than the preflight
minimum.

**Mitigation:** treat the threshold as an early blocker for clearly undersized
filesystems, not as a guarantee that later writes can never fail.

**[Contributors still run raw cargo with ambient defaults]**

Direct commands can still inherit tmpfs if the operator ignores the docs.

**Mitigation:** document the override knobs explicitly and keep the checked-in
proof helper safe by default.
