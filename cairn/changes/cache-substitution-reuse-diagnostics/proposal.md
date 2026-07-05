## Why

Mantle already has local PathInfo + castore cache hits, ordinary remote substitution, delta substitution, and manual `mantle store push` / `mantle store pull` sharing. The current operator experience still leaves avoidable rebuilds and opaque misses: `--substituters` is documented as comma-separated while the build/store path accepts one remote cache URL, repeated plan/build workflows can re-query the same remote metadata, and cache rejection reasons are not consistently visible as structured data.

Caching should improve hit rate and explain misses without weakening Mantle's trust boundary. Remote metadata may make planning faster, but a final output must still be accepted only after signed PathInfo, store-prefix, content, castore, and attestation checks pass.

## What Changes

- Introduce a `cache-substitution` Cairn capability for ordered substituter sets, advisory remote metadata reuse, structured cache admission diagnostics, and castore completeness checks.
- Treat configured substituters as a bounded ordered candidate set with deterministic tie-breakers rather than a single URL or latency race.
- Add a persistent advisory metadata cache for remote narinfo/reference availability, cache preflight, negative misses, and delta capability probes, keyed by cache identity, trust policy, store prefix, path digest, and metadata class.
- Surface per-output cache admission and miss reasons in `mantle build --plan`, `mantle --json build`, and bounded human diagnostics.
- Require local cache hits to prove complete backing castore content, not just root metadata, before reporting an output as cached.

## Impact

- **Files**: `src/main.rs`, `src/build_plan.rs`, `src/build_report.rs`, `src/realization_routing.rs`, `crates/crunch-store/src/{handle,closure,pull}.rs`, `crates/crunch-build/src/orchestrate.rs`, `crates/crunch-pipeline/src/lib.rs`, README/operator docs, and tests.
- **Testing**: ordered multi-cache positives, malformed/untrusted/empty substituter negatives, metadata-cache hit/miss/expiry tests, structured reason-code snapshots, and partial-castore rejection tests.

## Out of Scope

- Trusting unsigned cache data by default or weakening existing signature verification.
- Selecting remote caches by observed latency unless latency policy is later modeled as an explicit recorded fact.
- Adding a new cache server implementation, object-store backend, or remote build farm protocol.
- Caching arbitrary build-system sub-actions such as individual Rust crates or C translation units.
- Changing delta protocol wire semantics beyond reusing cached capability/probe facts.