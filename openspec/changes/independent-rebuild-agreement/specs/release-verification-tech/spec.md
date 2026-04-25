## ADDED Requirements

### Requirement: Canonical independent rebuild agreement report

Crunch MUST define a canonical independent rebuild agreement report that binds one release attestation to the accepted witness attestations, rebuilt output digest sets, witness identities, signer key names, and environment summaries used for agreement.
ID: release.verification.tech.independent.agreement.report

The report identity MUST be the BLAKE3 digest of canonical compact JSON bytes. Witness entries MUST be sorted deterministically and MUST include enough data for a verifier to explain why each witness counted, was skipped, or failed.

#### Scenario: Agreement report digest is stable

- GIVEN the same release attestation and same accepted witness set in different
  filesystem discovery orders
- WHEN the agreement report is serialized
- THEN the canonical bytes are identical
- AND the BLAKE3 report digest is identical

#### Scenario: Skipped witness is explainable

- GIVEN one valid matching witness and one witness with an unknown signing key
- WHEN the agreement report is generated
- THEN the matching witness is counted
- AND the unknown-key witness is listed as skipped with a signature-trust reason

### Requirement: Verifier exposes independent agreement class

Crunch MUST expose a technical or final verification class that is distinct from a raw matching witness when the configured independent rebuild agreement threshold is satisfied.
ID: release.verification.tech.independent.agreement.class

The verifier MUST keep digest matching, signature validity, independence, and policy sufficiency separately visible in JSON output. A release MUST NOT be reported as independently agreed when matching witnesses all come from the same configured independence domain.

#### Scenario: Independent witnesses satisfy agreement

- GIVEN a release with valid evidence
- AND two matching witnesses from distinct configured independence domains
- AND policy requiring two independent matching witnesses
- WHEN release verification runs
- THEN the output reports independent rebuild agreement satisfied
- AND it names the agreement report digest

#### Scenario: Same-domain witnesses do not satisfy agreement

- GIVEN a release with two matching witness attestations from the same
  configured independence domain
- WHEN policy requires two independent domains
- THEN verification keeps technical digest matching visible
- AND it reports independent rebuild agreement unsatisfied
