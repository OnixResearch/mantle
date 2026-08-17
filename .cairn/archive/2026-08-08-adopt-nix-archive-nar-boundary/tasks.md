# Tasks

## Phase 1: Source authority and baseline

- [x] [serial] I1 Record ADR 0071, the README reference, and the upstream review for the split NAR boundary. r[store_transports.nix_archive_boundary]
  - Evidence: `adr/0071-adopt-nix-archive-at-the-filesystem-nar-boundary.md`, `README.md`, and `cairn/changes/adopt-nix-archive-nar-boundary/evidence/upstream-review.md`.
- [x] [serial] V1 Before implementation, run the commands below and record exact output in `evidence/baseline-tests.md`. r[store_transports.nix_archive_parity]
  - Evidence: `evidence/baseline-tests.md` records successful pre-change retries after the unavailable locked Git source was mapped to its exact local sibling commit.
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p snix-store --lib nar`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-store --lib`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --test project_refresh_cli`
- [x] [serial] I2 Add exact `nix-archive` version `0.1.0`. Record its Cargo checksum and prove parity with reviewed commit `14362ab589daa4869bda744d4fbe26a1914b5491`. r[store_transports.nix_archive_boundary]
  - Evidence: the exact manifest pin, Cargo lock checksum, packaged `.cargo_vcs_info.json`, fixture manifest, and adapter constants bind the published source.
- [x] [serial] I3 Update `Cargo.lock` only through Cargo. Refresh `vendor-deps/` through the repository-owned dependency process. r[store_transports.nix_archive_boundary]
  - Evidence: Cargo generated the lock entry and task 11040 regenerated the complete locked vendor tree, including `vendor-deps/nix-archive/.cargo-checksum.json`.
- [x] [parallel] I4 Prove `cargo metadata --offline --locked --format-version 1 --config .cargo/vendor-config.toml` with no ambient registry or network access. r[store_transports.nix_archive_boundary]
  - Evidence: task 11106 passed with an empty `CARGO_HOME`, `CARGO_NET_OFFLINE=true`, `GIT_CONFIG_GLOBAL=/dev/null`, and the refreshed vendor config.

## Phase 2: Shared adapter and parity core

- [x] [serial] I5 Add a shared `crunch-nar` adapter with normalized requests, explicit case-hack policy, digest writers, byte counts, and typed observations. r[store_transports.nix_archive_filesystem_observation]
  - Evidence: `crates/crunch-nar/src/lib.rs` provides the bounded filesystem observation and evidence API for every supported digest.
- [x] [serial] I6 Keep filesystem work in a blocking shell. Keep parity comparison and cutover decisions pure and deterministic. r[store_transports.nix_archive_boundary]
  - Evidence: `observe_path_blocking` is the async shell; `compare_nar_facts` and `evaluate_cutover` are pure cores with positive and negative tests.
- [x] [parallel] I7 Add positive tests for empty files, data files, executable files, symlinks, nested directories, byte-sorted names, and non-UTF-8 names. r[store_transports.nix_archive_parity]
  - Evidence: `crates/crunch-nar/tests/parity.rs` covers the complete byte-safe fixture and retained upstream goldens.
- [x] [parallel] I8 Add negative tests for invalid names, unsupported hash modes, explicit case-hack mismatches, concurrent path substitution, read failures, and worker failures. r[store_transports.nix_archive_parity]
  - Evidence: adapter unit and integration tests cover request rejection, case collisions, root substitution, read failure, worker failure, and byte limits.
- [x] [parallel] I9 Add a dependency guard that rejects direct `nix_archive::nar` imports outside the shared adapter and compatibility tests. r[store_transports.nix_archive_boundary]
  - Evidence: `scripts/check-nar-boundary.rs` has positive and negative self-tests and scans first-party Rust sources.

## Phase 3: Differential evidence

- [x] [serial] I10 Retain upstream positive and negative NAR fixtures with source revision, package version, license, and BLAKE3 fixture identities. r[store_transports.nix_archive_parity]
  - Evidence: `crates/crunch-nar/fixtures/upstream/manifest.ncl` binds four positive files and three negative constructions; tests verify each BLAKE3 identity.
- [x] [serial] I11 Compare `nix-archive` and Snix NAR bytes, size, SHA-256, and all Mantle-supported recursive hash algorithms on the shared fixture domain. r[store_transports.nix_archive_parity]
  - Evidence: `nix_archive_and_snix_match_on_byte_safe_fixture_and_all_recursive_hashes` compares exact bytes, size, and five digest algorithms.
- [x] [parallel] I12 Add generated-tree parity with fixed case counts, depth limits, file-size limits, and deterministic seeds. r[store_transports.nix_archive_parity]
  - Evidence: the generated-tree fixture has named case, depth, entry, payload, and seed constants plus an over-limit negative test.
- [x] [parallel] I13 Add `nix-store --dump` differential tests when Nix is available. Report skipped oracle runs as unavailable evidence, not success. r[store_transports.nix_archive_parity]
  - Evidence: `nix_oracle_is_reported_as_matched_or_unavailable` reports the exact disposition and never converts unavailable to matched.
- [x] [serial] I14 Add a cutover gate that rejects any seam with missing cases, mismatched bytes, mismatched facts, stale upstream identity, or unsupported platform evidence. r[store_transports.nix_archive_parity]
  - Evidence: the pure cutover core and `evidence/cutover-decision.ncl` cover accepted and rejected decisions without fallback.

## Phase 4: Production filesystem seams

- [x] [serial] I15 Route physical store NAR verification through the shared adapter without temporary castore ingest and re-render. r[store_transports.nix_archive_filesystem_observation]
  - Evidence: `crates/crunch-store/src/query.rs::store_verify` now observes the physical path directly and compares hash plus NAR size.
- [x] [parallel] I16 Keep node identity and PathInfo trust checks separate from the filesystem NAR observation. Prove zero persistence on mismatch or read failure. r[store_transports.nix_archive_filesystem_observation]
  - Evidence: store unit and CLI tests cover success, tampering, read failure, missing paths, dangling symlinks, and unchanged PathInfo state.
- [x] [serial] I17 Route recursive project source hashing through the shared adapter for supported Nix hash algorithms. Keep flat hashes unchanged. r[project_workflows.nix_archive_recursive_hashing]
  - Evidence: `src/project_resolve.rs::hash_recursive_path` delegates to `crunch-nar`; the existing flat reader remains separate.
- [x] [parallel] I18 Add project tests for each supported recursive algorithm, flat-hash non-regression, non-UTF-8 tree members, symlinks, executable bits, and failed observation without lock mutation. r[project_workflows.nix_archive_recursive_hashing]
  - Evidence: `tests/project_refresh_cli.rs` covers all required tree facts, three recursive algorithms, flat archive inequality, and exact file non-mutation.
- [x] [serial] I19 Record adapter version, upstream identity, case-hack mode, algorithm, NAR size, digest, and observation disposition in existing reports or evidence. r[store_transports.nix_archive_filesystem_observation]
  - Evidence: `FilesystemNarObservation::evidence` emits the schema, source identity, policy, algorithm, size, hex/SRI digest, and success disposition; `evidence/filesystem-nar-observation.ncl` retains one concrete receipt.

## Phase 5: Castore and transport guardrails

- [x] [serial] I20 Keep Snix NAR rendering and ingest for castore, native archives, Nario, HTTP caches, remote builds, Rust cache transfer, repair, and output persistence. r[store_transports.nix_archive_castore_separation]
  - Evidence: production castore paths retain `write_nar`, `SimpleRenderer`, and `ingest_nar_and_hash`; only two filesystem seams changed.
- [x] [parallel] I21 Add compile-time or source guards that reject full-buffer production decode or restore in archive, cache, remote, and Nario paths. r[store_transports.nix_archive_castore_separation]
  - Evidence: the boundary guard rejects direct imports and full-buffer decode/restore symbols outside the adapter, and requires retained streamed Snix APIs.
- [x] [parallel] I22 Add regression tests that show large streamed NAR payloads do not enter a complete in-memory byte slice for this integration. r[store_transports.nix_archive_castore_separation]
  - Evidence: `large_archive_payload_stays_on_the_chunked_castore_ingest_path` feeds a large archive through a bounded non-seekable reader into castore ingest.
- [x] [serial] I23 Document deferred restore requirements: named payload limits, fresh staging, cleanup, no-replace publication, and post-publication verification. r[store_transports.nix_archive_castore_separation]
  - Evidence: `docs/nix-archive-nar-boundary.md` records all five controls and the bounded non-claims.

## Phase 6: Validation and lifecycle evidence

- [x] [serial] V2 After implementation, rerun every V1 command and record exact output in `evidence/focused-tests.md`. r[store_transports.nix_archive_parity] r[project_workflows.nix_archive_recursive_hashing]
  - Evidence: `evidence/focused-tests.md` records 33 Snix NAR tests, 338 store library tests, and 14 project refresh CLI tests passing.
- [x] [serial] V3 Run the commands below and record exact output in `evidence/package-checks.md`. r[store_transports.nix_archive_boundary]
  - Evidence: `evidence/package-checks.md` records all adapter/store tests, project tests, affected package checks, and Rustfmt passing.
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p crunch-nar -p crunch-store`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo test -p mantle --test project_refresh_cli`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo check -p crunch-nar -p crunch-store -p mantle`
  - `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo fmt --check -p crunch-nar -p crunch-store -p mantle -v`
- [x] [serial] V4 Run focused Clippy, dependency guards, fixture identity checks, `git diff --check`, and machine-contract checks. Record exact output in `evidence/quality-checks.md`. r[store_transports.nix_archive_boundary]
  - Evidence: `evidence/quality-checks.md` records focused Clippy, guard self-tests, fixture and CLI tests, empty-home offline metadata, Nickel exports, and whitespace checks passing.
- [x] [serial] V5 Run the lifecycle commands below and record exact output in `evidence/lifecycle-gates.md`. r[store_transports.nix_archive_parity]
  - Evidence: `evidence/lifecycle-gates.md` records validate, proposal, design, tasks, and Tracey coverage passing before archive.
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal adopt-nix-archive-nar-boundary --root .`
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design adopt-nix-archive-nar-boundary --root .`
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks adopt-nix-archive-nar-boundary --root .`
  - `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .`
- [x] [serial] V6 Run the relevant Nix checks, sync the accepted requirements, archive the change, and record post-archive validation. r[store_transports.nix_archive_parity]
  - Evidence: `evidence/nix-checks.md` records the passing focused Nix format check and bounded broad-check blockers. The archive operation syncs the accepted requirements; post-archive validation is appended after the move.
