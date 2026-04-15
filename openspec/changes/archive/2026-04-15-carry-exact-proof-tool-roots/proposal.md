# Carry exact proof tool roots into later self-build stages

## Why

The bootstrap spec now says later proof stages in strict mode MUST bind to the
exact stage0-produced `bwrap` and `busybox` roots. The previous hermeticity
change tightened fallback rejection and reporting, but later-stage process
handoff still relied on output-store rediscovery for tool selection.

That leaves a stale-sibling gap: if another `*-bwrap` or `*-busybox` entry is
present in the same store, later-stage selection can drift from the exact
stage0 roots even while staying crunch-built.

## What Changes

- carry exact stage0 `bwrap` and `busybox` paths into later proof-stage
  `crunch self-build` invocations through hidden proof-only arguments
- make later strict proof stages reuse those exact paths instead of scanning
  the output store for matching suffixes
- add regression coverage with stale sibling tool outputs present in the store;
  the exact assertion lives in `tests/self_hosting.rs::self_hosting_stage0_stage1_stage2`
  and checks that stage2 still uses the exact stage0 `bwrap` and `busybox`
  paths even when extra `*-bwrap` and `*-busybox` siblings exist
- rerun the exact validation commands and keep their command output as evidence:
  - `cargo test -p crunch --bin crunch self_build::tests -- --nocapture`
    - evidence: command stdout/stderr transcript
  - `cargo test -p crunch --test self_hosting prove_self_hosting_script_exports_strict_later_stage_hermeticity_by_default -- --nocapture`
    - evidence: command stdout/stderr transcript
  - `cargo test -p crunch --test self_hosting write_proof_bundle_copies_stage_artifacts_and_manifest -- --nocapture`
    - evidence: command stdout/stderr transcript
  - `CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE=strict SNIX_BUILD_SANDBOX_SHELL=target/proof-busybox-static/bin/busybox ./scripts/prove-self-hosting.sh --bundle-dir target/self-hosting-proof/strict-hermeticity-check`
    - evidence: command stdout/stderr transcript plus `target/self-hosting-proof/strict-hermeticity-check/summary.txt`
    - note: this repo-relative shell path depends on helper-side absolutization in `scripts/prove-self-hosting.sh`
  - `openspec validate carry-exact-proof-tool-roots`
    - evidence: command stdout/stderr transcript

## Non-Goals

- changing stage0 host-prerequisite behavior
- changing GNU mirror policy or other bootstrap source URLs
- changing non-strict proof behavior beyond the exact-root handoff plumbing
- changing staged-source strict rejection behavior; this change relies on the already-specified behavior but does not modify it
- changing unrelated `crunch build` tool-resolution behavior outside the self-hosting proof path

## Impact

- **Files**: `src/main.rs`, `src/self_build.rs`, `tests/self_hosting.rs`
- **Behavior**: strict later proof stages bind to exact stage0 tool roots even
  when stale siblings exist in the store
- **Testing**: add exact-root unit coverage and exercise the full strict proof
