## ADDED Requirements

### Requirement: Nario v2 can prepare exact foreign source requirements

r[foreign_derivation_import.nario_v2_source_preparation] Mantle MUST allow a verified Nario v2 record to satisfy an executable plan source requirement only when the record names the exact foreign source path. Preparation MUST verify original PathInfo and NAR facts, canonicalize the payload, and create the plan's target source identity.

#### Scenario: Matching source record is projected

GIVEN an admitted executable plan names one non-derivation foreign source path
AND a verified Nario record supplies that exact path with supported payload and metadata
WHEN Mantle prepares the source bundle
THEN it MUST create the mapped target source record from the verified NAR payload
AND the source evidence MUST bind the archive, original path, target path, plan, requirement, and content identities.

#### Scenario: Original signature does not become target authority

GIVEN a matching Nario record has a valid signature over its original Nix PathInfo
WHEN Mantle projects its payload to a recomputed target source path
THEN the signature MAY remain recorded as original-path provenance
AND Mantle MUST apply its own source and target PathInfo admission policy before using the target source.

#### Scenario: Unmatched or built output is rejected

GIVEN a Nario record does not match an exact plan source requirement or represents an arbitrary built package output
WHEN foreign source preparation considers the record
THEN it MUST reject the record for source projection
AND it MUST NOT relabel the record, add an undeclared source, or bypass normal graph rebuilding.

#### Scenario: Unsafe source payload fails closed

GIVEN a matched NAR payload has an identity mismatch, unsupported file kind, unsafe path, unsafe link, wrong mode, or canonicalization failure
WHEN source preparation decodes it
THEN preparation MUST fail before target source admission
AND no dependent builder MAY start from that record.

### Requirement: Nario v2 evidence keeps explicit non-claims

r[foreign_derivation_import.nario_v2_non_claims] Nario v2 list, store import, and source preparation receipts MUST state that Nario transports store data only. They MUST NOT claim derivation graph completeness, package selection meaning, Nix source translation, evaluator parity, package correctness, reproducibility, or release eligibility.

#### Scenario: Converter uses Nario source data

GIVEN a Mantlepkgs generation uses Nario records for every matched foreign source requirement
WHEN Mantle reports source readiness
THEN it MAY claim that those exact source payloads were verified and prepared
AND package recipes, graph admission, rebuilding, output trust, and broader correctness MUST require separate evidence.
