# Foreign Derivation Import Specification

## Purpose

Defines operator documentation requirements for the foreign derivation import trust model.

## Requirements

### Requirement: Operator trust model documentation

r[foreign_derivation_import.operator_trust_model_docs] Mantle documentation MUST explain the trust boundaries for foreign derivation import receipts, including graph provenance, policy digests, source verification, cache/substitution trust, sandbox capabilities, realization, and output verification.

#### Scenario: Operator can identify trust boundaries

GIVEN an operator reads the foreign import trust-model guide
WHEN they review a foreign import receipt
THEN the guide MUST explain which facts the receipt binds and which trust decisions remain separate
AND it MUST distinguish import admission from build success and output trust.

#### Scenario: Guide links from proof documentation

GIVEN operator proof documentation mentions foreign import evidence
WHEN a reader follows related documentation
THEN the trust-model guide MUST be reachable from the proof guide or README
AND the linked text MUST preserve Mantle naming while allowing exact schema/command identifiers.

### Requirement: Receipt non-claims are documented

r[foreign_derivation_import.receipt_non_claims_documentation] Foreign import documentation MUST state that an import receipt alone does not claim build success, package correctness, bootstrap parity, output trust, reproducibility, or foreign-frontend availability.

#### Scenario: Receipt is not mistaken for proof success

GIVEN a valid foreign import receipt exists for a translated graph
WHEN an operator reads the documentation
THEN the documentation MUST state that additional realization and verification evidence is required before claiming trusted outputs
AND it MUST NOT present receipt existence as proof of correctness.

#### Scenario: Cache hints are not output trust

GIVEN a receipt records cache or substitution metadata
WHEN the documentation explains cache reuse
THEN it MUST state that cache hints remain subject to store/substitution trust policy
AND they MUST NOT bypass output admission or signature verification.

### Requirement: Trust model documentation guard

r[foreign_derivation_import.trust_model_guard] Mantle SHOULD include a lightweight guard that fails when required foreign import trust-model headings or non-claim language disappear from operator documentation.

#### Scenario: Guard accepts complete docs

GIVEN the trust-model guide includes required trust-boundary and non-claim sections
WHEN the guard runs
THEN it MUST pass and report the checked sections.

#### Scenario: Guard rejects missing non-claims

GIVEN the trust-model guide omits receipt non-claim language
WHEN the guard runs in negative or self-test mode
THEN it MUST fail with a deterministic diagnostic
AND it MUST identify the missing section or phrase class.
