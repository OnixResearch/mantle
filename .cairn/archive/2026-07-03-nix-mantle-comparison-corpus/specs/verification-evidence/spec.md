## ADDED Requirements

### Requirement: Nix Mantle comparison corpus

r[verification_evidence.nix_mantle_comparison_corpus] Mantle MUST compare Nix and Mantle builds only through an explicit corpus that binds equivalence policy, input provenance, output surfaces, and byte-level digest evidence.

#### Scenario: corpus cases declare equivalence

GIVEN a corpus case compares a Nix build and a Mantle build
WHEN Mantle evaluates the case
THEN the case MUST bind source refs, toolchain refs, dependency refs, build recipe identity, output surfaces, normalization policy, and allowed differences
AND cases lacking equivalent inputs MUST be reported as blocked rather than matched or mismatched.

#### Scenario: comparison uses content digests

GIVEN a corpus case has produced Nix and Mantle outputs
WHEN Mantle compares them
THEN the report MUST compare BLAKE3 object digests, NAR/content digests, or another declared byte-level digest surface
AND store path strings, derivation hashes, and logical prefixes MUST NOT be treated as output equality proof.

#### Scenario: unsupported cases do not overclaim

GIVEN Nix or Mantle cannot build a corpus case because a feature is unsupported, non-equivalent, or missing required evidence
WHEN the corpus report is emitted
THEN the report MUST preserve an unsupported or blocker class with next action
AND the summary MUST NOT treat the blocked case as evidence that either system is more reproducible.
