# ADR 0129: Enforce application authority with resolved compiler views and runtime dependency edges

## Status

Proposed (2026-10-23). Implementation is source-complete; integrated verification
and Cairn acceptance remain pending.

## Context

The maintained CLI architecture source rail identifies direct authority paths and
file-local imports. Text alone cannot resolve cross-module re-exports or paths
introduced by macro expansion. A no_std declaration is also insufficient:
explicit `extern crate std` remains legal on a target whose sysroot contains
std. Treating these gaps as review-only would not satisfy the existing automated
non-bypass requirement.

## Decision Drivers

- Resolve aliases, re-exports, macros, and runtime dependency authority instead
  of treating source spelling as semantic evidence.
- Reuse the maintained authority inventory and existing no_std runner.
- Keep toolchain-specific compiler evidence bounded and fail closed on capture,
  compiler, or dependency-graph ambiguity.
- Separate checked authority boundaries from domain correctness and release claims.

## Decision

- Keep `scripts/check-cli-architecture.rs` as the authority inventory and entry
  point. Its `--semantic` mode delegates to the existing
  `scripts/no_std_core_checks.py` runner, passing that same inventory rather than
  defining another Rust-source parser or another authority configuration.
- Use the admitted development shell's current `cargo` and `rustc`; do not invoke
  rustup, install targets, select a different toolchain, or alter dependency pins.
  The integrated gate records those tool identities. Compiler debug output is
  toolchain-specific, not a stable interchange schema.
- For protected first-party workspace `*-core` libraries and
  `mantle-application-contract`, inspect macro-expanded/name-resolved HIR and typed
  HIR from `cargo rustc --lib` with `-Zunpretty=hir-tree` and
  `-Zunpretty=thir-flat`. Indexed typed HIR visits every body without recursively
  reprinting its subtrees; compiler fixtures cross-check its authority findings
  against `thir-tree`. Typed HIR resolves callees that an untyped HIR method
  segment need not identify. Match every compiler `DefId` against the existing
  forbidden authority paths, independently of aliases and macro source spelling.
- Resolve package identities and rename edges with locked, offline Cargo
  metadata. Use Cargo's package-selected normal runtime tree to select active
  edges, rather than attributing unrelated workspace feature unification to each
  core. Reject forbidden runtime dependencies with owner, alias, package identity
  and dependency-chain evidence. Build/dev edges and proc-macro host execution
  are not runtime authority supplied to the protected library.
- Do not expand or re-audit vendored dependency implementations. Pure crypto and
  RNG traits are not forbidden through crate-name substrings; ambient RNG
  entry points and concrete OS entropy remain forbidden authority.
- Compiler fixtures include accepted alloc/core values and pure Duration
  re-exports; rejected cross-module filesystem aliases, macro process/output
  calls, and cross-crate process re-exports; a real renamed/transitive Cargo
  dependency graph; and accepted build-only/proc-macro dependency edges.
- Drain at most 4 GiB of raw HIR/typed-HIR stdout per compiler command while
  retaining at most 256 MiB of compact resolved-definition evidence. Other
  command stdout remains bounded to 256 MiB; stderr and each compiler line stay
  bounded to 8 MiB. The real `crunch-project-core` HIR exceeded the former shared
  256 MiB raw/capture limit, so streamed input and retained evidence now have
  separate limits. No truncation or skipped definitions become success.
- Keep the 600-second command and 3600-second campaign deadlines. Overflow,
  timeout, compiler failure, malformed/unrecognized graph records, ambiguous
  package identities, and missing resolved-definition records are nonpassing.
  Stream and validate every compiler line, retaining every `DefId`, its original
  compiler-view line number and complete raw/line/definition counts. Kill the
  subprocess group on failure and retain bounded failure evidence. Successful
  scratch is removed. Actual-owner compilation uses a private temporary target,
  not the integrator's target artifacts.

## Consequences

The lexical rail remains a cheap topology guard; it is not presented as the
semantic gate. Host/wasm no_std compilation remains a separate requirement.
Both lexical and compiler fixtures and the actual compiler/dependency gate must
run on the final integrated source before acceptance is claimed.

This is bounded enforcement of the declared authority and dependency inventory,
not proof of arbitrary domain-policy ownership, universal purity, external
implementation correctness, every possible feature/target configuration,
sandbox completeness, reproducibility, or release eligibility. A toolchain
change must rerun real positive and negative compiler fixtures before its
compiler-output interpretation is admitted.
