# ADR 0070: Bind the source fixed point to a root action trust report

## Status

Proposed (2026-08-01)

## Context

Mantle has two useful evidence levels. StageX records detailed executable authorization and protected execution events. The source-built fixed-point plan records six broad proof stages.

No single pre-execution artifact lists every action reachable from the selected proof root. The six-stage plan can therefore hide an incomplete native-provider or Rust-unit adapter until a long proof runs.

`fzakaria/stage0-bazel` demonstrates a cheap root action review before execution. Mantle adopts this report shape only. It does not adopt Bazel, path-prefix trust, host-shell exceptions, test exceptions, or SHA-256 for Mantle-owned identities.

## Decision Drivers

- Reject incomplete action authority before a long proof starts.
- Keep StageX seccomp enforcement as runtime authority.
- Bind generated executables to producer actions and content identities.
- Reuse the existing fixed-point plan, stage evidence, and v2 receipt.
- Preserve local-only, no-fetch, no-fallback proof policy.
- Give operators one clear planned-versus-observed summary.

## Decision

Mantle adds a deterministic root-scoped action trust plan to the source-built fixed-point proof.

The plan enumerates every reachable action. Each action records its broad proof stage, producer actions, input authorities, outputs, execution locality, event-count bounds, and resource limits.

Each permitted executable uses one of two authority forms:

1. A fixed executable carries a reviewed BLAKE3 identity.
2. A produced executable carries its producer action and output identity. Mantle binds its observed BLAKE3 before execution.

A path, store prefix, output directory, executable name, or generated-file location cannot grant execution authority by itself.

The proof fails before construction when any action adapter or authority relation is incomplete. Remote execution and cache-only completion are forbidden for this proof.

Produced toolchains create a planning dependency. Provider stages therefore emit complete stage-local plans before their own execution. Promoted checkpoints bind those plans. A restored attempt composes them into the root plan before any current build action. Post-execution synthesis remains forbidden.

For stage1 and stage2, the fixed-authority file exists before native Rust planning. Planning reads the bound rustc identity without executing rustc. The worker writes its unit action plan before seccomp installation. Compile scopes include rustc and nested fixed tools. Build-script scopes use the compile action and declared output identity to promote measured executable bytes before launch.

Full-source Rust-provider stages use stage-local plans. Each plan names its authenticated script inputs, predecessor action, fixed host/native executables, bounded output-tree authorities, resources, and event limit. Fixed native authority includes explicit GCC driver backends and generic binutils paths from the admitted provider; a wrapper path does not authorize an unnamed backend. GCC subprogram discovery uses the receipt-derived `-B<provider>/libexec/gcc/x86_64-unknown-linux-musl/10.5.0/` option in C, C++, linker, and rustc linker arguments. Ambient `GCC_EXEC_PREFIX` and `COMPILER_PATH` remain excluded. Parent-component exec paths remain denied. An output path alone grants nothing. While the declared producer action is active, the seccomp supervisor can bind the first execution under that exact output tree to the planned output identity and observed BLAKE3. It pins those bytes before it permits the kernel launch. Later byte drift fails closed. Each stage writes its raw audit before reconciliation, including on failure.

After execution, Mantle reconciles protected-exec and build execution records with the plan. It rejects unknown events, missing required events, digest drift, producer drift, count-bound violations, undeclared inputs, and locality drift.

The v2 proof receipt binds the action plan digest and observed reconciliation digest. Bootstrap promotion independently recomputes these links. `mantle --json bootstrap trust-report --proof-root <path>` renders the bound result. The command is a view, not a new authority source.

## Alternatives Considered

### Keep only the six-stage summary

Rejected because broad stages do not expose every child tool, generated executable, or producer edge before execution.

### Trust every executable under a generated path

Rejected because a path does not prove which action produced the file or which bytes it contains.

### Use only the seccomp audit

Rejected because runtime interception cannot provide the same cheap, complete review before a long proof starts.

### Add an independent report database

Rejected because it would create a second evidence authority. The report must derive from the existing plan, observations, and receipt links.

## Consequences

- Native-provider and Rust-unit planners need complete action adapters.
- Missing adapters block the proof instead of producing a partial success report.
- Generated build scripts need producer-linked authority before execution.
- Rust unit plans distinguish fixed toolchain executables from producer-linked build-script executables.
- Receipt-bound tool aliases and compatibility probes must use the source-built BusyBox shell bound by the Rust-provider receipt. Ambient `/bin/sh` cannot authorize them.
- Action and event collections require deterministic order and explicit limits.
- The report improves review and diagnostics without proving compiler correctness, seed correctness, kernel isolation, or independent reproducibility.
- The operator view revalidates the fixed-point receipt before rendering. If the receipt does not bind both action files, it reports `fixed-point-only` and explicit blockers.
- The view cannot create missing authority. A later proof must bind the root action plan and reconciliation before it can emit the complete claim.
