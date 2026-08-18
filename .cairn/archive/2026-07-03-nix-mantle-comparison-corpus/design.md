## Context

Nix comparison is only meaningful when the compared builds use equivalent sources, toolchains, inputs, and output normalization. Mantle should compare object or NAR bytes, not store path strings, and should treat intentional semantic differences as blockers.

## Decisions

### 1. Corpus entries declare equivalence policy

**Choice:** Each corpus case names source refs, toolchain refs, build command/derivation identity, expected output surfaces, normalization policy, and allowed differences.

**Rationale:** Without a declared equivalence policy, a digest mismatch could reflect different inputs rather than a reproducibility failure.

### 2. Compare bytes, not path names

**Choice:** Reports compare BLAKE3 object digests and NAR/content digests. Store prefixes and derivation hashes are recorded but not used as equality proof.

**Rationale:** Mantle and Nix intentionally differ in hashing and default prefixes, so output bytes are the relevant comparison surface.

### 3. Unsupported cases become evidence debt

**Choice:** Unsupported Nix or Mantle features produce deterministic blockers with next-action fields.

**Rationale:** The corpus should grow over time without converting missing support into false failures or false successes.

## Risks / Trade-offs

- True equivalence can be hard for packages with timestamps, debug paths, or toolchain differences.
- Running Nix and Mantle side by side may require host-specific prerequisites; reports must record those prerequisites.
