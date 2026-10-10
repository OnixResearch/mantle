# Design: Separate the Rust-plan hexagon

## Context

Rust planning contains pure package and topology policy, Cargo oracle calls, workspace reads, rustc execution, cache access, and presentation in one root module. The target architecture separates deterministic planning from observation and execution.

```text
CLI request
  -> Rust-plan application command
  -> shell obtains bounded workspace and tool facts
  -> Rust-planning core
  -> plan, blockers, receipt preimage, and effect plan
  -> shell executes Cargo, rustc, cache, and filesystem effects
  -> adapters return observations
  -> core classifies results
```

## Decisions

### Decision: create a strict planning core

A new `no_std + alloc` core will own normalized package facts, feature activation, dependency selection, host and target classification, unit topology, action planning, compatibility classification, deterministic identities, blockers, and receipt preimages.

The core will not parse host paths, traverse files, read environment state, invoke tools, access caches, render JSON, or depend on CLI errors.

**Rationale:** Package-planning meaning must replay from supplied facts on any supported host.

### Decision: normalize structural inputs before admission

Filesystem and Cargo adapters will decode manifests, lockfiles, metadata, and unit graphs into bounded structural DTOs. The core will validate and admit those DTOs into nominal package, target, feature, artifact, and toolchain values.

Malformed, duplicate, ambiguous, oversized, and unsupported inputs will return typed blockers before tool execution.

**Rationale:** Parsing bytes is an adapter concern. Semantic admission is a core concern.

### Decision: define narrow application ports

The Rust-plan application will own ports for workspace fact loading, Cargo oracle capture, compiler inspection, unit execution, and Rust cache access. Port contracts will use Mantle-owned requests and results.

The process, filesystem, Cargo, rustc, and store implementations will remain adapters. `RunError` will not appear in port signatures.

### Decision: model unit execution as an effect plan

The core will return ordered bounded unit effects with declared arguments, environment facts, input identities, expected outputs, and limits. The shell will execute each effect and return typed observations for result classification and receipt completion.

**Rationale:** Planning an invocation is not proof that rustc ran or produced the expected artifact.

### Decision: preserve parity through dual-path fixtures

During migration, accepted fixtures will run legacy and extracted planners over identical normalized facts. Results must match package selection, features, unit identities, topology, arguments, environment, blockers, receipts, and output ordering.

Cargo oracle material remains external evidence. It is not hidden core state.

## Error ownership

The core owns package, feature, topology, limit, and unsupported-boundary errors. Ports own capability failures. Adapters retain I/O and process details. The CLI maps typed failures to stable diagnostics and exit codes.

## Testing and evidence

Tests will include valid workspaces, malformed manifests, missing lock data, duplicate packages, target mismatches, feature conflicts, build-script and proc-macro cases, cache faults, rustc failures, hidden environment negatives, parity fixtures, and architecture guards.

Evidence proves deterministic planning over supplied facts and bounded observed execution. It does not prove Cargo equivalence outside the supported subset, rustc correctness, linker correctness, or full ecosystem support.

## Risks

- TOML or Cargo structures can leak into the core. Structural DTOs must remain adapter-owned.
- Unit effects can contain ambient paths. Plans must use declared logical identities and adapter-resolved paths.
- Dual-path tests can preserve accidental behavior. Fixtures must distinguish accepted compatibility from known defects.
