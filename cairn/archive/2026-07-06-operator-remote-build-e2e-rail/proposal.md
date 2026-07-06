## Why

The `remote-builds` spec already accepts `[depends:remote_builds.operator_e2e_rail]`:
Mantle MUST provide a deterministic operator-facing remote-build e2e validation
rail that composes route planning, framed handshake, source/input sync, remote
execution, signed output admission, and bounded observability without ambient
network services or hidden global state, and MUST emit machine-readable evidence
with explicit non-claims and cross-seam fail-closed behavior.

The production-remote-build-farm change landed the production scheduler,
cryptographic trust, streaming transfer, coordinator runtime, publication, and
CI-boundary requirements, and its V13 task ran the focused unit/integration
tests plus gates. What it did not land is a single bounded multi-process
operator rail that proves the *composition* of those seams end to end and emits
the machine-readable, redacted, fixture-composition-only evidence the accepted
requirement mandates. Partial rail surface already exists
(`src/remote_build.rs`, `tests/remote_stdio_cli.rs`, and the parity/determinism
scripts), but it has not been audited against every clause of the accepted
requirement, and the cross-seam negative cases are not asserted as a set.

This change closes that gap so the week's remote-build archival is backed by a
proven operator workflow rather than a spec-only claim.

## What Changes

- Audit the existing operator e2e rail surface against every scenario clause of
  `[depends:remote_builds.operator_e2e_rail]` and record covered phases and gaps.
- Promote the rail into a single bounded local multi-process fixture that
  composes route planning, framed handshake, source/input sync, remote
  execution, signed output admission, and bounded observability through the same
  core validation seams used by supported remote-build operation.
- Emit machine-readable JSON rail evidence naming the selected route, handshake
  phase, upload summary, execution phase, transfer/admission phase, signer or
  trust basis, artifact-attestation reference, log/status bounds, redaction, and
  explicit non-claims (fixture composition only).
- Add negative cross-seam cases that fail closed before output admission or
  local-success reporting, each reporting phase + stable reason code without
  revealing bearer tickets, private key paths, raw environment values, uploaded
  content, or unbounded logs.

## Impact

- **Files**: `src/remote_build.rs`, `tests/remote_stdio_cli.rs`,
  `scripts/bootstrap-parity-snapshot.rs`, `scripts/release-determinism-smoke.rs`
  and/or a new `scripts/prove-operator-remote-build-rail.rs`, plus any shared
  rail-evidence helper extracted into `crates/crunch-build` or `src/`.
- **Testing**: a bounded local multi-process positive fixture, a set of
  cross-seam negative fixtures, redaction assertions, determinism assertions,
  and the Cairn proposal/design/tasks gates for this change.

## Out of Scope

- Production P2P deployment, multi-region builder pools, or real SSH endpoints.
- Release reproducibility or bootstrap correctness claims.
- Package-manager compatibility claims.
- Changes to the accepted `[depends:remote_builds.operator_e2e_rail]` requirement text.
