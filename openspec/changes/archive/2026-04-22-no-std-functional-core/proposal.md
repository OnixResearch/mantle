# no-std functional core

## Why

Crunch talks about a portable, functional core, but compiler does not enforce
that boundary yet. Much of today’s pure logic still lives in `std` crates, so a
future edit can quietly pull in filesystem, process, environment, clock, or
network dependencies without tripping any structural guard.

That weakens two goals at once:

- Tiger Style functional-core / imperative-shell separation stays partly social
  instead of compiler-enforced.
- portability claims stay weaker than they need to be because “OS-agnostic”
  code can still depend on `std` ambient facilities.

Making the core `#![no_std]` is the first serious basis for a real functional
core. It forces effectful concerns into explicit shell layers and gives crunch a
portable substrate for later evaluator, planner, and store-boundary work.

## What Changes

- define a compiler-enforced no-std core tier for crunch using dedicated
  `#![no_std]` + `alloc` crates
- introduce a new functional-core spec that says core code accepts plain data,
  returns plans/results as data, and performs no filesystem/process/env/time
  effects
- update architecture and portability specs so the workspace distinguishes
  no-std core crates from std shell/adaptor crates
- scope the first extraction wave to specific high-value pure modules instead
  of trying to rewrite the whole workspace in one pass
- require no-std target checks and dependency-boundary checks so the boundary
  stays honest after the first extraction lands

## Capabilities

### New Capabilities

- `compiler-enforced-functional-core`: core logic is protected from ambient
  OS/runtime dependencies by crate boundaries, not convention alone
- `no-std-core-compilation`: selected core crates compile on a `no_std` target
  with `alloc`
- `std-shell-adapters`: effectful crates own file I/O, subprocesses, clocks,
  environment reads, and network access, then translate those inputs into plain
  core values

### Modified Capabilities

- `workspace-architecture`: crate layout now carries an explicit no-std core /
  std shell split
- `portability-boundary`: portable crunch logic is defined more strongly than
  “OS-agnostic”; it is compiler-checked against `std` leakage

## Impact

- **Files**: `Cargo.toml`, new `crates/*-core/` crates, existing pure/mixed
  crates such as `crunch-attestation` and `crunch-project`, plus shell callers
  in the binary crate and other std adapters
- **APIs**: mixed crates may gain thinner std adapters or re-exports over new
  core crates; shell-facing code will translate path/process/env/time data into
  plain core inputs
- **Dependencies**: selected core crates must use `no_std`-compatible
  dependencies or wrapper types; std-only dependencies stay in shell crates
- **Testing**: validation adds `no_std` target checks, dependency-boundary
  checks, and positive/negative tests for pure core logic and shell translation

## Non-Goals

- rewriting the whole workspace to `#![no_std]` in one change
- making build execution, store mutation, or CLI orchestration themselves
  `no_std`
- forcing `crunch-eval`, `crunch-build`, `crunch-store`, or the binary crate to
  become fully no-std immediately
- changing user-visible CLI behavior as the main goal of this change
- replacing clear crate boundaries with a large matrix of `std` feature flags in
  every existing crate

## How to Validate

- run `openspec validate no-std-functional-core`
- confirm the change defines delta specs for `architecture`, `portability`, and
  new `functional-core` domain
- confirm the delta specs include requirement/scenario `ID:` lines throughout
- confirm the specs name the first-wave core crates, the concrete module scope,
  the ownership/API-shape/shell-translation proof mechanisms, and the exact
  no-std/dependency-boundary validation commands
