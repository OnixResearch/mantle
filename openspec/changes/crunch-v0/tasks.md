## Phase 1: Workspace scaffolding and vendoring

- [ ] Create workspace Cargo.toml with members for crunch (bin), crunch-eval, crunch-glue, and vendored crates
- [ ] Vendor nix-compat and nix-compat-derive from ../snix/snix/ into crunch/vendor/
- [ ] Vendor snix-castore into crunch/vendor/
- [ ] Vendor snix-store into crunch/vendor/
- [ ] Vendor snix-build into crunch/vendor/
- [ ] Vendor snix-serde and snix-tracing into crunch/vendor/
- [ ] Patch vendored Cargo.toml files: replace workspace dependencies with local paths, remove unused features
- [ ] Strip snix-eval and snix-glue references from all vendored crates
- [ ] Modify vendored nix-compat: replace SHA-256 with BLAKE3 in hash_derivation_modulo, build_output_path, build_text_path, build_store_path_from_fingerprint; keep SHA-256 for FOD content hashing
- [ ] Make STORE_DIR configurable in vendored nix-compat: replace hardcoded `/nix/store` constant with a runtime-configurable value threaded through store path computation
- [ ] Audit vendored crates for hardcoded Unix paths in core (non-sandbox) code; move platform assumptions to sandbox implementations
- [ ] Verify `cargo check` passes for the whole workspace

## Phase 2: crunch-eval (Nickel evaluation)

- [ ] Create crunch-eval crate with nickel-lang-core dependency
- [ ] Implement `evaluate(path: &Path) -> Result<NickelValue>` that loads a .ncl file, runs eval_full_for_export, returns NickelValue directly
- [ ] Handle Nickel evaluation errors and map them to crunch error types
- [ ] Write unit tests: evaluate simple records, catch contract violations, handle parse errors
- [ ] Implement `crunch eval <file.ncl>` subcommand that exports to JSON for debug output

## Phase 3: crunch-glue (Nickel record → Derivation)

- [ ] Create crunch-glue crate depending on nix-compat and nickel-lang-core (for NickelValue type)
- [ ] Define `#[derive(serde::Deserialize)]` Rust structs: CrunchDerivation, Input enum (untagged: Derivation or Source), System enum, FixedOutput, HashAlgo enum, HashMode enum
- [ ] Implement KnownPaths (track derivations and their output paths, compute hash_derivation_modulo)
- [ ] Implement `value_to_derivation(value: NickelValue, known_paths: &mut KnownPaths) -> Result<(StorePath, Derivation)>` — deserialize via serde, convert to nix_compat::Derivation
- [ ] Handle environment auto-population: inject output paths and system into Derivation.environment
- [ ] Handle fixed-output derivations: parse fixed_output → CAHash → build_ca_path
- [ ] Implement input resolution: Input::Derivation → recursive convert + input_derivations; Input::Source → validate + input_sources
- [ ] Implement dependency ordering: recursive descent with memoization via KnownPaths, cycle detection via in-progress set
- [ ] Write store path determinism tests: same inputs → same path; paths differ from Nix (BLAKE3 vs SHA-256)
- [ ] Write unit tests for error cases: missing fields, invalid hashes, circular inputs

## Phase 4: Build pipeline (Derivation → sandbox → store)

- [ ] Implement derivation_to_build_request: translate Derivation + resolved inputs into snix-build BuildRequest
- [ ] Wire up store services: BlobService + DirectoryService + PathInfoService + NarCalculationService backed by local store
- [ ] Implement build caching: check PathInfoService for existing output before building; skip if present
- [ ] Implement source input validation: verify all Input::Source paths exist in the store before sandbox invocation
- [ ] Implement build orchestration: recursively ensure all inputs are built, then submit BuildRequest to BuildService
- [ ] Persist build outputs: compute NAR hash, scan references, create PathInfo, insert into PathInfoService
- [ ] Capture build logs (stdout/stderr), display on failure
- [ ] Ensure bwrap sandbox enforces seccomp-bpf + no-new-privileges unconditionally
- [ ] Implement FOD hash mismatch reporting: print expected vs actual hash and .ncl file location to update
- [ ] Write integration test: build a trivial derivation end-to-end, verify store path and contents
- [ ] Write integration test: rebuild same derivation, verify cache hit (no rebuild)
- [ ] Write integration test: FOD with wrong hash, verify mismatch error includes correct hash

## Phase 5: Nickel stdlib

- [ ] Write lib/contracts.ncl: StorePath validator, Name validator, System enum, HashAlgo enum, HashMode enum, Sandbox enum, Input contract
- [ ] Write lib/derivation.ncl: closed Derivation contract with enums, defaults, optional fixed_output, doc annotations on every field
- [ ] Write lib/helpers.ncl: enum-to-string converters using match; type-annotated utility functions
- [ ] Write lib/lib.ncl: single entry point re-exporting all contracts, enums, validators, helpers
- [ ] Write lib/seed.ncl template: StorePath-validated record with doc annotations
- [ ] Embed stdlib .ncl files into crunch binary at compile time
- [ ] Write test: contract catches missing name, wrong type, extra field
- [ ] Write test: enum tags deserialize correctly through serde
- [ ] Write test: StorePath validator accepts valid paths, rejects malformed
- [ ] Write test: recursive record self-references resolve (env.X = name)

## Phase 6: CLI and end-to-end

- [ ] Implement `crunch build <file.ncl>`: eval → derivations → build → print output paths
- [ ] Implement store initialization with configurable prefix
- [ ] Add --store, --verbose, --log-level flags
- [ ] Implement exit codes: 0 success, 1 build failure, 2 eval error, 3 internal error
- [ ] Write end-to-end test: build a hello-world C program using seed toolchain, verify binary runs

## Phase 7: Bootstrap and self-hosting prep

- [ ] Implement `crunch bootstrap` subcommand that generates seed.ncl from existing Nix store paths
- [ ] Write hello-world.ncl: C hello world built with crunch
- [ ] Write crunch.ncl: crunch building itself (needs rustc in seed)
- [ ] Document bootstrap process in crunch/README.md
