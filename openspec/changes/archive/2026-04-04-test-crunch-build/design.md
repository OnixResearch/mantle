## Context

crunch-build has three layers:

1. **Pure translation** — `derivation_to_build_request()`,
   `collect_input_paths()`, `resolve_references()`, `replace_placeholders()`.
   These are deterministic functions that take derivation data and produce
   build requests or path sets. Fully testable in isolation.

2. **FOD verification** — `verify_fod_hash()`, `nar_hash()`, `hash_blob()`.
   Already well-tested (20+ tests).

3. **Orchestration** — `Builder.build()`, `build_derivation_inner()`,
   `all_outputs_exist()`, `load_cached_outputs()`, `ensure_input_nodes()`.
   These mix async I/O (blob service, directory service, build service,
   filesystem) with control flow (dependency order, caching). Need mocks
   or integration setup.

## Goals / Non-Goals

**Goals:** Test layers 1 and 3. Layer 1 is pure and should have exhaustive
tests. Layer 3 needs a mock BuildService to verify orchestration behavior
(dependency order, caching, error propagation).

**Non-Goals:** Don't duplicate the FOD hash tests. Don't test snix_build's
`BubblewrapBuildService` internals — that's a vendored dependency.

## Decisions

### 1. Test pure functions directly with constructed Derivations

**Choice:** Build `nix_compat::Derivation` structs manually, call the pure
functions, assert outputs.

**Rationale:** Same approach as the existing `build_request.rs` tests.
Derivations are deterministic structs with no magic.

### 2. Mock BuildService for orchestration tests

**Choice:** Implement a `MockBuildService` that records calls and returns
canned `BuildResult` values.

**Rationale:** `BuildService` is an async trait. A mock lets us verify
that `Builder.build()` calls `do_build` for each derivation in the right
order, and skips cached outputs.

**Implementation:** The mock stores a `Vec<BuildRequest>` of received
requests and returns pre-configured `BuildResult` values keyed by some
identifier (e.g., the first command_arg).

### 3. Use tempdir for cache-hit tests

**Choice:** Create temporary store directories with pre-existing output
paths to test `all_outputs_exist()` and the cache-hit code path.

**Rationale:** The cache check is `abs.exists()` on the filesystem. We
need real files to exercise it.

## Risks / Trade-offs

**[Mock fidelity]** → The mock won't catch issues in the real bwrap
integration. That's what the integration-tests openspec covers.

**[Visibility changes]** → `replace_placeholders` and `resolve_references`
are module-private. Tests in the same module can access them directly. If
we add a separate test file, they'd need `pub(crate)`.
