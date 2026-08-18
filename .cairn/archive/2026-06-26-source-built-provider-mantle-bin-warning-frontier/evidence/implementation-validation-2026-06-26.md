# Implementation validation: Mantle binary warning frontier

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]

## Environment

```text
cwd: /home/brittonr/git/mantle
cargo: /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo
PKG_CONFIG_PATH: /nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
```

## cargo-import-tests

```text
command: cargo test -p mantle --bin mantle cargo_import_plan -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling parking_lot_core v0.9.12
   Compiling generic-array v0.14.7
   Compiling crossbeam-utils v0.8.21
   Compiling serde v1.0.228
   Compiling thiserror v2.0.18
   Compiling zmij v1.0.21
   Compiling zerofrom v0.1.7
   Compiling futures-util v0.3.32
   Compiling tracing v0.1.44
   Compiling aws-lc-sys v0.39.1
   Compiling ring v0.17.14
   Compiling httparse v1.10.1
   Compiling icu_properties_data v2.2.0
   Compiling darling_core v0.23.0
   Compiling num-traits v0.2.19
   Compiling derive_more-impl v2.1.1
   Compiling anyhow v1.0.102
   Compiling zstd-sys v2.0.16+zstd.1.5.7
   Compiling lalrpop-util v0.22.2
   Compiling yoke v0.8.2
   Compiling arc-swap v1.9.1
   Compiling parking_lot v0.12.5
   Compiling serde_json v1.0.149
   Compiling tracing-subscriber v0.3.23
   Compiling zstd-safe v7.2.4
   Compiling pin-project v1.1.11
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling crossbeam-channel v0.5.15
   Compiling bytes v1.11.1
   Compiling bstr v1.12.1
   Compiling dashmap v6.1.0
   Compiling crossbeam-epoch v0.9.18
   Compiling aws-lc-rs v1.16.2
   Compiling zerovec v0.11.6
   Compiling zerotrie v0.2.4
   Compiling derive_more v2.1.1
   Compiling serde_urlencoded v0.7.1
   Compiling darling_macro v0.23.0
   Compiling prodash v31.0.0
   Compiling clap v4.6.0
   Compiling bzip2-sys v0.1.13+1.0.8
   Compiling digest v0.10.7
   Compiling memoffset v0.6.5
   Compiling libmimalloc-sys v0.1.44
   Compiling crossbeam-deque v0.8.6
   Compiling tokio v1.51.0
   Compiling gix-validate v0.11.0
   Compiling gix-utils v0.3.1
   Compiling gix-error v0.2.1
   Compiling http v1.4.0
   Compiling gix-packetline v0.21.2
   Compiling string_cache v0.8.9
   Compiling lzma-sys v0.1.20
   Compiling darling_core v0.20.11
   Compiling futures-executor v0.3.32
   Compiling darling v0.23.0
   Compiling tinystr v0.8.3
   Compiling potential_utf v0.1.5
   Compiling sha1 v0.10.6
   Compiling sha2 v0.10.9
   Compiling chrono v0.4.44
   Compiling mimalloc v0.1.48
   Compiling rustls-webpki v0.103.12
   Compiling rayon-core v1.13.0
   Compiling gix-path v0.11.2
   Compiling nix v0.24.3
   Compiling sha3 v0.10.8
   Compiling md-5 v0.10.6
   Compiling gix-date v0.15.1
   Compiling gix-chunk v0.7.0
   Compiling gix-quote v0.7.0
   Compiling icu_collections v2.2.0
   Compiling gix-bitmap v0.3.0
   Compiling serde_with_macros v3.18.0
   Compiling icu_locale_core v2.2.0
   Compiling futures v0.3.32
   Compiling sha1-checked v0.10.0
   Compiling http-body v1.0.1
   Compiling xz2 v0.1.7
   Compiling gix-features v0.46.2
   Compiling blake3 v1.8.2
   Compiling rustls v0.23.37
   Compiling gix-config-value v0.17.1
   Compiling darling_macro v0.20.11
   Compiling curve25519-dalek v4.1.3
   Compiling heapless v0.7.17
   Compiling gix-actor v0.40.0
   Compiling gix-command v0.8.0
   Compiling lalrpop v0.22.2
   Compiling async-stream v0.3.6
   Compiling http-body-util v0.1.3
   Compiling thiserror v1.0.69
   Compiling n0-error v0.1.3
   Compiling malachite-nz v0.6.1
   Compiling serde_with v3.18.0
   Compiling num_enum_derive v0.7.6
   Compiling crunch-attestation-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation-core)
   Compiling icu_provider v2.2.0
   Compiling gix-hash v0.23.0
   Compiling gix-fs v0.19.2
   Compiling gix-glob v0.24.0
   Compiling bzip2 v0.5.2
   Compiling clap-verbosity-flag v3.0.4
   Compiling tracing-indicatif v0.3.14
   Compiling zstd v0.13.3
   Compiling quick-xml v0.39.2
   Compiling cobs v0.3.0
   Compiling logos-codegen v0.15.1
   Compiling num_enum v0.7.6
   Compiling gix-hashtable v0.13.0
   Compiling icu_properties v2.2.0
   Compiling icu_normalizer v2.2.0
   Compiling gix-tempfile v21.0.2
   Compiling gix-commitgraph v0.35.0
   Compiling gix-attributes v0.31.0
   Compiling gix-ignore v0.19.1
   Compiling postcard v1.1.3
   Compiling serde_qs v0.12.0
   Compiling tokio-util v0.7.18
   Compiling tower v0.5.3
   Compiling tokio-stream v0.1.18
   Compiling ed25519-dalek v2.2.0
   Compiling async-compression v0.4.19
   Compiling gix-object v0.58.0
   Compiling darling v0.20.11
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/mantle/vendor/fuse-backend-rs)
   Compiling serde_tagged v0.3.0
   Compiling gix-lock v21.0.2
   Compiling proc-macro2-diagnostics v0.10.1
   Compiling gix-url v0.35.2
   Compiling tokio-rustls v0.26.4
   Compiling rustls-platform-verifier v0.6.2
   Compiling gix-pathspec v0.16.1
   Compiling codespan-reporting v0.13.1
   Compiling logos-derive v0.15.1
   Compiling nix-compat v0.1.0 (/home/brittonr/git/mantle/vendor/nix-compat)
   Compiling tower-http v0.6.8
   Compiling derive_builder_core v0.20.2
   Compiling fastcdc v3.2.1
   Compiling h2 v0.4.13
   Compiling n0-future v0.3.2
   Compiling idna_adapter v1.2.1
   Compiling astral-tokio-tar v0.6.0
   Compiling ouroboros_macro v0.18.5
   Compiling gix-revwalk v0.29.0
   Compiling gix-filter v0.28.0
   Compiling gix-ref v0.61.0
   Compiling crunch-attestation v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation)
   Compiling nickel-lang-parser v0.1.1
   Compiling gix-prompt v0.14.1
   Compiling logos v0.15.1
   Compiling nickel-lang-core v0.16.1
   Compiling codespan v0.13.1
   Compiling nickel-lang-vector v0.1.0
   Compiling idna v1.1.0
   Compiling irpc v0.13.0
   Compiling zerocopy v0.8.48
   Compiling derive_builder_macro v0.20.2
   Compiling gix-traverse v0.55.0
   Compiling gix-revision v0.43.0
   Compiling serde_yaml v0.9.34+deprecated
   Compiling gix-credentials v0.37.1
   Compiling gix-pack v0.68.0
   Compiling gix-negotiate v0.29.0
   Compiling gix-shallow v0.10.0
   Compiling ouroboros v0.18.5
   Compiling gix-discover v0.49.0
   Compiling gix-config v0.54.0
   Compiling malachite-q v0.6.1
   Compiling sha-1 v0.10.1
   Compiling derive_builder v0.20.2
   Compiling url v2.5.8
   Compiling typed-builder v0.22.0
   Compiling gix-mailmap v0.32.0
   Compiling gix-index v0.49.0
   Compiling gix-refspec v0.39.0
   Compiling gix-worktree-stream v0.30.0
   Compiling ureq-proto v0.6.0
   Compiling serde_spanned v0.6.9
   Compiling toml_datetime v0.6.11
   Compiling crunch-glue v0.1.0 (/home/brittonr/git/mantle/crates/crunch-glue)
   Compiling oci-spec v0.7.1
   Compiling crunch-project-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project-core)
   Compiling crunch-shell-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-shell-core)
   Compiling float-cmp v0.10.0
   Compiling crunch-bootstrap-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-bootstrap-core)
   Compiling crunch-release-core v0.1.0 (/home/brittonr/git/mantle/crates/crunch-release-core)
   Compiling gix-odb v0.78.0
   Compiling malachite-float v0.6.1
   Compiling gix-submodule v0.28.0
   Compiling hyper v1.9.0
   Compiling gix-archive v0.30.0
   Compiling toml_edit v0.22.27
   Compiling predicates v3.1.4
   Compiling ureq v3.3.0
   Compiling gix-worktree v0.50.0
   Compiling crunch-shell v0.1.0 (/home/brittonr/git/mantle/crates/crunch-shell)
   Compiling crunch-project v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project)
   Compiling gix-diff v0.61.0
   Compiling gix-dir v0.23.0
   Compiling gix-worktree-state v0.28.0
   Compiling assert_cmd v2.2.0
   Compiling malachite v0.6.1
   Compiling hyper-util v0.1.20
   Compiling toml v0.8.23
   Compiling gix-blame v0.11.0
   Compiling gix-status v0.28.0
   Compiling ppv-lite86 v0.2.21
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.13.2
   Compiling reqwest v0.12.28
   Compiling rand_chacha v0.3.1
   Compiling rand v0.8.6
   Compiling object_store v0.13.2
   Compiling reqwest-middleware v0.5.1
   Compiling gix-transport v0.55.1
   Compiling reqwest-tracing v0.6.0
   Compiling gix-protocol v0.59.0
   Compiling snix-tracing v0.1.0 (/home/brittonr/git/mantle/vendor/snix-tracing)
   Compiling gix v0.81.0
   Compiling snix-castore v0.1.0 (/home/brittonr/git/mantle/vendor/snix-castore)
   Compiling snix-store v0.1.0 (/home/brittonr/git/mantle/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
   Compiling crunch-store v0.1.0 (/home/brittonr/git/mantle/crates/crunch-store)
   Compiling crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
   Compiling crunch-delta v0.1.0 (/home/brittonr/git/mantle/crates/crunch-delta)
   Compiling nickel-lang v2.0.0
   Compiling crunch-eval v0.1.0 (/home/brittonr/git/mantle/crates/crunch-eval)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 21.40s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 4 tests
test cargo_import::tests::cargo_import_plan_blocks_unsupported_and_ambiguous_surfaces ... ok
test cargo_import::tests::cargo_import_plan_blocks_missing_lockfile_malformed_name_and_registry_dependency ... ok
test cargo_import::tests::cargo_import_plan_reports_existing_file_conflicts_without_applying ... ok
test cargo_import::tests::cargo_import_plan_generates_build_shaped_files_for_local_binary_workspace ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 864 filtered out; finished in 0.00s


exit_status: 0
```

## toolchain-alias-tests

```text
command: cargo test -p mantle --bin mantle toolchain_path_aliases -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 868 filtered out; finished in 0.00s


exit_status: 0
```

## rust-bootstrap-patch-plan-tests

```text
command: cargo test -p mantle --bin mantle rust_bootstrap_patch_plan -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 7 tests
test rust_bootstrap_patch_plan::tests::patch_plan_rejects_mismatched_first_stage_versions ... ok
test rust_bootstrap_patch_plan::tests::patch_plan_rejects_unsupported_bootstrap_version ... ok
test rust_bootstrap_patch_plan::tests::patch_plan_rejects_missing_source_identity_before_shell_mutation ... ok
test rust_bootstrap_patch_plan::tests::patch_plan_rejects_missing_musl_runtime_capability ... ok
test rust_bootstrap_patch_plan::tests::non_musl_first_stage_plan_omits_musl_route_repairs ... ok
test rust_bootstrap_patch_plan::tests::rust_bootstrap_plan_includes_final_rustdoc_rlib_lookup_repair ... ok
test rust_bootstrap_patch_plan::tests::musl_host_first_stage_plan_derives_ordered_operations_and_stable_digest ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 861 filtered out; finished in 0.00s


exit_status: 0
```

## unset-env-mantle-bin-check

```text
command: env -u SNIX_BUILD_SANDBOX_SHELL cargo check -p mantle --bin mantle
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking thiserror v2.0.18
    Checking thiserror v1.0.69
    Checking curve25519-dalek v4.1.3
    Checking n0-error v0.1.3
    Checking logos v0.15.1
    Checking nickel-lang-parser v0.1.1
    Checking serde_qs v0.12.0
    Checking gix-path v0.11.2
    Checking gix-packetline v0.21.2
    Checking reqwest-middleware v0.5.1
    Checking cobs v0.3.0
    Checking object_store v0.13.2
    Checking crunch-attestation v0.1.0 (/home/brittonr/git/mantle/crates/crunch-attestation)
    Checking oci-spec v0.7.1
    Checking irpc v0.13.0
    Checking postcard v1.1.3
    Checking gix-features v0.46.2
    Checking gix-command v0.8.0
    Checking gix-config-value v0.17.1
    Checking gix-url v0.35.2
    Checking crunch-project v0.1.0 (/home/brittonr/git/mantle/crates/crunch-project)
    Checking reqwest-tracing v0.6.0
    Checking ed25519-dalek v2.2.0
    Checking gix-hash v0.23.0
    Checking gix-fs v0.19.2
    Checking gix-glob v0.24.0
    Checking gix-prompt v0.14.1
    Checking nix-compat v0.1.0 (/home/brittonr/git/mantle/vendor/nix-compat)
    Checking snix-tracing v0.1.0 (/home/brittonr/git/mantle/vendor/snix-tracing)
    Checking gix-credentials v0.37.1
    Checking gix-tempfile v21.0.2
    Checking gix-attributes v0.31.0
    Checking gix-hashtable v0.13.0
    Checking gix-commitgraph v0.35.0
    Checking gix-ignore v0.19.1
    Checking snix-castore v0.1.0 (/home/brittonr/git/mantle/vendor/snix-castore)
    Checking gix-object v0.58.0
    Checking gix-lock v21.0.2
    Checking gix-transport v0.55.1
    Checking gix-pathspec v0.16.1
    Checking crunch-glue v0.1.0 (/home/brittonr/git/mantle/crates/crunch-glue)
    Checking gix-shallow v0.10.0
    Checking gix-revwalk v0.29.0
    Checking gix-filter v0.28.0
    Checking gix-ref v0.61.0
    Checking gix-pack v0.68.0
    Checking gix-traverse v0.55.0
    Checking gix-revision v0.43.0
    Checking gix-negotiate v0.29.0
    Checking gix-discover v0.49.0
    Checking gix-config v0.54.0
    Checking gix-odb v0.78.0
    Checking gix-refspec v0.39.0
    Checking gix-index v0.49.0
    Checking gix-worktree-stream v0.30.0
    Checking snix-store v0.1.0 (/home/brittonr/git/mantle/vendor/snix-store)
    Checking snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
    Checking gix-archive v0.30.0
    Checking gix-submodule v0.28.0
    Checking gix-protocol v0.59.0
    Checking gix-worktree v0.50.0
    Checking gix-diff v0.61.0
    Checking gix-dir v0.23.0
    Checking gix-worktree-state v0.28.0
    Checking crunch-store v0.1.0 (/home/brittonr/git/mantle/crates/crunch-store)
    Checking gix-status v0.28.0
    Checking gix-blame v0.11.0
    Checking gix v0.81.0
    Checking crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
    Checking nickel-lang-core v0.16.1
    Checking nickel-lang v2.0.0
    Checking crunch-eval v0.1.0 (/home/brittonr/git/mantle/crates/crunch-eval)
    Checking crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
    Checking mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.15s

exit_status: 0
```

## Warning-frontier assertion

```text
no rustc unused-item warnings matched in focused transcript
```
