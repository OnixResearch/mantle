## ADDED Requirements

### Requirement: Substitution cache attack gauntlet

r[verification_evidence.substitution_cache_attack_gauntlet] Mantle MUST prove substitution trust boundaries with deterministic attack fixtures before using cache acceptance as strict reproducibility evidence.

#### Scenario: trusted substitutes are accepted only with matching evidence

GIVEN a cache fixture serves a substitute signed by a trusted key with matching PathInfo, closure facts, content digest, and artifact attestation
WHEN Mantle imports or reuses that substitute
THEN the report MUST record the trusted key material, expected digest set, accepted output identity, and closure evidence
AND the substitute MAY be accepted only within the configured fallback and reproducibility policy.

#### Scenario: malicious cache material fails closed

GIVEN a cache fixture serves unsigned material, wrong-key signatures, mismatched PathInfo, corrupted NAR content, incomplete closures, stale attestations, or authority/query-confused endpoints
WHEN Mantle evaluates the substitute in strict mode
THEN the substitute MUST be rejected or marked as a strict blocker before admissible reuse evidence is emitted
AND the report MUST name the failed trust edge and expected/observed digest information when available.

#### Scenario: practical fallback is not strict cache proof

GIVEN practical mode falls back from an invalid substitute to a local build or degraded closure resolution
WHEN Mantle summarizes substitution evidence
THEN the report MUST keep fallback/degradation events separate from trusted substitute acceptance
AND strict release/global reproducibility admission MUST remain blocked for that cache evidence class.
