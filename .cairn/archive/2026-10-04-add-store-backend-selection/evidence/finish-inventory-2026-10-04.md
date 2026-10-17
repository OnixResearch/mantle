# Store backend selection: pre-change inventory (2026-10-04)

This is an inventory, **not** a completed T1.1 golden capture or a passing
T3/T4 gate. The clean original was `c5740ee6e220c41c16eaa2de988eaf6c489aea1b`;
`7ec5177718a6950297e04eb4eb957a10b02e23ce` is its pre-selection
ancestor. The historical source is checked out separately at
`/home/brittonr/.cargo-target/mantle-backend-selection-evidence-3uhbppvb/prechange-snix`.
The original user checkout at `/home/brittonr/git/OnixResearch/mantle` was not touched.

Historical `StoreConfig` construction / `StoreHandle::open` inventory, obtained
by literal source search in `src/`, `crates/`, and `tests/` of the pre-change
snapshot. Some lines contain a constructor and open in one expression; listed
lines are call sites, not a sum of separately counted constructors and opens:

- Production composition: `src/attest_cmd.rs:675,683`;
  `src/build_plan.rs:591`; `src/foreign_import_cmd.rs:1064`;
  `src/foreign_provenance_audit.rs:118`;
  `src/foreign_realization_shell.rs:165,204`;
  `src/full_source_provider.rs:334`; `src/main.rs:5621,6316,7596`;
  `src/remote_build.rs:2542,2621,4003`;
  `src/store_cmd.rs:273,501`.
- Production wrappers: `crates/crunch-pipeline/src/lib.rs:270,289` constructs
  both the unlayered and overlay configs and opens their stores;
  `crates/crunch-rustc-wrapper/src/lib.rs:306` constructs the Rust cache
  daemon's store config;
  `crates/crunch-rust-cache/src/lib.rs:253` receives a caller-supplied config
  and opens its store. The historical config type and constructor live in
  `crates/crunch-store/src/handle.rs:109,137`.
- Source-local fixtures: `src/remote_build.rs:17481`,
  `src/remote_transfer.rs:2749`, `src/rust_plan.rs:19130`;
  `crates/crunch-build/src/orchestrate.rs:3123,3129`;
  `crates/crunch-pipeline/src/lib.rs:1201`;
  `crates/crunch-store/src/archive.rs:1075`, `composition.rs:424`,
  `gc.rs:1475`, `handle.rs:5612,5651,5740,6782,6856,7014,7045,7087,7123,7286,7312,7472`,
  `nario.rs:960`, `pull.rs:1380,1880,2854`, `push.rs:328,559`,
  `repair.rs:702`.
- Root integration fixtures: `tests/attest_cli.rs:257`,
  `tests/composition_root_cli.rs:39`, `tests/integration.rs:204`,
  `tests/integration_build.rs:475,504`, `tests/store_archive_cli.rs:38`,
  `tests/store_gc_cli.rs:33`.

Historical launcher inventory for backend forwarding review:

1. Local remote worker: `src/main.rs:5315-5322` forwards `--state-dir`
   and `--store-prefix`.
2. Bootstrap validation: `src/bootstrap_validate.rs:267-272` forwards
   `--store-prefix` and `--state-dir`.
3. Fixed-point shell: `src/source_built_fixed_point_shell.rs:1902-1907`
   forwards `--store-prefix` and `--state-dir`.
4. Transcript real child: `src/transcript_cmd.rs:494-499` injects
   `--state-dir` when the visible command omits it.
5. Rust cache daemon entry: `crates/crunch-rustc-wrapper/src/bin/mantle-rust-cache-daemon.rs:30-33`
   accepts `--state-dir` and `--store-output-dir` from its launcher.

The fixed historical signing fixture is `TEST_KEYPAIR` in the original
`tests/store_archive_cli.rs:22-23` and signs via that file's
`sign_pathinfo` at lines 109-120. This checked-in key is *test-only*;
no private-key bytes are copied into this evidence entry. PathInfo and NAR
capture must use that same fixture, the same logical prefix and a recorded
fixed environment before any historical-vs-selected comparison is accepted.
