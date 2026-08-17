## Context

Mantle’s trust model depends on signed PathInfo, content-addressed castore data, closure resolution, and strict fallback policy. Cache attack fixtures should exercise those boundaries with deterministic malicious inputs.

## Decisions

### 1. Attack fixtures are local and deterministic

**Choice:** Use tiny local cache servers/directories with generated keys and fixed payloads for trusted, corrupted, unsigned, wrong-key, stale, incomplete, and authority-confused cases.

**Rationale:** Local fixtures make negative tests fast, reproducible, and safe.

### 2. Strict mode rejects before sandbox start

**Choice:** In strict reproducibility contexts, invalid substitutes or missing closure facts must block before build or reuse evidence is admitted.

**Rationale:** Fallback builds can be useful operationally, but strict release evidence must not hide cache trust failures.

### 3. Reports name the failed trust edge

**Choice:** Attack reports include expected digest, observed digest when available, key identity, authority, fallback mode, and blocker class.

**Rationale:** Operators need to distinguish bad signatures, content mismatches, missing closure facts, and stale attestations.

## Risks / Trade-offs

- Real HTTP cache behavior can differ from local fixtures; include at least one real HTTP fixture for narinfo/NAR paths.
- Negative fixtures must avoid leaking generated signing keys into trusted default state.
