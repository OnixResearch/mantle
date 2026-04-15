# Tasks: Carry exact proof tool roots into later self-build stages

## Phase 1: Proof handoff plumbing

- [x] Add hidden self-build arguments for exact stage0 `bwrap` and `busybox` paths
- [x] Validate handed-off tool paths and reject paths outside the active self-build store
- [x] Reuse handed-off exact tool roots in later proof stages instead of suffix scanning

## Phase 2: Regression coverage

- [x] Add unit coverage for exact tool-path validation helpers
- [x] Add a stale-sibling regression/assertion that stage2 still uses the exact stage0 `bwrap` and `busybox` roots when extra `*-bwrap` and `*-busybox` siblings exist in the store

## Phase 3: Validation

- [x] Run `cargo test -p crunch --bin crunch self_build::tests -- --nocapture`
- [x] Keep the stdout/stderr transcript from `cargo test -p crunch --bin crunch self_build::tests -- --nocapture` as review evidence
- [x] Run `cargo test -p crunch --bin crunch resolve_explicit_bootstrap_bwrap_source_ignores_stale_sibling -- --nocapture`
- [x] Keep the stdout/stderr transcript from `cargo test -p crunch --bin crunch resolve_explicit_bootstrap_bwrap_source_ignores_stale_sibling -- --nocapture` as review evidence
- [x] Run `cargo test -p crunch --test self_hosting prove_self_hosting_script_exports_strict_later_stage_hermeticity_by_default -- --nocapture`
- [x] Keep the stdout/stderr transcript from `cargo test -p crunch --test self_hosting prove_self_hosting_script_exports_strict_later_stage_hermeticity_by_default -- --nocapture` as review evidence
- [x] Run `cargo test -p crunch --test self_hosting write_proof_bundle_copies_stage_artifacts_and_manifest -- --nocapture`
- [x] Keep the stdout/stderr transcript from `cargo test -p crunch --test self_hosting write_proof_bundle_copies_stage_artifacts_and_manifest -- --nocapture` as review evidence
- [x] Run `CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE=strict SNIX_BUILD_SANDBOX_SHELL=target/proof-busybox-static/bin/busybox ./scripts/prove-self-hosting.sh --bundle-dir target/self-hosting-proof/strict-hermeticity-check` as the end-to-end `tests/self_hosting.rs::self_hosting_stage0_stage1_stage2` proof, including the stale-sibling exact-root assertions
- [x] Keep the stdout/stderr transcript from the strict proof helper run as review evidence
- [x] Inspect `target/self-hosting-proof/strict-hermeticity-check/summary.txt` for `stage2_hermeticity_mode: strict`, `stage0_bwrap_equals_stage2_bwrap: true`, `stage0_busybox_equals_stage2_busybox: true`, and `stage2_fallback_events: []`
- [x] Inspect the stage2 proof output for `self-build-proof: fallback-event=none`, the exact stage0 `bwrap-source=crunch-built:...`, and the exact stage0 `busybox-path=...`
- [x] Review `src/main.rs`, `src/self_build.rs`, and `tests/self_hosting.rs` for hidden exact-root handoff plumbing and later-stage reuse without store rescanning
- [x] Run `openspec validate carry-exact-proof-tool-roots`
- [x] Keep the stdout/stderr transcript from `openspec validate carry-exact-proof-tool-roots` as review evidence
- [x] Write `openspec/changes/carry-exact-proof-tool-roots/validation.md` with transcript paths, summary-artifact checks, and code-evidence pointers for `src/main.rs`, `src/self_build.rs`, and `tests/self_hosting.rs`
