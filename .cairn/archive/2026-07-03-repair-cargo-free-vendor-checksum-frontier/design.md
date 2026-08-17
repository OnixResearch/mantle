## Context

`mantle self-build --cargo-free --fixed-point` now emits a deterministic blocker diagnostic. The latest known frontier is `vendor-checksum-mismatch` for `registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3`, with the failure rooted in native registry source planning before any stage1 binary is produced.

This change treats the mismatch as an input-material integrity problem, not as a planner success. The repair must preserve the guarantee that native planning rejects source drift rather than papering over it.

## Decisions

### 1. Repair material, not checks

**Choice:** Make `Cargo.lock`, vendored package contents, Cargo checksum metadata, and native source-material records agree. Do not weaken checksum comparison or silently accept mismatches.

**Rationale:** The proof is only meaningful if the source closure is the one described by the lockfile and vendored metadata.

### 2. Keep the classifier as the first validation rail

**Choice:** Re-run focused native registry source planning and Cargo-free blocker classifier tests before launching the expensive fixed-point proof.

**Rationale:** The fast checks prove the known blocker has moved or disappeared before spending time on the full proof.

### 3. Record moved-frontier evidence honestly

**Choice:** If the fixed-point proof remains blocked by a new deterministic blocker, update docs/evidence with that exact class, package/unit, receipt digests, and non-claim language.

**Rationale:** Fresh blocked evidence is still progress, but it is not proof success.

## Risks / Trade-offs

- Refreshing vendored material can create large diffs.
- Repairing one package may reveal the next source-material or topology frontier.
- The expensive proof can fail for environmental reasons unrelated to the checksum repair; evidence must distinguish those cases.
