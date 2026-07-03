## Context

`docs/release-notes/provider-bound-release-evidence-2026-06-28-provider-remap-fixed.md` records a successful provider-bound release evidence bundle and a local/provider-bound witness replay. It also states that stronger external wording requires sending the witness request to a separate operator or machine and importing their returned sidecars.

This change is primarily an evidence and workflow handoff. It should not change release verification semantics unless the handoff exposes a missing fail-closed check.

## Decisions

### 1. Treat operator independence as evidence, not code metadata

**Choice:** The tracked evidence must name the external witness identity, request digest, returned sidecar digest, verification status, and operator-independence basis. The release verifier still evaluates cryptographic signatures, policy, revocations, release digest, and rebuilt binary digests.

**Rationale:** Social independence cannot be inferred from a file path. It must be recorded as an evidence claim bounded by observed operator handoff facts.

### 2. Keep public request material key-free

**Choice:** The exported request contains only public release evidence, release attestation, request metadata, and verifier material. It must not contain signing keys or verifier-local private state.

**Rationale:** Witness handoff should be safe to send to another operator.

### 3. Verify both happy path and fail-closed paths

**Choice:** Completion requires a positive external witness import/verify and at least one negative check, such as missing signature, wrong release digest, duplicate conflicting witness identity, unknown key, or policy-insufficient witness set.

**Rationale:** Release evidence is a trust surface. Negative cases prove the handoff cannot be silently weakened.

## Risks / Trade-offs

- External witness turnaround may be operationally blocked. If so, the change should record a blocker and keep local/provider-bound wording unchanged.
- A separate machine may produce environmental differences. The final claim must be limited to the policy-counted witness agreement actually verified.
