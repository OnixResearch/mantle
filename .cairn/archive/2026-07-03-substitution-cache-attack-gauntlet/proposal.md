## Why

Mantle’s substitution model is stronger only if it rejects malicious or stale cache material under pressure. We need a cache attack gauntlet that feeds bad signatures, mismatched PathInfo, incomplete closures, stale attestations, corrupted NARs, malicious narinfo-like metadata, and query/path authority tricks into substitution and proves fail-closed behavior.

## What Changes

- Add deterministic local cache fixtures for trusted substitutes, bad signatures, digest mismatches, incomplete closures, stale attestations, corrupted payloads, and authority/query normalization attacks.
- Emit substitution-attack reports showing accept/reject decisions, fallback mode, trusted key material, expected digest, observed digest, and blocker class.
- Bind cache attack evidence into release/global reproducibility non-claims so cache acceptance is never inferred from path identity alone.
- Preserve practical-vs-strict fallback semantics without weakening strict reproducibility evidence.

## Impact

- **Files**: store/substitution tests, tiny HTTP cache fixtures, report schema, docs, Cairn verification-evidence spec delta.
- **Testing**: positive trusted substitute, negative bad signature, negative mismatched PathInfo/NAR, negative incomplete closure/stale attestation, Cairn validation/gates.
