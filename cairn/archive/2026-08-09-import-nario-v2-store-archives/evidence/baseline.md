# Baseline evidence

Pinned candidate authority: DeterminateSystems/nix-src tag v3.12.0 at 9512828397f684d0f732ea76b7631f69a0db34f7.

## crunch-store archive tests

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling smallvec v1.15.1
   Compiling rustls v0.23.37
   Compiling rustls-webpki v0.103.13
   Compiling rustix v0.38.44
   Compiling xattr v1.6.1
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/fuse-backend-rs)
   Compiling linux-raw-sys v0.4.15
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/nix-compat)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/snix-castore)
   Compiling tempfile v3.27.0
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/nix-compat-derive)
   Compiling sha1 v0.10.6
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/snix-build)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/snix-store)
   Compiling nix-archive v0.1.0
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-attestation-core)
   Compiling crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-action-result-core)
   Compiling ppv-lite86 v0.2.21
   Compiling crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-gc-core)
   Compiling crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-overlay-core)
   Compiling parking_lot_core v0.9.12
   Compiling parking_lot v0.12.5
   Compiling icu_normalizer v2.2.0
   Compiling nibble_vec v0.1.0
   Compiling tracing-subscriber v0.3.23
   Compiling tar v0.4.45
   Compiling crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-repair-core)
   Compiling rusty-fork v0.3.1
   Compiling rand_chacha v0.9.0
   Compiling crunch-delta-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-delta-core)
   Compiling tokio v1.51.0
   Compiling radix_trie v0.2.1
   Compiling proptest v1.11.0
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-attestation)
   Compiling idna_adapter v1.2.1
   Compiling idna v1.1.0
   Compiling url v2.5.8
   Compiling tracing-indicatif v0.3.14
   Compiling rustls-platform-verifier v0.6.2
   Compiling tokio-util v0.7.18
   Compiling tokio-rustls v0.26.4
   Compiling tower v0.5.3
   Compiling tokio-stream v0.1.18
   Compiling async-compression v0.4.19
   Compiling astral-tokio-tar v0.6.3
   Compiling fastcdc v3.2.1
   Compiling tower-http v0.6.8
   Compiling h2 v0.4.13
   Compiling n0-future v0.3.2
   Compiling irpc v0.13.0
   Compiling crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-nar)
   Compiling hyper v1.9.0
   Compiling hyper-util v0.1.20
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling object_store v0.14.0
   Compiling reqwest-tracing v0.6.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/snix-tracing)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-store)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-delta)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 49.50s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_store-b14c7d60e8245d3c)

running 22 tests
test archive::tests::import_action_core_rejects_untrusted_and_skips_present ... ok
test archive::tests::archive_list_rejects_bad_magic ... ok
test archive::tests::pathinfo_fixture_service_is_bounded ... ok
test archive::tests::archive_list_rejects_header_end_count_mismatch ... ok
test archive::tests::archive_export_refuses_unsigned_without_escape_hatch ... ok
test archive::tests::ca_path_identity_accepts_reference_aware_standard_path ... ok
test archive::tests::archive_export_rejects_stale_final_nar_facts_before_writing ... ok
test archive::tests::missing_closure_reference_fails_export_plan ... ok
test archive::tests::archive_export_list_round_trip_preserves_metadata_before_payload ... ok
test archive::tests::export_closure_includes_references_deterministically ... ok
test archive::tests::archive_list_drains_non_seekable_payloads_in_bounded_chunks ... ok
test archive::tests::archive_import_rejects_store_prefix_mismatch_before_persisting ... ok
test archive::tests::archive_import_rejects_unsupported_ca_metadata_without_persisting ... ok
test archive::tests::archive_import_rejects_ca_metadata_for_another_store_path ... ok
test archive::tests::archive_import_rejects_existing_path_with_stale_final_nar_facts ... ok
test archive::tests::archive_round_trip_preserves_distinct_marker_ca_and_final_nar_identities ... ok
test archive::tests::archive_import_rejects_conflicting_local_pathinfo ... ok
test archive::tests::archive_import_rejects_untrusted_signature_without_persisting ... ok
test archive::tests::archive_import_rejects_truncated_payload_without_persisting ... ok
test archive::tests::archive_import_rejects_tampered_payload_without_persisting ... ok
test archive::tests::archive_import_round_trip_and_skip_existing_are_idempotent ... ok
test archive::tests::large_archive_payload_stays_on_the_chunked_castore_ingest_path ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 323 filtered out; finished in 0.09s

     Running tests/authority_source_policy.rs (/home/brittonr/.cargo-target/debug/deps/authority_source_policy-d6caab10be9df2e6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

```

## store archive CLI tests

```text
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-attestation-core)
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/fuse-backend-rs)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/nix-compat)
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/snix-tracing)
   Compiling crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-action-result-core)
   Compiling crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-gc-core)
   Compiling crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-repair-core)
   Compiling crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-overlay-core)
   Compiling crunch-rust-cache-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-rust-cache-core)
   Compiling crunch-eval v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-eval)
   Compiling crunch-wasm-component-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-wasm-component-core)
   Compiling crunch-shell-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-shell-core)
   Compiling crunch-bootstrap-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-bootstrap-core)
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-release-core)
   Compiling mantlepkgs-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/mantlepkgs-core)
   Compiling mantle-portable-client-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/mantle-portable-client-core)
   Compiling crunch-hardware-simulation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-hardware-simulation-core)
   Compiling crunch-kernelscript-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-kernelscript-core)
   Compiling crunch-shell v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-shell)
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-attestation)
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-project-core)
   Compiling crunch-hardware-simulation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-hardware-simulation)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/snix-castore)
   Compiling crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-nar)
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-glue)
   Compiling crunch-wasm-component v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-wasm-component)
   Compiling crunch-project v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-project)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/vendor/snix-build)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-store)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-build)
   Compiling crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-rust-cache)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-delta)
   Compiling crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-rustc-wrapper)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-import-nario-v2-store-archives-20260809)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 57s
     Running tests/store_archive_cli.rs (/home/brittonr/.cargo-target/debug/deps/store_archive_cli-37cfb75e9dbcd173)

running 3 tests
test store_archive_cli_requires_unsigned_escape_hatch ... ok
test store_archive_cli_streams_stdout_and_stdin ... ok
test store_archive_cli_exports_lists_json_and_imports_from_file ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

```
