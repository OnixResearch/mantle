# second-wave functional-core validation

## Why

Crunch now has two more real no-std functional-core domains:
`crunch-shell-core` and `crunch-release-core`.

That code already exists, and targeted checks are green, but the main
`functional-core`, `architecture`, and `portability` specs still describe only
`crunch-attestation-core` and `crunch-project-core`. The checked-in no-std
runner and Python checkers still model only that first wave too.

That mismatch creates false confidence:

- `./scripts/check-no-std-core.sh` can go green while second-wave cores are
  outside the declared proof surface.
- ownership and API-shape review still reason from first-wave-only inventory
  even though shell activation and release evidence now use the same no-std/
  std split.
- future FCIS work has no spec-backed way to extend validation beyond the
  original pilot crates.

This change closes that drift before more no-std work stacks on top.

## What Changes

- extend `functional-core`, `architecture`, and `portability` specs so the
  no-std core tier explicitly includes `crunch-shell-core` and
  `crunch-release-core`
- define the second-wave std shell/adaptor ownership for `crunch-shell`,
  `src/release_evidence.rs`, and `src/release_cmd.rs`
- require the no-std validation runner, scope/API-shape checks, dependency
  allowlist, and ownership review to cover both second-wave cores
- require exact shell/release verification commands so the boundary stays
  reviewable and repeatable
- define how retroactive validation should classify already-landed second-wave
  adapter files instead of pretending only change-history-derived first-wave
  files matter

## Capabilities

### Modified Capabilities

- `compiler-enforced-functional-core`: adopted no-std domains now include shell
  activation and release evidence, not only attestation and project management
- `std-shell-adapters`: second-wave std adapters now have explicit ownership
  rules and required proof commands
- `workspace-architecture`: the workspace no-std tier expands from the
  first-wave pair to a four-core inventory with named std shell counterparts
- `portability-boundary`: wasm/no-std compilation and dependency-boundary proof
  now applies to second-wave cores too

## Impact

- **Files**: new change-local deltas for `functional-core`, `architecture`, and
  `portability`; later implementation will touch `scripts/check-no-std-core.sh`,
  `scripts/no_std_core_checks.py`, `openspec/specs/functional-core/validation/*`,
  and `openspec/specs/functional-core/evidence/ownership-review.md`
- **APIs**: no broad user-visible runtime API expansion is required; this
  change documents and validates the second-wave core boundary, but it may
  require narrow public API normalization inside `crunch-shell-core` or
  `crunch-release-core` when widened API-shape enforcement rejects existing
  borrowed public surfaces
- **Dependencies**: no new runtime dependencies are intended; later
  implementation may add a checked-in validation inventory file so the runner
  stops hard-coding first-wave-only constants
- **Testing**: later implementation must extend exact host+wasm checks and
  shell-boundary tests to the second-wave cores

## Non-Goals

- broad semantic rewrites of `crunch-shell-core` or `crunch-release-core` as
  part of this change; only narrow checker-driven signature normalization that
  preserves the existing shell/core responsibility split is in scope
- reopening the archived first-wave extraction as if shell/release had been in
  that original scope all along
- adding a third no-std wave for build, store, pipeline, or eval crates
- archiving or force-closing the still-open archived
  `no-std-functional-core` `V5` task as part of this new work
- changing user-visible shell or release CLI behavior as the main goal

## How to Validate

- run `openspec validate second-wave-functional-core-validation`
- confirm the delta specs modify `functional-core`, `architecture`, and
  `portability`
- confirm the delta specs name the second-wave cores, their std shell/adaptor
  files, the exact shell/release boundary tests, and the exact host+wasm
  validation commands
- confirm the proposal/design/tasks describe deterministic ownership review for
  already-landed second-wave adapter files instead of relying only on the old
  first-wave change history
