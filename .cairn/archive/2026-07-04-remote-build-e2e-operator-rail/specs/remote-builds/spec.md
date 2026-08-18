## ADDED Requirements

### Requirement: Operator remote-build e2e rail is proof-bound and redacted

r[remote_builds.operator_e2e_rail] Mantle MUST provide a deterministic operator-facing remote-build e2e validation rail that composes route planning, framed handshake, source/input sync, remote execution, signed output admission, and bounded observability without depending on ambient network services or hidden global state. The rail MUST produce or assert machine-readable evidence for the selected route, handshake phase, upload summary, execution phase, transfer/admission phase, signer or trust basis, artifact-attestation reference, log/status bounds, redaction, and explicit non-claims.

#### Scenario: successful fixture proves remote composition only

GIVEN a concrete build request has a compatible remote route, bounded upload requirements, and explicit output trust for the builder key
WHEN the operator rail executes the remote-build fixture
THEN Mantle MUST complete route planning, framed handshake, input sync, remote execution, signed output admission, and report rendering through the same core validation seams used by supported remote-build operation
AND the resulting evidence MUST state that the rail proves fixture composition only, not production P2P deployment, release reproducibility, or general package-manager compatibility.

#### Scenario: cross-seam failures fail closed

GIVEN the rail fixture lacks output trust, emits unframed stdout, presents stale source state, exceeds upload quota or privacy policy, or requests fallback without an explicit policy
WHEN Mantle runs the corresponding negative case
THEN Mantle MUST reject the remote path before output admission or local-success reporting
AND diagnostics MUST identify the phase and stable reason code without revealing bearer tickets, private key paths, raw environment values, uploaded content, or unbounded logs.
