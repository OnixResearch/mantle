# Tasks

## Phase 1: Review boundary and baseline

- [x] [serial] I1 Add ADR 0054 for behavior-selected Snix backports, retained Mantle compatibility boundaries, and explicit upstream dispositions. r[vendored_snix.selective_backport_policy]
  - Evidence: `adr/0054-select-snix-backports-by-mantle-compatibility-boundary.md` and `adr/README.md`.
- [x] [serial] I2 Record the reviewed Gerrit changes, upstream status, local impact, selected work, deferrals, rejection reasons, and reopen triggers. r[vendored_snix.selective_backport_policy]
  - Evidence: `cairn/changes/backport-snix-correctness-fixes/evidence/upstream-review.md`.
- [x] [serial] V1 Before implementation, run the focused baseline commands below and record exact output in `evidence/baseline-tests.md`. r[vendored_snix.selective_backport_policy]
  - Evidence: `evidence/baseline-tests.md` records 86 `snix-store`, 252 `snix-castore`, 0 `snix-tracing`, and 338 `crunch-store` library tests passing.
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-store --lib`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-castore --lib`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-tracing --lib`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib`

## Phase 2: Remote cache identity and transport

- [x] [serial] I3 Add a pure requested-digest comparator and enforce it in the Snix HTTP PathInfo service before a mismatched response can return to its caller. r[cache_substitution.requested_path_identity]
  - Evidence: `evidence/requested-digest-service-guard-2026-08-08.md` records the pure comparator, pre-NAR guards, positive and negative tests, and focused validation.
- [x] [serial] I4 Enforce the same decision in Mantle’s remote-substitution finalization shell before PathInfo persistence, sidecars, castore registration, export, root registration, advisory publication, or success reporting. r[cache_substitution.requested_path_identity]
  - Evidence: `evidence/requested-digest-shell-guard-2026-08-08.md` records the pure first-party guard, stable error, side-effect ordering, and focused validation.
- [x] [parallel] I5 Add positive and negative service tests for a matching signed narinfo, a valid signed narinfo for another path, malformed metadata, and zero returned PathInfo on mismatch. r[cache_substitution.requested_path_identity]
  - Evidence: `evidence/remote-transport-2026-08-08.md` records the local HTTP fixtures and passing Snix service tests.
- [x] [parallel] I6 Add a mutation-counting substitution test that proves an alternative service cannot cause writes or side effects with mismatched PathInfo. r[cache_substitution.requested_path_identity]
  - Evidence: `evidence/requested-digest-shell-guard-2026-08-08.md` records zero local or remote writes, sidecars, exports, roots, advisory entries, and success reports.
- [x] [serial] I7 Normalize binary-cache base URLs as directory bases before endpoint joins while preserving existing credential and query handling. r[cache_substitution.transport_normalization]
  - Evidence: `evidence/remote-transport-2026-08-08.md` records directory-base normalization and URL fixtures.
- [x] [serial] I8 Enable multi-member zstd decoding in the vendored Nix HTTP path and Mantle’s `crunch-store` pull path. r[cache_substitution.transport_normalization]
  - Evidence: `evidence/remote-transport-2026-08-08.md` records both multi-frame decoder paths and bounded rejection behavior.
- [x] [parallel] I9 Add positive and negative URL and zstd fixtures for base paths with and without a trailing slash, concatenated frames, a truncated later frame, malformed input, and bounded rejection without partial admission. r[cache_substitution.transport_normalization]
  - Evidence: `evidence/remote-transport-2026-08-08.md` records every required URL and zstd case and zero local `PathInfo` after rejection.

## Phase 3: Castore and FUSE correctness

- [x] [serial] I10 Repair the pure castore directory-size calculation so each entry and node contribution is counted once with bounded arithmetic. r[vendored_snix.castore_metadata]
  - Evidence: `evidence/castore-fuse-2026-08-08.md` records the pure checked sum and regression tests.
- [x] [serial] I11 Map castore node kinds to FUSE `DT_*` values for `readdir` and set the adopted valid nonzero `nlink` attributes. r[vendored_snix.castore_metadata]
  - Evidence: `evidence/castore-fuse-2026-08-08.md` records the FUSE entry and attribute changes.
- [x] [parallel] I12 Add positive and negative tests for empty and mixed directories, directory-size regression values, every supported FUSE node kind, and guards against `S_IF*` entry types or zero link counts. r[vendored_snix.castore_metadata]
  - Evidence: `evidence/castore-fuse-2026-08-08.md` records all required positive and negative cases and passing feature-enabled tests.

## Phase 4: Store services and operational alignment

- [ ] [serial] I13 Move redb write-transaction creation, mutation, and commit into the blocking worker with owned database state. r[vendored_snix.store_service_behavior]
- [ ] [serial] I14 Implement `PathInfoCache::list()` by delegating to the writable near service only. r[vendored_snix.store_service_behavior]
- [ ] [parallel] I15 Add service tests for successful near writes and listing, empty near state, far-only records, a panic-on-list far service, redb error propagation, and concurrent async callers. r[vendored_snix.store_service_behavior]
- [ ] [serial] I16 Adopt the maintained filesystem-ingestion buffer and copy path without restoring the removed fixed buffer constant. r[vendored_snix.operational_alignment]
- [ ] [serial] I17 Apply `EnvFilter` to the combined tracing layers and preserve progress and non-format layer behavior. r[vendored_snix.operational_alignment]
- [ ] [parallel] I18 Add focused ingestion and tracing tests for normal files, empty files, copy errors, enabled events, disabled events, and non-format layer visibility. r[vendored_snix.operational_alignment]
- [ ] [serial] I19 Recheck the virtiofs used-length and read-only redb-builder triggers. If either path is active, backport and test its fix. Otherwise, update the ledger with current evidence and keep the deferral. r[vendored_snix.selective_backport_policy]
- [ ] [parallel] I20 Add implementation and test requirement references. Use first-party adapter references and `tools/tracey_refs.rs` bridge references where Tracey does not scan vendored paths. r[vendored_snix.selective_backport_policy]

## Phase 5: Validation and lifecycle evidence

- [ ] [serial] V2 After implementation, rerun every V1 command and record exact output in `evidence/focused-tests.md`. r[cache_substitution.requested_path_identity] r[cache_substitution.transport_normalization] r[vendored_snix.castore_metadata] r[vendored_snix.store_service_behavior] r[vendored_snix.operational_alignment]
- [ ] [serial] V3 Run the package checks below and record exact output in `evidence/package-checks.md`. r[vendored_snix.selective_backport_policy]
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo check -p snix-store -p snix-castore -p snix-tracing -p crunch-store`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo fmt --check -p snix-store -p snix-castore -p snix-tracing -p crunch-store -v`
- [ ] [serial] V4 Run the lifecycle commands below and record exact output in `evidence/lifecycle-gates.md`. r[vendored_snix.selective_backport_policy]
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal backport-snix-correctness-fixes --root .`
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design backport-snix-correctness-fixes --root .`
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks backport-snix-correctness-fixes --root .`
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .`
- [ ] [serial] V5 Review the final diff against the upstream ledger. Record each selected CL as adapted with test evidence, or retain a justified non-complete disposition. r[vendored_snix.selective_backport_policy]
