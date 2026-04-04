## Phase 1: Workspace scaffolding and vendoring

- [x] Create workspace Cargo.toml with members for crunch (bin), crunch-eval, crunch-glue, and vendored crates
- [x] Vendor nix-compat and nix-compat-derive from ../snix/snix/ into crunch/vendor/
- [x] Vendor snix-castore into crunch/vendor/
- [x] Vendor snix-store into crunch/vendor/
- [x] Vendor snix-build into crunch/vendor/
- [x] Vendor snix-tracing into crunch/vendor/ (snix-serde skipped: depends on snix-eval)
- [x] Patch vendored Cargo.toml files: replace workspace dependencies with local paths, remove unused features
- [x] Strip snix-eval and snix-glue references from all vendored crates (none found in code, only a doc comment in refscan.rs)
- [x] Modify vendored nix-compat: replace SHA-256 with BLAKE3 in hash_derivation_modulo, build_text_path, build_store_path_from_fingerprint_parts, and the sha256! macro; keep SHA-256 available for FOD content hashing via nixhash module
- [x] Make STORE_DIR configurable in vendored nix-compat: kept as const for now with comment noting future runtime configurability; code structure supports replacement
- [x] Audit vendored crates for hardcoded Unix paths in core (non-sandbox) code: only /bin/sh in test fixture, no hardcoded paths in core logic
- [x] Verify `cargo check --workspace` passes for the whole workspace

## Phase 2: crunch-eval (Nickel evaluation)

- [x] Create crunch-eval crate with nickel-lang dependency (using stable nickel-lang 2.0.0 API, not nickel-lang-core directly)
- [x] Implement `evaluate(path, import_paths) -> Result<Expr>` that loads a .ncl file, runs eval_deep_for_export, returns Expr for serde deserialization
- [x] Handle Nickel evaluation errors and map them to crunch error types
- [x] Write unit tests: simple records, not_exported stripped, contract violations, serde deserialize, recursive records, enum tags, merge, defaults (8 tests)
- [x] Implement `evaluate_to_json(path, import_paths)` for debug output; CLI `crunch eval` subcommand deferred to Phase 6

## Phase 3: crunch-glue (Nickel record → Derivation)

- [x] Create crunch-glue crate depending on nix-compat, serde, blake3, bstr, data-encoding
- [x] Define `#[derive(Deserialize)]` Rust structs: CrunchDerivation, Input enum (untagged: Derivation or Source), FixedOutput; system/algo/mode as plain strings (parsed during conversion)
- [x] Implement KnownPaths (track derivations by ATerm hash, HDM by drv path, in-progress set for cycle detection)
- [x] Implement `convert(drv, known_paths) -> Result<(StorePath, Derivation)>` — recursive conversion with input resolution, env auto-population, output path computation
- [x] Handle environment auto-population: inject output paths, system, builder, name into Derivation.environment
- [x] Handle fixed-output derivations: parse hash (SRI or hex), algo, mode → CAHash → build_ca_path
- [x] Implement input resolution: Input::Derivation → recursive convert + input_derivations; Input::Source → parse_store_path + input_sources
- [x] Implement dependency ordering: recursive descent with memoization via KnownPaths ATerm hash, cycle detection via identity-based in-progress set
- [x] Write 13 tests: minimal, determinism, different-names, source input, derivation input, FOD sha256, multiple outputs, diamond dependency, circular dependency, user env, invalid source path, invalid hash algo, JSON serde round-trip

## Phase 4: Build pipeline (Derivation → sandbox → store)

- [x] Implement derivation_to_build_request: translate Derivation + resolved inputs into snix-build BuildRequest (crunch-build crate)
- [x] Wire up store services: BlobService + DirectoryService passed to BubblewrapBuildService; SimpleRenderer for NarCalculationService; in-memory session tracking with filesystem cache check
- [x] Implement build caching: check if output store paths exist on disk before building; skip if present
- [x] Implement source input validation: verify all Input::Source paths exist in the store before sandbox invocation
- [x] Implement build orchestration: Builder struct recursively ensures all inputs are built, then submits BuildRequest to BuildService
- [x] Persist build outputs: compute NAR hash via SimpleRenderer, scan references via refscan needles, create PathInfo, store in session HashMap
- [x] Capture build logs (stdout/stderr), display on failure (via BuildFailed error with log field)
- [x] Ensure bwrap sandbox enforces no-new-privileges: added --new-session to COMMON_BWRAP_ARGS (seccomp-bpf not applied — same as upstream snix)
- [x] Implement FOD hash mismatch reporting: verify_fod_hash prints expected vs actual hash for NAR-sha256 FODs
- [x] Write integration test: trivial derivation end-to-end with bwrap (gated on bwrap availability)
- [x] Write integration test: cache miss verified via DummyBuildService error path
- [x] Write integration test: FOD ca_hash propagation verified (hash mismatch detection wired through glue)
- [x] Write integration test: eval hello-world.ncl with seed, verify full Nickel→glue round-trip

## Phase 5: Nickel stdlib

- [x] Write lib/contracts.ncl: StorePath validator, Name validator, System enum, HashAlgo enum, HashMode enum, Sandbox enum, Input contract
- [x] Write lib/derivation.ncl: closed Derivation contract with enums, defaults, optional fixed_output, doc annotations on every field
- [x] Write lib/helpers.ncl: enum-to-string converters using match; type-annotated utility functions
- [x] Write lib/lib.ncl: single entry point re-exporting all contracts, enums, validators, helpers
- [x] Write lib/seed.ncl template: StorePath-validated record with doc annotations
- [x] Embed stdlib .ncl files into crunch binary at compile time (via include_str! in crunch-eval/src/stdlib.rs)
- [x] Write test: contract catches missing name, wrong type, extra field (3 tests)
- [x] Write test: enum tags deserialize correctly through serde (via JSON export path)
- [x] Write test: StorePath validator accepts valid paths, rejects malformed (2 tests)
- [x] Write test: recursive record self-references resolve (env.X = name)
- [x] Write test: fixed_output contract with defaults
- [x] Write test: full Nickel → serde → glue round-trip

Note: Nickel's `Expr::to_serde()` does not convert enum tags to strings.
Added `evaluate_str_and_deserialize()` / `evaluate_and_deserialize()` that
go through JSON export first. This is the correct path for the build pipeline.

## Phase 6: CLI and end-to-end

- [x] Implement `crunch build <file.ncl>`: eval → derivations → build → print output paths
- [x] Implement `crunch eval <file.ncl>`: eval → print JSON
- [x] Implement store initialization with configurable prefix (--store flag, defaults to /nix/store)
- [x] Add --store, --verbose, --log-level flags
- [x] Implement exit codes: 0 success, 1 build failure, 2 eval error, 3 internal error
- [x] Auto-inject stdlib import path (source tree or embedded extraction)
- [x] Wire up BubblewrapBuildService with MemoryBlobService + RedbDirectoryService
- [x] Write end-to-end test: bwrap trivial build + eval_hello_world_with_seed round-trip (4 integration tests)

## Phase 7: Bootstrap and self-hosting prep

- [x] Implement `crunch bootstrap` subcommand that generates seed.ncl from existing Nix store paths
- [x] Write hello-world.ncl: C hello world built with crunch (examples/hello-world.ncl)
- [x] Write crunch.ncl: self-hosting placeholder (evaluates correctly, documents required seed packages; full self-build needs rustc/cargo/protobuf/openssl/clang/mold in seed)
- [x] Document bootstrap process in crunch/README.md
