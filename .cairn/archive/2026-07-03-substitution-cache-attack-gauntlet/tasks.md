## Implementation

- [x] [serial] I1 Define deterministic substitution attack fixtures and pure verdict mapping for trusted, unsigned, wrong-key, digest-mismatched, stale, incomplete, corrupted, and authority-confused cache cases. r[verification_evidence.substitution_cache_attack_gauntlet]
- [x] [serial] I2 Implement local directory/HTTP cache fixtures and report plumbing that records trust edge, expected digest, observed digest, fallback mode, and blocker class. r[verification_evidence.substitution_cache_attack_gauntlet]
- [x] [serial] I3 Ensure strict release/global reproducibility evidence treats invalid substitutes and degraded fallback as blockers. r[verification_evidence.substitution_cache_attack_gauntlet]

## Verification

- [x] [serial] V1 Positive: accept a correctly signed trusted substitute with matching PathInfo, closure, and content digest evidence. r[verification_evidence.substitution_cache_attack_gauntlet]
- [x] [serial] V2 Negative: reject or block bad signatures, wrong keys, mismatched PathInfo/NAR digests, incomplete closures, stale attestations, and authority/query attacks. r[verification_evidence.substitution_cache_attack_gauntlet]
- [x] [serial] V3 Run focused store/substitution tests, real HTTP fixture tests, report checks, and Cairn validate/gates for this change. r[verification_evidence.substitution_cache_attack_gauntlet]
