## Phase 1: Contract and pure decisions

- [x] [serial] I1 Add route `cache-inputs-build-roots-v2` as a plan constant, accepted only with `preserve-cache-paths-v1`, with v1 structural invariants, a route non-claim, and admission treating both cache-input routes as build routes. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: plan unit tests for accepted/rejected selection and FOD refusal.
- [x] [serial] I2 Add builder fingerprint, trust-policy digest, builder-key set construction, and the pure `classify_present_root` with every typed refusal and reuse fact. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: unit tests for each refusal plus the reuse path, and digest stability.
- [x] [serial] I3 Add receipt v2 types, digest, closed-world parse, the v2 builder, and `input_closure_blake3`; keep v1 byte-identical. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: unit tests for v2 digest and parse rejection of unknown/missing fields; v1 digest test unchanged.

## Phase 2: Shell and CLI

- [x] [serial] I4 Add `realize --trusted-builder-public-key` (repeatable, verify-only), add those keys to the trusted set, and thread the builder-key set and trust-policy digest into realization. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: CLI parse test or command-level rejection test for an invalid key; route v2 acceptance in `tests/cache_inputs_build_route.rs`.
- [x] [serial] I5 On route v2, take per-root locks under `<state-dir>/foreign-realization/locks/`, check presence before hydration, classify present roots, skip hydration/build for reused roots, follow the v1 path for absent roots, publish the write-once built-root index, and reject any rebuilt reused root. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: `tests/cache_inputs_build_route.rs` reuse, changed-input, refusal, rotation, concurrency, and v1-regression tests.
- [x] [serial] I6 Add `crunch-store` NAR re-measurement and the `nix-compat` public-key accessor. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: `cargo test -p crunch-store --lib` focused NAR test.
- [x] [serial] I7 Update `docs/foreign-realization-operator-guide.md` and `docs/foreign-derivation-import-trust-model.md` for route v2, builder keys, the index, refusals, and non-claims. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: docs diff in this change.

## Phase 3: Verification

- [x] [serial] V1 Run focused `cargo test -p mantle --test cache_inputs_build_route --test foreign_import_cli`, the touched `--bin mantle` unit modules, and focused `-p crunch-build`/`-p crunch-store` lib tests. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: captured test transcripts under `evidence/`.
- [x] [serial] V2 Run `cargo fmt --check` and strict Clippy for the touched crates, plus `git diff --check`. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: captured command output.
- [x] [serial] V3 Run `cairn validate --root .` and the proposal, design, and tasks gates; append transcripts to `evidence/`. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: `evidence/orchestrator-checks.md` (validate 0 issues; gates PASS).
- [x] [serial] V4 With the checked build of the landed commit, realize a piloted job on route v2 twice from a fresh store: the first run builds and indexes the root, the second reuses it with zero builds and zero NAR downloads and links the first receipt; record Cairn's convergence report for both and the negative controls. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: `evidence/live-wp8/` (Terrane `bounds.clamp`, the job E2 qualified on route v1). The Octet pilot attempt stopped at its first build on one core; that limit moves to the change `add-realize-build-cores`, whose evidence reruns the Octet pilot.

## Follow-ups found by the pilot

- [x] [serial] I8 Index only successful built roots: the pilot's `partial-failure` receipt (`roots = []`) was published to the write-once built-root index, so a later successful build of the same derivation could not index its receipt. A pure function over the finished receipt v2 now indexes nothing unless `status` is `complete` and only `target_derivation`s of root entries whose disposition is `built`, sorted and deduplicated. r[foreign_derivation_import.cache_inputs_build_or_reuse]
  - Evidence: `/home/brittonr/wt/convergence-evidence/build-or-reuse/pilot-v2/README.md` (observation under "Not evidence of, and out of scope"); `foreign_realization_receipt::built_root_index_derivations` unit test for complete, partial-failure, and reused receipts.
