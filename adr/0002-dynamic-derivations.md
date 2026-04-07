# ADR 0002: Dynamic Derivations

## Status

Accepted

## Context

crunch's spec requires support for dynamic derivations: builds that
produce `.drv` files as outputs, which are then built in turn. ADR 0001
chose the lazy goal scheduler explicitly to enable this — `want()` is
callable mid-run, KnownPaths grows during builds, and the goal registry
accepts new entries after dispatch starts.

The question: how to wire up the detection, parsing, and scheduling of
dynamically-discovered derivations without changing the fundamental
Worker/Goal architecture.

## Decision

Implement dynamic derivations via **post-build output inspection** in
the Worker, with no changes to the Nickel schema or convert pipeline.

### Detection

After every build completion, `detect_dynamic_derivations()` scans the
build's output PathInfos. An output is a candidate if:

1. The store path name ends with `.drv`
2. The output node is a regular file (not dir/symlink)
3. The file is under 4 MiB

### Parsing

Candidate outputs are read from the castore blob service via
`Builder::read_blob()`. Content is checked for the ATerm prefix
(`Derive(`) and parsed with `Derivation::from_aterm_bytes()`.

### Registration

Parsed derivations are registered in KnownPaths via
`register_dynamic_drv()`, which computes the ATerm hash, HDM, and
derivation path. If a parent derivation referenced by the dynamic drv
isn't in KnownPaths, its HDM falls back to zeros with a warning — the
build still works, the output path just won't match what Nix computes.

### Scheduling

Two paths depending on whether a goal was pre-created:

1. **AwaitingDerivation goals**: If a goal exists in the
   `AwaitingDerivation` state with a matching `producer_key`, the
   parsed derivation is injected via `set_derivation()`, transitioning
   to `Pending`. The goal is then inspected and wired normally.

2. **Auto-discovered goals**: If no awaiting goal exists, a new root
   goal is created via `want()`. The Worker's dispatch loop picks it
   up on the next iteration.

### Goal State Machine Extension

One new state: `AwaitingDerivation`. A goal in this state has no
`Derivation` yet — it's waiting for a producer build to complete.

```
AwaitingDerivation → Pending  (set_derivation called)
AwaitingDerivation → Failed   (producer failed)
```

The `Goal::new_awaiting(drv_path, producer_key)` constructor creates
goals in this state. The `producer_key` field links the goal to its
producer for lookup after build completion.

### Module Structure

New `dynamic.rs` in crunch-build: pure detection and parsing logic.
No async, no I/O — the Worker handles blob reads and goal mutations.
FCIS boundary maintained.

## Alternatives Considered

### Nickel schema changes

Add `Input::Dynamic(Box<CrunchDerivation>)` to the Nickel type system,
letting users explicitly declare producers at eval time.

Pro: Type-safe, eval-time visibility.
Con: Premature — we don't yet know the right Nickel API for this. The
post-build approach works today without forcing a schema commitment.
Can be added later as sugar on top.

### Full IFD (Import From Derivation)

Pause Nickel evaluation, build a derivation, import its output into
the evaluator, and continue.

Pro: Most powerful — matches Nix's `import (derivation { ... })`.
Con: Requires deep Nickel runtime integration (eval suspension/
resumption). Much larger scope. The post-build approach handles the
common case (build → discover → build) without eval changes.

### Content-based detection only

Check output bytes for ATerm prefix without requiring `.drv` name.

Pro: Works regardless of naming.
Con: False positives on files that happen to start with `Derive(`.
Name-based check is cheap and matches Nix convention.

## Consequences

- 201 tests total in crunch-build (was 198). 14 tests cover dynamic
  derivation detection, parsing, registration, goal lifecycle, and
  Worker integration.
- The Worker loop is slightly longer per build completion (output
  scan). The scan is O(outputs) per build — negligible.
- KnownPaths grows during builds. Already supported by the lazy goal
  architecture (ADR 0001).
- No Nickel schema changes needed. Users produce `.drv` files as
  build outputs; crunch detects and builds them automatically.
- The `AwaitingDerivation` state is available for future use when
  explicit pre-declaration of dynamic deps is wanted.
