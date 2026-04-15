# Tasks: Tighten self-build proof hermeticity

## Phase 0: Spec check

- [x] Re-read the touched bootstrap spec against the final proof behavior before implementation starts
- [x] Keep scope fixed: no devshell/coreutils rewrite and no proof-helper shell rewrite in this change

## Phase 1: Strict mode plumbing

- [x] Thread hermeticity mode into `crunch self-build`
- [x] Thread hermeticity mode through `scripts/prove-self-hosting.sh` into the checked-in proof helper
- [x] Thread the exact stage0-produced `bwrap` and `busybox` roots into later proof-stage selection/reporting paths
- [x] Record exact bootstrap-tool provenance in proof summaries
- [x] Emit and summarize the selected later-stage hermeticity mode plus explicit fallback-event markers in proof output

## Phase 2: Later-stage enforcement

- [x] Reject host fallback sandbox-tool discovery in strict later proof stages once crunch-built tool roots exist
- [x] Reject staged-source checkout discovery in strict later proof stages unless `--source-store-path` is explicit
- [x] Keep practical mode reporting fallback use explicitly
- [x] Add regression coverage for stage1/stage2 fallback rejection, staged-source rejection, and zero later-stage fallback markers
- [x] Add regression coverage that proof summaries record exact later-stage tool roots plus both zero and nonzero fallback-event reporting
- [x] Add regression coverage that `scripts/prove-self-hosting.sh` resolves the default static busybox path when `SNIX_BUILD_SANDBOX_SHELL` is unset or `/bin/sh`
- [x] Add regression coverage that missing default-shell candidates fail explicitly without hidden `nix-build` fallback
- [x] Add regression coverage that `scripts/prove-self-hosting.sh` absolutizes a repo-relative `SNIX_BUILD_SANDBOX_SHELL` before later proof stages run from a temp directory
- [x] Add a stale-sibling regression/assertion that later-stage execution uses the exact reported stage0 `bwrap` and `busybox` roots, not a different store sibling

## Phase 3: Validation

- [x] Run `cargo test -p crunch --bin crunch self_build::tests -- --nocapture`
- [x] Run `cargo test -p crunch --test self_hosting prove_self_hosting_script_exports_strict_later_stage_hermeticity_by_default -- --nocapture`
- [x] Run `cargo test -p crunch --test self_hosting prove_self_hosting_script_discovers_repo_local_default_sandbox_shell_when_env_is_bin_sh -- --nocapture`
- [x] Run `cargo test -p crunch --test self_hosting prove_self_hosting_script_fails_fast_when_default_sandbox_shell_missing_without_nix_build_fallback -- --nocapture`
- [x] Run `cargo test -p crunch --test self_hosting prove_self_hosting_script_anchors_relative_sandbox_shell_to_repo_root -- --nocapture`
- [x] Run `cargo test -p crunch --test self_hosting write_proof_bundle_copies_stage_artifacts_and_manifest -- --nocapture`
- [x] Run `CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE=strict SNIX_BUILD_SANDBOX_SHELL=target/proof-busybox-static/bin/busybox ./scripts/prove-self-hosting.sh --bundle-dir target/self-hosting-proof/strict-hermeticity-check`
- [x] Run `openspec validate tighten-self-build-proof-hermeticity`
