## Why

Mantle's Rust result-cache daemon can restore complete unit results, but a miss
launches a new sandboxed compiler. A developer repeatedly editing one admitted
workspace cannot currently retain a compiler provider's loaded workspace and
query state through that interface. A cache hit, Cargo freshness, or an idle
daemon process is not evidence of warm compiler reuse.

The immediate consumer is the Rust planning/execution maintainer qualifying an
opt-in small-workspace development loop. The outcome is a real retained-provider
edit, compile, check, link and run path with exact revision identities, bounded
ownership and clean-reference comparisons. The durable capability is bounded
resident workspace reuse behind Rust-owned application ports, not a new build
scheduler or store authority.

The conceptual reference is [DioxusLabs/rustd at
131de832c0035b3214289eb65e6a945342180bd8](https://github.com/DioxusLabs/rustd/tree/131de832c0035b3214289eb65e6a945342180bd8),
reviewed 2026-10-15. Its live rust-analyzer database and compiler-owned function
queries are relevant mechanisms, not evidence that its raw protocol, toolchain,
ABI or output format is admitted by Mantle. This proposal imports no code and
makes no claim of compatibility or speedup.

## What Changes

- Admit an explicitly selected development-session provider only with exact
  backend/toolchain/protocol identities, supported fixture/operation rows,
  authoritative-check and enforceable-isolation capabilities. Defaults and the
  existing wrapper protocol remain unchanged.
  r[rust_package_planning.warm_workspace.admission]
- Partition resident state by semantic configuration and apply content-bound,
  ordered revisions with exact base identity. Watcher events and mtimes are
  hints, never evidence that a snapshot is unchanged.
  r[rust_package_planning.warm_workspace.identity]
- Retain actual workspace/query state across successful admitted edits, with
  explicit invalidation, recovery and reset observations. Whole-result restores
  and repeated cold launches cannot satisfy the warm-reuse acceptance criterion.
  r[rust_package_planning.warm_workspace.reuse]
- Keep dependencies, compiler-private fragments and outputs session-owned. A
  runnable current result requires the matching revision's authoritative compiler
  check and successful link/output validation; no stale executable or backend
  success bypasses that gate.
  r[rust_package_planning.warm_workspace.publication]
- Charge idle residents as well as active work to finite enforced memory,
  process, byte, revision and lifetime budgets; retain ownership until bounded
  teardown is observed. Unknown capabilities, errors and exhaustion fail closed.
  r[rust_package_planning.warm_workspace.bounds]
- Qualify same-provider cold/warm behavior separately from any different-provider
  rustc reference, exercise cumulative break/repair and invalidation cases, and
  report complete populations, cold/first/warm costs and resident resources.
  r[rust_package_planning.warm_workspace.qualification]

## Dependencies

Metadata declares `validate-rust-pipelining-equivalence` and
`explain-rust-build-waits` for bounded replay, stateful comparisons, complete
coverage and diagnostic evidence. Those packages remain independently useful
with ordinary compilers. This change does not add a dependency from them or
from either existing Headstart execution package back to warm workspaces.

Provider admission is a separate implementation prerequisite. The reviewed
rustd harness pins `nightly-2026-05-10`; it does not establish a compatible
Mantle provider binary, matched ABI, bounded update protocol or valid deployment
on the current toolchain. Missing prerequisites are blocked, not a silent cold
compiler fallback or a successful unsupported row. Early metadata and a fix for
Headstart's incremental-identity bug are not prerequisites to this independent
development-session feature.

## Impact

- **Owners:** Rust planning/execution maintainers own revision semantics, provider
  qualification and the session adapter; cache maintainers retain complete-result
  admission/publication authority. Provider owners maintain exact upstream,
  patch, protocol, ABI and toolchain compatibility records.
- **Likely files:** `crates/mantle-rust-plan-core`,
  `crates/mantle-rust-plan-app`, `src/rust_plan.rs`, and the existing
  `crunch-rust-cache-core::wrapper` / `crunch-rustc-wrapper` admission shell where
  its capabilities actually fit. Reuse its peer authentication, bounded framing,
  sealed inputs and staging; do not claim V1 already accepts multi-output or
  incremental resident requests.
- **Repeatability evidence:** pinned offline small fixtures, immutable request
  and revision manifests, real cold/warm/reference executions, independently
  expected values/errors, complete case ledgers, retained raw observations and
  enforced-limit/teardown negatives. No full Mantle workspace proof is implied.
- **Compatibility:** opt-in host development evidence remains outside ordinary
  derivation/PathInfo authority. No receipt identity is weakened, no shared
  fragment cache is added, and default Cargo/unit execution remains unchanged.

## Non-Goals

- Adopting rustd or an RA/Cranelift compiler as the default, importing or
  maintaining a rolling compiler fork, or implementing a MIR/codegen backend.
- Implementing Mantle-owned per-function fragments, relocations, ABI lowering,
  persistent/shared fragment CAS, hot patching or incremental linker internals.
  The provider may reuse its own qualified scratch; promoting that into a Mantle
  feature needs a separate current consumer and qualification proposal.
- Changing generic `GoalState`, scheduling policy, ordinary dynamic-plan outputs,
  source-slice limits, cache trust, or remote live-artifact transfer.
- Requiring Headstart, changing its readiness package, or claiming compiler
  correctness, release optimization, full Cargo support, self-hosting, bootstrap
  proof, reproducibility or measured Mantle performance.

## Success Criteria

An admitted real provider retains loaded state across multiple exact edits,
invalidates affected work and produces checked current executables matching the
declared references. Same-provider clean/warm identities and every required
stateful case pass with complete population accounting; compiler rejection,
crash, timeout and protocol failure remain distinct. Idle and active residents
remain inside enforced budgets, and shutdown/error paths prove owned teardown.
Performance reports retain setup, first-edit, failed and unsupported outcomes;
no timing result alone qualifies a provider. This is proposed authoring only:
all implementation tasks remain unchecked and no provider execution is claimed.
