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
- Define one stage0 inventory schema for protected-phase executable seed
  artifacts and protected-phase source artifacts, including every field required
  by the delta spec.

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

### 3. Replace first sandbox launch with a declared seed sandbox executable

**Choice:** this change uses a declared seed sandbox executable with a checked
BLAKE3 digest for the first protected sandbox launch. Native Rust namespace setup
remains a later option, not the first implementation path.

**Rationale:** host `bwrap` is itself a host tool. A declared seed sandbox
executable gives the proof rail a bounded artifact with provenance and digest
without duplicating bwrap behavior in Rust during this change.

### 4. Store the inventory as typed Nickel and validate it in Rust

**Choice:** the source inventory lives at `bootstrap/stage0-inventory.ncl` as a
typed Nickel record. Stage0 evaluates it in-process through crunch's embedded or
source-tree Nickel evaluator path, never by executing an external Nickel binary
or helper command. The evaluated value is deserialized into a Rust
`Stage0Inventory` model. The Rust model is the validation entry point for digest
syntax, role allowlisting, required-entry checks, Nix-command rejection, source
URL declarations, and the canonical inventory digest recorded in proof bundles.

**Rationale:** Nickel gives reviewers a typed checked-in schema while the Rust
model keeps the protected execution core deterministic and unit-testable.

### 5. Provision seed sandbox artifacts explicitly

**Choice:** `sandbox-entry`, `sandbox-shell`, `bootstrap-toolchain-tool`, and
`bootstrap-build-tool` are operator-supplied seed artifacts named by the stage0
inventory. The checked-in
`bootstrap/stage0-inventory.ncl` provides the schema and empty/default policy;
real no-host-tools proof runs pass a concrete inventory path with
`crunch self-build --stage0-inventory <path>` or the proof helper's
`./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory <path>`.
The entries point to absolute artifact paths with BLAKE3 digests, provenance
category, provenance text, owner, and allowed reason. Accepted provenance
categories are `operator-supplied-source-build`, `operator-supplied-bootstrap-seed`,
and `test-fixture`; entries marked `host-path-discovery`, `nix-store-discovery`,
or empty provenance are rejected in no-host-tools mode. Stage0 does not discover
these paths from `PATH` or `/nix/store`.
If any required seed role is absent or its digest mismatches, no-host-tools
mode fails before sandbox startup or build dispatch and records the
missing/mismatched role in the proof audit.

**Rationale:** host-tool-free proof cannot honestly conjure a sandbox binary or
shell from host discovery. The operator owns seed provenance, and crunch owns
validation plus audit recording.

For validation, the proof fixture is a generated inventory under
`target/host-tool-free-stage0/stage0-inventory.ncl`. A checked-in preparation
helper takes explicit seed paths from `CRUNCH_STAGE0_SEED_SANDBOX_ENTRY`,
`CRUNCH_STAGE0_SEED_SANDBOX_SHELL`, and `CRUNCH_STAGE0_SEED_TOOLCHAIN_ROOT`,
computes BLAKE3 digests, writes provenance categories, and fails if any variable
is missing. The helper never searches `PATH` or `/nix/store`; it only records
operator-supplied artifact paths. Full no-host-tools proof validation uses that
generated inventory path, so the external seed dependency is explicit in the
proof evidence.

### 6. Enumerate allowed protected-phase seed roles

**Choice:** protected mode allows only four executable seed roles before
crunch-built tools are selected:

- `sandbox-entry`: a declared sandbox executable used to start the first
  protected build sandbox.
- `sandbox-shell`: a declared static shell/busybox-style executable mounted into
  that sandbox before crunch-built busybox exists.
- `bootstrap-toolchain-tool`: declared compiler, assembler, linker, archive, or
  runtime helper executables required by the seed toolchain.
- `bootstrap-build-tool`: declared build drivers such as a seed `make` needed to
  build the first crunch-owned bootstrap tools.

All other protected-phase work uses Rust-owned logic in the stage0 process.
Source staging, archive creation/extraction, copy operations, vendored-input
checks, digest computation, and helper discovery are not seed executable roles.
They must not invoke `git`, `tar`, `cp`, `cargo`, ad hoc helpers, or host shell
commands. Later use of source-built Rust/Cargo or crunch-built shell tools is
outside the protected phase only after the transition event defined in the spec.

**Rationale:** the first implementation needs a sandbox entry point, shell,
seed toolchain, and minimal build driver to build bootstrap derivations, but
every named host-helper family can be replaced by Rust-owned staging, hashing,
validation, and filesystem code.

### 7. Supervise protected child execs

**Choice:** no-host-tools mode runs the declared seed sandbox under a Linux
seccomp user-notification exec supervisor for this change. The protected sandbox
and descendants inherit a filter that traps `execve`/`execveat` before the kernel
executes the target. The supervisor reads the requested path from tracee memory,
resolves it in the tracee root/mount namespace where possible, checks the target
path and digest against `Stage0Inventory`, appends an audit event, and only then
allows the syscall to continue. It denies and kills/fails the proof on the first
undeclared or mismatched executable. If seccomp user notification, filter
inheritance, or path resolution cannot be enabled deterministically, the mode
fails closed before sandbox startup. Ptrace exec events remain a diagnostic
fallback for tests only, not the enforcement mechanism for this change.

**Rationale:** auditing only crunch-owned launches misses shell/toolchain child
execution inside the first sandbox. The proof claim requires every protected
executable until the bwrap/busybox transition to be validated or rejected.

### 8. Treat Nix commands as permanently forbidden protected-phase seeds

**Choice:** `nix-build`, `nix-store`, `nix-shell`, `nix`, and `nix develop` are
never valid protected-phase seed executables, even if an operator supplies a
matching digest.

**Rationale:** the existing first-bootstrap contract is Nix-free. This change is
a stricter host-tool-free mode, not a path to reintroduce Nix through the seed
inventory.

## Implementation Sketch

1. Introduce a `ProtectedExecPolicy` pure core that classifies executable paths.
2. Define the stage0 inventory schema and make it the single source of truth
   for executable seed artifacts, source artifacts, and protected-phase
   ownership. Executable entries carry `schema_version`, stable `id`, `role`,
   `phase`, absolute `executable_path`, `digest.algorithm`, `digest.hex`,
   optional `digest.interoperability_reason`, `provenance`, `allowed_reason`,
   `owner`, and `required`. Source entries carry `schema_version`, stable `id`,
   `role`, `phase`, `urls`, `digest.algorithm`, `digest.hex`, optional
   `digest.interoperability_reason`, `provenance`, `allowed_reason`, `owner`,
   `required`, and extraction rules. Non-`blake3` algorithms fail unless
   `digest.interoperability_reason` names the external format that requires
   that algorithm.
3. Restrict protected-phase executable seed roles to `sandbox-entry`,
   `sandbox-shell`, `bootstrap-toolchain-tool`, and `bootstrap-build-tool`;
   reject any other executable seed role before execution.
4. Route self-build/proof protected-phase process launches through one
   non-shell protected process-launch adapter that enforces `ProtectedExecPolicy`
   before any crunch-owned `execve`.
5. Run declared seed sandbox children under the exec-audit supervisor so every
   protected child `execve` is validated against the same inventory and audit
   stream until the bwrap/busybox transition.
6. Replace protected-phase `git`, `tar`, `cp`, `cargo`, host `sh`, and helper
   behavior with Rust-owned source staging, archive/copy, vendored-input
   validation, hashing, and declared inventory validation.
7. Add the declared seed sandbox artifact path, digest validation, and
   fail-closed diagnostics for absent or mismatched seed sandbox entries.
8. Extend proof helper with `--no-host-tools` and fake-host-tool tests while
   preserving explicit `self-build-proof: fallback-event=...` markers for
   protected-phase host-tool fallback decisions.
9. Emit `protected-exec-audit.json` into proof bundles with event records for
   executable path, digest, reason, phase, policy decision, and source
   inventory entry.
10. Extend `summary.txt` with the stage0 inventory digest, blocked host command
   set, declared seed sandbox artifact records, fallback-event markers, and
   final result.

## Traceability

| Requirement ID | Design coverage | Validation coverage |
| --- | --- | --- |
| `bootstrap.hosttoolfree.exec.boundary` | Decisions 1, 2, 4, 5, 6, 7, and 8 define protected phase classification, inventory schema, seed roles, Nix rejection, and exec supervision. | Unit tests for policy/inventory validation, fake-PATH runner tests, forbidden seed entries, digest/provenance/source URL checks, and transition checks. |
| `bootstrap.hosttoolfree.sandbox.entrypoint` | Decisions 3, 5, 6, and 7 define declared seed sandbox/shell provisioning, digest validation, and supervised child execution. | Declared seed sandbox acceptance/reporting tests, digest mismatch tests, missing seed fail-closed tests, and host `bwrap` rejection tests. |
| `bootstrap.hosttoolfree.proof.mode` | Decisions 2, 5, 7, and Implementation Sketch items 8–10 define no-host-tools mode, audit events, fallback markers, summary fields, and proof bundle output. | Full no-host-tools self-hosting proof plus hidden-host-tool failure fixture and proof bundle inspection. |

## Risks / Trade-offs

**Exec supervision is Linux-specific and invasive.** Keep it scoped to
no-host-tools proof mode and fail closed when seccomp user notification is not
available.

**Some host tools may be hidden in libraries.** The exec-audit supervisor must
catch child process execution in addition to crunch-owned launches; fake-PATH
tests remain a regression rail for common helpers.

**Strict mode may be Linux-specific.** That is acceptable; bootstrap sandboxing
already targets Linux.

## Validation Plan

- Unit tests for allowed/forbidden executable classification, inventory field
  validation, missing required entries, digest mismatches, non-`blake3` digest
  acceptance only with an interoperability reason, provenance category rejection,
  undeclared source URLs, missing provenance/allowed reason, forbidden Nix seed entries even with
  matching digests, seccomp/supervisor-unavailable fail-closed behavior, and the protected-phase transition after verified
  `bootstrap/bwrap.ncl` plus `bootstrap/busybox.ncl` outputs.
- Runner tests with fake host tools on `PATH`, including `git`, `tar`, `cp`,
  host `sh`, `cargo`, host `bwrap`, and Nix commands.
- Full no-host-tools self-hosting proof that records the inventory digest,
  blocked host command set, declared seed sandbox records, fallback-event
  markers, final result, and stage1/stage2/busybox/bwrap fixed-point status.
- `openspec validate host-tool-free-first-bootstrap --strict`.
