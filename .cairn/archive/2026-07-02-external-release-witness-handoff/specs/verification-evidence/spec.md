## ADDED Requirements

### Requirement: External release witness handoff evidence

r[release_witness.external_handoff] Mantle MUST record durable evidence before describing a release witness result as externally independent rebuild agreement.

#### Scenario: external witness request is public and digest-bound

GIVEN a publisher exports a release witness request for an external operator
WHEN the handoff evidence is recorded
THEN the evidence MUST name the request directory or artifact, its BLAKE3 digest, release id, release attestation digest, and exported public verifier material
AND it MUST state that signing keys and verifier-local private state were not included in the request.

#### Scenario: imported external witness verifies

GIVEN an independently operated witness returns signed witness sidecars for the requested release
WHEN the publisher imports the sidecars and runs release verification
THEN the tracked evidence MUST record the witness identity, trusted public key token or verifier name, signature status, release-digest match, rebuilt artifact digest match, policy-counting status, and final verification class
AND the claim MUST be limited to the verified policy-scoped witness set.

#### Scenario: local provider-bound witness is not external evidence

GIVEN a witness replay was performed by the same operator, same authority, or provider-bound environment as the publisher
WHEN release evidence is summarized
THEN Mantle MUST describe it as local or provider-bound witness evidence
AND it MUST NOT call it external independent rebuild agreement unless separate handoff evidence supports that claim.

#### Scenario: bad witness material fails closed

GIVEN returned witness material is missing a signature, names the wrong release digest, conflicts with an existing identity, uses an unknown key, or is policy-insufficient
WHEN import or release verification runs
THEN Mantle MUST reject or skip the material according to the documented trust model
AND it MUST NOT weaken the final release claim.
