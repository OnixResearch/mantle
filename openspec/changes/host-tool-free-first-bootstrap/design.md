## Context

The current proof can hide Nix commands from `PATH`, but the first bootstrap
still relies on host-side command execution such as sandbox launching and host
helper discovery. That is acceptable for seed-assisted proof, but not for a
host-tool-free first bootstrap claim.

## Goals / Non-Goals

**Goals**
- Define a protected phase from stage0 crunch start until crunch-built tools are
  available.
- Forbid undeclared host executable use during that phase.
- Replace host `bwrap` fallback with a native or declared-seed sandbox path.
- Produce an execution audit in proof bundles.

**Non-Goals**
- Replacing the current seed provider.
- Removing the Linux kernel as a trusted execution substrate.
- Changing normal developer-mode builds outside explicit strict/proof modes.

## Decisions

### 1. Make executable use auditable before making it impossible

**Choice:** add a protected execution wrapper that records and validates every
pre-toolchain executable path.

**Rationale:** this creates a deterministic failure rail and exposes remaining
host-tool dependencies before deeper refactors land.

**Alternative:** rely on a poisoned `PATH` only.

**Why not:** absolute path execution can bypass `PATH`; the audit must live at
crunch's execution boundary.

### 2. Separate host-tool-free mode from practical developer mode

**Choice:** keep practical mode for day-to-day development, and add explicit
no-host-tools proof/strict mode for the stronger claim.

**Rationale:** first-bootstrap research should not make normal local iteration
fragile while the protected path is under construction.

**Alternative:** make all self-builds host-tool-free immediately.

**Why not:** that creates a large compatibility cliff and makes incremental
validation harder.

### 3. Replace first sandbox launch with native or declared seed execution

**Choice:** protected mode may use Rust-owned namespace setup or a declared seed
sandbox executable, but not host `bwrap` fallback.

**Rationale:** host `bwrap` is itself a host tool. If it remains, the claim is
not host-tool-free.

## Implementation Sketch

1. Introduce a `ProtectedExecPolicy` pure core that classifies executable paths.
2. Route self-build/proof protected-phase process launches through one shell
   adapter that enforces that policy.
3. Add a native sandbox prototype or declared seed sandbox artifact path.
4. Extend proof helper with `--no-host-tools` and fake-host-tool tests.
5. Emit `protected-exec-audit.json` into proof bundles.

## Risks / Trade-offs

**Native sandboxing duplicates bwrap behavior.** Keep the native entry point
minimal and bootstrap-only, or use a declared seed bwrap with a digest.

**Some host tools may be hidden in libraries.** The audit catches crunch-owned
process launches; tests should also poison PATH for common helpers.

**Strict mode may be Linux-specific.** That is acceptable; bootstrap sandboxing
already targets Linux.

## Validation Plan

- Unit tests for allowed/forbidden executable classification.
- Runner tests with fake host tools on `PATH`.
- Full no-host-tools self-hosting proof.
- `openspec validate host-tool-free-first-bootstrap --strict`.
