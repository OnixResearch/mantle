## ADDED Requirements

### Requirement: Operator remote-build e2e rail composition proof emits bounded versioned evidence

r[remote_builds.operator_e2e_rail_composition_proof] Mantle MUST provide a bounded local multi-process composition proof of the operator remote-build e2e rail that composes route planning, framed handshake, source/input sync, remote execution, signed output admission, and bounded observability through the same core validation seams used by supported remote-build operation, without ambient network services or hidden global state, and MUST emit the result as a versioned, machine-readable evidence record with a stable schema version and stable field names for `rail_version`, `fixture_id`, `mode`, ordered composition phases, `upload_summary`, `trust_basis`, `artifact_attestation_ref`, `log_status_bounds`, `redaction`, and `non_claims`. The record MUST be byte-stable across repeated runs on the same fixture, with any inherently non-deterministic field documented and scrubbed or omitted, and MUST omit bearer tickets, private key paths, raw environment values, uploaded content, and unbounded logs, argv, or path lists.

#### Scenario: completed fixture proves composition and emits the mandated field set

GIVEN the operator remote-build e2e rail runs a bounded local multi-process fixture through route planning, framed handshake, source/input sync, remote execution, signed output admission, and bounded observability
WHEN Mantle writes the rail evidence record
THEN the record MUST carry the stable schema version and the mandated field set naming each composition phase and the core seam used
AND `non_claims` MUST state that the rail proves fixture composition only, not production P2P deployment, release reproducibility, or package-manager compatibility.

#### Scenario: cross-seam failures fail closed with stable reason codes

GIVEN the rail fixture lacks output trust, emits unframed stdout, presents stale source state, exceeds upload quota or privacy policy, or requests fallback without an explicit policy
WHEN Mantle runs the corresponding negative case
THEN Mantle MUST reject the remote path before output admission or local-success reporting
AND diagnostics MUST identify the phase and a stable reason code without revealing bearer tickets, private key paths, raw environment values, uploaded content, or unbounded logs.

#### Scenario: repeated runs are byte-stable

GIVEN the rail is run repeatedly against the same bounded fixture
WHEN Mantle renders the evidence record
THEN the record MUST be byte-stable modulo documented non-deterministic fields
AND inherently non-deterministic fields MUST be scrubbed or omitted rather than embedded verbatim.

#### Scenario: malformed or oversized evidence is not promoted

GIVEN the rail would emit a bearer ticket, private key path, raw environment value, uploaded content, unbounded log, unbounded argv list, or unbounded path list, or the evidence record is malformed or oversized
WHEN Mantle renders or consumes the record
THEN Mantle MUST omit or redact the disallowed field and diagnose the malformed or oversized record
AND it MUST NOT promote the record to a successful rail claim.
