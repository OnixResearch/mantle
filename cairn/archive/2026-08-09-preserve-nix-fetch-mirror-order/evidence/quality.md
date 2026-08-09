## Task formatting command

```text
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/foreign_derivation_import.rs:3057:
         legacy_source.as_object_mut().expect("source object").remove("fetch_candidates");
         let legacy = serde_json::from_value::<ForeignDerivationGraph>(legacy_value).expect("legacy graph decode");
         validate_graph(&legacy).expect("legacy graph validation");
[31m-        assert!(legacy
(B[m[31m-            .nodes
(B[m[31m-            .iter()
(B[m[31m-            .find(|node| node.original_derivation == NIXPKGS_SOURCE_DRV)
(B[m[31m-            .expect("legacy source")
(B[m[31m-            .fetch_candidates
(B[m[31m-            .is_empty());
(B[m[32m+        assert!(
(B[m[32m+            legacy
(B[m[32m+                .nodes
(B[m[32m+                .iter()
(B[m[32m+                .find(|node| node.original_derivation == NIXPKGS_SOURCE_DRV)
(B[m[32m+                .expect("legacy source")
(B[m[32m+                .fetch_candidates
(B[m[32m+                .is_empty()
(B[m[32m+        );
(B[m 
         let mut duplicate = artifacts.graph.clone();
         let duplicate_source = duplicate
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/foreign_derivation_import.rs:3080:
             .iter_mut()
             .find(|node| node.original_derivation == NIXPKGS_SOURCE_DRV)
             .expect("source node")
[31m-            .fetch_candidates = (0..=MAX_MIRROR_CANDIDATES)
(B[m[31m-            .map(|index| format!("https://mirror{index}.example/source"))
(B[m[31m-            .collect();
(B[m[32m+            .fetch_candidates =
(B[m[32m+            (0..=MAX_MIRROR_CANDIDATES).map(|index| format!("https://mirror{index}.example/source")).collect();
(B[m         assert_error_class(validate_graph(&oversized), "nix-fetch-candidate-count-invalid");
 
         let mut wrong_kind = artifacts.graph;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:18:
 use crate::errors::RunError;
 use crate::full_source_rust_binding_shell::FullSourceRustHostToolMaterializationRequest;
 use crate::native_toolchain_closure::NativeToolchainClosureOptions;
[31m-use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m use crate::source_built_fixed_point::InitialOutputAuthorityState;
 use crate::source_built_fixed_point::ProofHermeticityMode;
[32m+use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m use crate::source_built_fixed_point::SourceAuthorityInput;
 use crate::source_built_fixed_point::SourceAuthorityRole;
 use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:28:
 use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
 use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
 use crate::source_built_fixed_point::SourceContentKind;
[31m-use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheLookup;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:39:
[32m+use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m use crate::source_built_fixed_point_dev_cache::FastFailDecision;
 use crate::source_built_fixed_point_dev_cache::StageCompletionMarker;
 use crate::source_built_fixed_point_dev_cache::StageMarkerValidation;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:42:
[31m-use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[32m+use crate::source_bundle::SourceRecord;
(B[m use crate::source_bundle::assemble_source_bundle;
 use crate::source_bundle::materialize_source_record_payload;
 use crate::source_bundle::source_built_fixed_point_profile_records;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:47:
[31m-use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[31m-use crate::source_bundle::SourceRecord;
(B[m use crate::stagex_provider::StagexProviderRequest;
 use crate::stagex_transition::StagexTransitionRequest;
 

exit_code=1
```

## First-party Clippy

```text
[clippy] first-party strict gate
running: cargo clippy --workspace --all-targets --no-deps --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing -- -D warnings
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking crunch-attestation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-attestation-core)
   Compiling nix-compat v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/nix-compat)
   Compiling nix-compat-derive v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/nix-compat-derive)
   Compiling fuse-backend-rs v0.12.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/fuse-backend-rs)
   Compiling snix-castore v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-castore)
    Checking snix-tracing v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-tracing)
   Compiling snix-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-store)
   Compiling snix-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/vendor/snix-build)
    Checking crunch-action-result-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-action-result-core)
    Checking crunch-overlay-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-overlay-core)
    Checking crunch-gc-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-gc-core)
    Checking crunch-repair-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-repair-core)
    Checking crunch-rust-cache-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rust-cache-core)
    Checking crunch-delta-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-delta-core)
    Checking crunch-eval v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-eval)
    Checking crunch-wasm-component-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-wasm-component-core)
    Checking crunch-shell-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-shell-core)
    Checking crunch-bootstrap-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-bootstrap-core)
    Checking mantlepkgs-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/mantlepkgs-core)
    Checking crunch-release-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-release-core)
    Checking mantle-portable-client-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/mantle-portable-client-core)
    Checking crunch-kernelscript-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-kernelscript-core)
    Checking crunch-hardware-simulation-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-hardware-simulation-core)
    Checking crunch-spacewasm-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-spacewasm-core)
    Checking crunch-shell v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-shell)
    Checking crunch-attestation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-attestation)
    Checking crunch-project-core v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-project-core)
    Checking crunch-spacewasm v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-spacewasm)
    Checking crunch-hardware-simulation v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-hardware-simulation)
    Checking crunch-kernelscript-adapter v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-kernelscript-adapter)
    Checking crunch-nar v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-nar)
    Checking crunch-glue v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-glue)
    Checking crunch-wasm-component v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-wasm-component)
    Checking crunch-project v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-project)
    Checking crunch-store v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-store)
    Checking crunch-delta v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-delta)
    Checking crunch-rust-cache v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rust-cache)
    Checking crunch-build v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-build)
    Checking crunch-rustc-wrapper v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-rustc-wrapper)
    Checking crunch-pipeline v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/crates/crunch-pipeline)
    Checking mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 03s

exit_code=0
```

## Foreign import trust model

```text
foreign import trust-model doc check passed

exit_code=0
```

## Foreign import trust model self-test

```text
foreign import trust-model checker self-test passed

exit_code=0
```

## Diff whitespace

```text

exit_code=0
```


# Definitive quality rerun

## Changed-file rustfmt

```text

exit_code=0
```

## Task formatting command final

```text
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:18:
 use crate::errors::RunError;
 use crate::full_source_rust_binding_shell::FullSourceRustHostToolMaterializationRequest;
 use crate::native_toolchain_closure::NativeToolchainClosureOptions;
[31m-use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m use crate::source_built_fixed_point::InitialOutputAuthorityState;
 use crate::source_built_fixed_point::ProofHermeticityMode;
[32m+use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m use crate::source_built_fixed_point::SourceAuthorityInput;
 use crate::source_built_fixed_point::SourceAuthorityRole;
 use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:28:
 use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
 use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
 use crate::source_built_fixed_point::SourceContentKind;
[31m-use crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_OPEN_FILE_DESCRIPTORS_MAX;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_built_fixed_point::plan_source_built_fixed_point;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheEntry;
 use crate::source_built_fixed_point_dev_cache::DevProviderCacheLookup;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:39:
[32m+use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m use crate::source_built_fixed_point_dev_cache::FastFailDecision;
 use crate::source_built_fixed_point_dev_cache::StageCompletionMarker;
 use crate::source_built_fixed_point_dev_cache::StageMarkerValidation;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:42:
[31m-use crate::source_built_fixed_point_dev_cache::DEV_CACHE_ENTRY_FILE;
(B[m[31m-use crate::source_built_fixed_point_dev_cache::FAST_FAIL_SCHEMA;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::dev_provider_cache_key;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_fast_fail;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::evaluate_provider_cache_lookup;
(B[m[32m+use crate::source_built_fixed_point_dev_cache::validate_stage_marker;
(B[m[32m+use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[32m+use crate::source_bundle::SourceRecord;
(B[m use crate::source_bundle::assemble_source_bundle;
 use crate::source_bundle::materialize_source_record_payload;
 use crate::source_bundle::source_built_fixed_point_profile_records;
Diff in /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/source_built_fixed_point_shell.rs:47:
[31m-use crate::source_bundle::SourceBuiltFixedPointProfileRecords;
(B[m[31m-use crate::source_bundle::SourceRecord;
(B[m use crate::stagex_provider::StagexProviderRequest;
 use crate::stagex_transition::StagexTransitionRequest;
 

exit_code=1
```

## First-party Clippy final

```text
[clippy] first-party strict gate
running: cargo clippy --workspace --all-targets --no-deps --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing -- -D warnings
warning: /home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/Cargo.toml: file `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking mantle v0.1.0 (/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-preserve-nix-fetch-mirror-order-20260808)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 22.00s

exit_code=0
```

## Foreign import trust model final

```text
foreign import trust-model doc check passed

exit_code=0
```

## Foreign import trust model self-test final

```text
foreign import trust-model checker self-test passed

exit_code=0
```

## Diff whitespace final

```text

exit_code=0
```


## Unchanged excluded formatting blocker

```text
exit_code=0
```
