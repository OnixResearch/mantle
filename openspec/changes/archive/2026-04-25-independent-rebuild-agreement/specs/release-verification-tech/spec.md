## ADDED Requirements

### Requirement: Witness independence evidence is structured

Crunch MUST derive witness independence evidence from signed witness attestations and detached signature verification results before any witness can count toward independent rebuild agreement.
ID: release.verification.tech.witness.independence.evidence

A countable witness MUST have a trusted signer key name, a witness identity, a rebuild environment summary with host class, rebuilt output digests, and a selected independence-domain value for the active policy selector. Missing, malformed, untrusted, or selector-empty evidence MUST keep the witness out of the counted set and MUST appear in verifier output as skipped or failed with a reason.

#### Scenario: Missing independence field is skipped

- GIVEN a witness whose signature is trusted and rebuilt digests match
- BUT the active independence selector resolves to an empty field
- WHEN agreement verification runs
- THEN the witness does not count toward independent agreement
- AND the report lists it as skipped with a missing-independence-evidence reason

#### Scenario: Malformed environment evidence is rejected

- GIVEN a witness sidecar with malformed rebuild environment summary data
- WHEN agreement verification loads witness material
- THEN verification does not count that witness
- AND the diagnostic names the malformed environment evidence

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

Crunch MUST expose JSON field `independent_agreement_status` and class value `independent-rebuild-agreement` when the configured independent rebuild agreement threshold is satisfied.
ID: release.verification.tech.independent.agreement.class

The verifier MUST keep digest matching, signature validity, independence, and policy sufficiency separately visible in JSON output. `independent_agreement_status` MUST be `satisfied` only when the configured policy threshold is met across distinct selected independence domains; otherwise it MUST be `unsatisfied` with diagnostics. The JSON output MUST also include `independent_agreement_class`, `independent_agreement_report_digest`, `independent_agreement_counted_witness_count`, `independent_agreement_skipped_witness_count`, `independent_agreement_failed_witness_count`, and per-witness classification reasons. The class value `independent-rebuild-agreement` MUST NOT replace or obscure existing technical/final classes such as `external-witness-match`; it is an additional agreement class reported beside them. A release MUST NOT be reported as independently agreed when matching witnesses all come from the same configured independence domain.

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

### Requirement: Operator docs bound independent agreement claims

Crunch MUST document independent rebuild agreement as policy-scoped evidence from accepted witness files, not as proof of full-source bootstrap, global reproducibility, or public witness discovery.
ID: release.verification.tech.independent.agreement.docs

The docs MUST name the JSON fields `independent_agreement_status`, `independent_agreement_class`, agreement report digest, counted/skipped/failed counts, and witness classification reasons. The docs MUST state that independence metadata is evaluated from signed witness sidecars and verifier-local policy, and stronger identity vetting is outside first-phase file verification.

#### Scenario: Docs avoid overclaiming agreement

- GIVEN operator documentation describes independent rebuild agreement
- WHEN it explains a satisfied agreement
- THEN it ties the claim to configured policy and accepted witness files
- AND it does not claim full-source bootstrap or global reproducibility

## MODIFIED Requirements

### Requirement: Witness attestations MUST bind rebuilt outputs to one release attestation

Crunch MUST define a witness-attestation format whose signed payload names one release-attestation digest and records the witness rebuilt output digest set. The format MUST carry an explicit versioned signature-suite identifier and a signature encoding that binds signer identity to the canonical witness attestation digest. Release reference and rebuilt-output mismatches remain fatal for a witness that is otherwise signature-valid, while missing, unknown, or cryptographically invalid witness signatures are classified and skipped so other trusted witnesses can still satisfy policy.

#### Scenario: Witness with wrong release reference is rejected

- GIVEN a witness attestation that names a different release-attestation digest
- WHEN verification runs
- THEN the command exits non-zero
- AND it identifies the mismatched release reference

#### Scenario: Witness with wrong rebuilt output digest is rejected

- GIVEN a witness attestation whose rebuilt output digest set differs from the
  published release digest set
- WHEN verification runs
- THEN the command exits non-zero
- AND it identifies the mismatched rebuilt output digest

#### Scenario: Witness with invalid signature is classified and skipped

- GIVEN a witness attestation whose detached signature is missing, unknown, or
  cryptographically invalid
- AND at least one other trusted witness can still be evaluated
- WHEN verification runs
- THEN the command keeps evaluating trusted witness material
- AND the invalid-signature witness appears in independent agreement output as
  skipped or failed with a signature reason
