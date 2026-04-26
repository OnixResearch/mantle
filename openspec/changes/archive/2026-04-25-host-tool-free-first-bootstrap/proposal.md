# Host-tool-free first bootstrap

## Why

The current first bootstrap path is explicit about its host prerequisites, but
it still depends on host-side tools and discovery before crunch-built tools
exist. That leaves a gap between "Nix-free" and "host-tool-free". To make the
first bootstrap reviewable, crunch should stop executing undeclared host tools
once the stage0 crunch binary starts, and should fail closed if a required seed
artifact is missing.

## What Changes

- **Define the host-tool-free boundary.** State exactly which kernel interfaces
  and predeclared seed artifacts remain allowed before crunch builds its own
  tools.
- **Define the stage0 inventory schema.** Make executable path, digest,
  provenance, allowed reason, and protected-phase ownership explicit so the
  audit has one source of truth.
- **Remove host command execution.** Replace host `bwrap`, shell, copy, tar,
  git, cargo, and helper invocations in the first-bootstrap path with Rust-owned
  logic or declared seed artifacts.
- **Add execution auditing.** Record every pre-toolchain executable path used by
  stage0 and fail if an undeclared host path is executed.
- **Harden proof mode.** Extend `--non-nix-host` into a broader no-host-tools
  proof rail.
- **Make V4 evidence honest.** Require namespace-aware exec digesting, a bundled
  concrete inventory, actual protected child-exec events, declared seed records,
  an explicit seed-toolchain strategy, and a derived proof result before the full
  no-host-tools proof can be claimed.

## Non-Goals

- Replacing the current binary seed provider; that is handled by
  `full-source-bootstrap-root`.
- Removing the need for a Linux kernel with namespaces and filesystem support.
- Changing normal non-bootstrap `crunch build` ergonomics.

## Capabilities

### New Capabilities
- `bootstrap-host-tool-free-stage0`: perform first bootstrap without executing
  undeclared host commands.
- `bootstrap-stage0-exec-audit`: produce machine-readable proof of every
  executable used before crunch-built tools are available.

### Modified Capabilities
- `prove-self-hosting`: add a stronger proof mode that blocks broad host-tool
  fallback, not only Nix commands.

## Impact

- **Files**: `src/self_build.rs`, `src/protected_exec_seccomp.rs`,
  `scripts/prove-self-hosting.sh`, sandbox launch code, bootstrap docs,
  self-hosting tests.
- **APIs**: new strict proof flag or mode, plus report fields for stage0 execs.
- **Dependencies**: may add internal Rust syscall/filesystem code, but not new
  host command dependencies.
- **Testing**: fake-PATH tests, proof-helper branch tests, and an ignored full
  proof in no-host-tools mode.

## Relationship to Other Changes

This change controls host tool execution. It complements
`full-source-bootstrap-root`, which controls the seed provenance.

## How to validate

1. `openspec validate host-tool-free-first-bootstrap --strict` passes.
2. Unit tests prove stage0 rejects undeclared executable paths.
3. Runner tests prove fake `git`, `tar`, `cp`, `sh`, `cargo`, `bwrap`, and Nix
   commands are not invoked during the protected phase.
4. Proof bundle tests show copied concrete inventory, declared seed records,
   actual protected child-exec events, blocked host command set, and a derived
   final result.
5. The full self-hosting proof passes in the no-host-tools mode using a concrete
   stage0 inventory and explicit seed-toolchain strategy.
