## Context

`convert()` takes a `CrunchDerivation` and returns a `(StorePath, Derivation)`
with BLAKE3-computed paths. It depends on `nix_compat::Derivation` methods
(`hash_derivation_modulo`, `calculate_output_paths`, `calculate_derivation_path`)
which are deterministic given the same inputs. This means we can construct
known inputs and assert exact output paths.

`KnownPaths` is a plain HashMap wrapper with cycle detection state. Testable
in isolation without any IO.

## Goals / Non-Goals

**Goals:** Cover the conversion logic's correctness properties — path
computation, input wiring, cycle detection, diamond dedup, FOD propagation.

**Non-Goals:** Don't test nix-compat internals (those have their own 1500+
tests). Don't test serde deserialization (already covered in tests.rs).

## Decisions

### 1. Test strategy: known-good path assertions

**Choice:** Build `CrunchDerivation` structs in Rust, run `convert()`,
assert specific store path strings.

**Rationale:** Store paths are deterministic. Once we compute a correct
path, we pin it as a test expectation. Any regression in the hashing or
ATerm serialization breaks the assertion.

**Alternative:** Property-based tests that only check structural invariants
(path starts with /nix/store, correct name suffix). Rejected — too weak,
wouldn't catch hash computation bugs.

### 2. Test location: inline `#[cfg(test)]` modules

**Choice:** Add `mod tests` to `convert.rs` and `known_paths.rs`.

**Rationale:** These functions are private to the crate. Inline test modules
can access them directly without making internals `pub`.

## Risks / Trade-offs

**[Brittle path assertions]** → If nix-compat changes its ATerm format or
hash computation, pinned paths break. Acceptable — we'd want to know.
