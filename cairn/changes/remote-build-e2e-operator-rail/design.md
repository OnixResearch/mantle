## Context

Remote-build support has landed as a set of narrow seams: deterministic route plans, output-trust admission, source-bundle input sync, hardened stdio/SSH bindings, scheduler-compatible dispatch, and bounded observability. Each seam has focused positive and negative tests, but the current evidence does not prove that an operator can run one fixture through the whole path and inspect a single coherent report.

## Decisions

### 1. Use an in-process fixture around the stdio-compatible protocol core

**Choice:** The first rail will exercise the same framed protocol, request validation, upload/admission helpers, and report models as stdio/SSH operation while keeping process and filesystem setup deterministic inside the test harness.

**Rationale:** This gives composition evidence without depending on a long-lived external builder, SSH daemon, network availability, or ambient host state. Transport process-spawn coverage remains in the existing stdio/SSH tests; this change proves that the protocol and build-result path compose end to end.

### 2. Keep proof claims narrow and machine-checkable

**Choice:** The rail will assert explicit evidence fields for route phase, handshake phase, upload summary, execution result, transfer/admission result, signer/trust basis, artifact-attestation reference, log/status bounds, and non-claims.

**Rationale:** Mantle's verification rules require proof-before-claim. A single rail is valuable only if it names what it proves and what it does not prove.

### 3. Put failure cases beside the happy path

**Choice:** Negative fixtures will use the same fixture harness and fail before output admission for missing output trust, protocol stdout pollution, stale source identity, quota/privacy rejection, and fallback without an explicit policy.

**Rationale:** The e2e rail must prove fail-closed behavior across seams, not just a success path.

## Risks / Trade-offs

- A deterministic in-process fixture is not a production network proof. Reports must call this out as a non-claim.
- Overly broad assertions can make the rail brittle. Keep exact checks on stable evidence fields and redaction guarantees, not incidental formatting.
- The rail must not bypass the pure core helpers in order to be convenient; otherwise it would stop proving the real route/admission/input-sync seams.
