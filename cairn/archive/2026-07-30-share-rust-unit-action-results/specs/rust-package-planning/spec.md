## ADDED Requirements

### Requirement: Shared Rust unit cache receipts

r[rust_package_planning.unit_execution.topology.shared_cache_receipts] Mantle MUST emit deterministic Rust topology evidence for every attempted shared unit-result lookup, admission, transfer, restoration, publication, and fallback.

#### Scenario: Remote hit records bounded route evidence

GIVEN a supported Rust unit is restored from an admitted shared result
WHEN Mantle emits the unit and topology receipts
THEN the receipts MUST bind the canonical action and result references
AND they MUST identify a sanitized source, accepted verifier, authority disposition, remote-hit reason, transferred bytes, reused bytes, and zero compiler invocations
AND they MUST bind the current restored artifact digests.

#### Scenario: Rejected candidate explains compiler fallback

GIVEN Mantle rejects one or more shared Rust unit result candidates
WHEN explicit execution policy permits local compiler execution
THEN the final receipt MUST preserve bounded rejection reasons and the compiler-executed disposition
AND it MUST NOT claim that a rejected candidate supplied the output.

#### Scenario: Shared reuse preserves bounded non-claims

GIVEN Mantle emits successful shared Rust unit reuse evidence
WHEN a consumer reviews its claim class
THEN the receipt MUST limit the claim to admitted reuse for the recorded action, policy, authority, content, platform, and materialization facts
AND it MUST NOT claim compiler correctness, full Cargo compatibility, universal reproducibility, remote executor correctness, or release eligibility.
