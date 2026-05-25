## Context

The accepted Rust package-planning spec already requires explicit source closure ownership and rejects ambient Cargo caches as build inputs. Recent native-planning work made local/path package facts executable through native unit graphs, host artifacts, unified topology execution, and output reuse. The remaining practical gap is registry/vendored source material: many real Rust dependency graphs rely on registry package coordinates and Cargo.lock checksum identity, but Mantle should not ask Cargo's cache layout to be an undeclared source of truth.

This package defines the next bounded seam only. It does not implement a full Cargo resolver, network fetcher, remote cache, feature unification engine, or broad registry compatibility story.

## Decisions

### 1. Treat registry source material as declared local/vendor inputs

**Choice:** The first implementation will accept only declared local/vendor source roots for registry packages, keyed by lockfile package identity and checksum material. It will compute deterministic BLAKE3 tree digests over those source roots and record the vendor/source root as explicit receipt material.

**Rationale:** This keeps the seam offline and reviewable. It proves Mantle-owned source identity without introducing network fetching, credentials, Cargo registry cache discovery, or mirror policy in the same slice.

### 2. Keep Cargo as an oracle comparison, not an execution dependency

**Choice:** Cargo metadata/unit-graph material may be retained to compare supported native source facts, but ready native registry source facts must be computed from Mantle-owned lockfile/vendor inputs rather than copied from Cargo cache paths.

**Rationale:** The existing Rust-planning direction keeps Cargo as evidence while replacing hidden Cargo planning/execution. Registry source planning should follow that pattern and fail closed on mismatches instead of silently falling back.

### 3. Fail closed before native unit/topology claims

**Choice:** Missing checksums, missing or unreadable vendor roots, source digest mismatches, unsupported registry/git/source layouts, and native-vs-oracle divergence will mark the native registry source fragment not ready and prevent downstream Cargo-free unit/topology claims for affected packages.

**Rationale:** A deterministic blocker is safer than a partial native claim that accidentally reads from `$CARGO_HOME`, `target/`, or a developer's git checkout.

### 4. Bound implementation to source-planning facts first

**Choice:** This package stops at explicit source facts feeding the native planning boundary. It may wire ready facts into the existing local/path unit graph only where the source material is fully declared, but it will not add a general dependency scheduler, network registry fetch, version solving, feature expansion, or crates.io mirror policy.

**Rationale:** Source identity must be durable before execution/scheduling can safely grow. A narrow source-planning receipt gives the next implementation slice a clear verification boundary.

## Risks / Trade-offs

- Vendored registry layouts vary; the first slice should support one deterministic checked-in vendor shape and emit precise unsupported-layout blockers for the rest.
- Cargo lockfile/package-source formatting can be subtle; retaining Cargo oracle comparison reduces drift risk while keeping Mantle-owned facts authoritative.
- The implementation may expose existing fixture assumptions that only path dependencies are supported; update negative tests to assert explicit blockers rather than broad failures.
