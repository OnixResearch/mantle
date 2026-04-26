## Context

The current proof can hide Nix commands from `PATH`, but the first bootstrap
still relies on host-side command execution such as sandbox launching and host
helper discovery. That is acceptable for seed-assisted proof, but not for a
host-tool-free first bootstrap claim.

The implementation now has a protected execution policy, declared seed sandbox
selection, Linux seccomp user-notification supervision, no-host-tools proof
flags, and proof bundle audit scaffolding. The remaining V4 risk is evidence
honesty: a full no-host-tools proof must not pass by validating host paths,
omitting child-exec events, or using an empty/schema inventory in place of real
seed artifacts.

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
- Make V4 evidence durable enough to show the exact concrete inventory, seed
  entries, protected child-exec events, blocked host tool set, and fixed-point
  status used by the proof run.

**Non-Goals**
- Replacing the current seed provider.
- Claiming operator-supplied seed artifacts are source-built unless their own
  provenance proves that separately.
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
inventory. The checked-in `bootstrap/stage0-inventory.ncl` provides the schema
and empty/default policy; real no-host-tools proof runs pass a concrete
inventory path with `crunch self-build --stage0-inventory <path>` or the proof
helper's `./scripts/prove-self-hosting.sh --no-host-tools --stage0-inventory
<path>`. The entries point to absolute artifact paths with BLAKE3 digests,
provenance category, provenance text, owner, and allowed reason. Accepted
provenance categories are `operator-supplied-source-build`,
`operator-supplied-bootstrap-seed`, and `test-fixture`; entries marked
`host-path-discovery`, `nix-store-discovery`, or empty provenance are rejected in
no-host-tools mode. Stage0 does not discover these paths from `PATH` or
`/nix/store`. If any required seed role is absent or its digest mismatches,
no-host-tools mode fails before sandbox startup or build dispatch and records the
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

**Rationale:** the first implementation needs a sandbox entry point, shell, seed
toolchain, and minimal build driver to build bootstrap derivations, but every
named host-helper family can be replaced by Rust-owned staging, hashing,
validation, and filesystem code.

### 7. Supervise protected child execs in the tracee namespace

**Choice:** no-host-tools mode runs the declared seed sandbox under a Linux
seccomp user-notification exec supervisor for this change. The protected sandbox
and descendants inherit a filter that traps `execve`/`execveat` before the kernel
executes the target. The supervisor reads the requested path from tracee memory,
resolves the executable bytes through the tracee root/mount namespace (for
example `/proc/<pid>/root/bin/sh` for tracee path `/bin/sh`), records both the
tracee path and resolved host path, checks the digest against `Stage0Inventory`,
appends an audit event, and only then allows the syscall to continue. It denies
and kills/fails the proof on the first undeclared or mismatched executable. If
seccomp user notification, filter inheritance, path resolution, or an `execveat`
dirfd/path form cannot be handled deterministically, the mode fails closed before
execution. Ptrace exec events remain a diagnostic fallback for tests only, not
the enforcement mechanism for this change.

**Rationale:** auditing only crunch-owned launches misses shell/toolchain child
execution inside the first sandbox. Hashing host `/bin/sh` for a sandboxed tracee
path `/bin/sh` is wrong evidence. The proof claim requires every protected
executable until the bwrap/busybox transition to be validated or rejected in the
namespace where it will run.

### 8. Treat Nix commands as permanently forbidden protected-phase seeds

**Choice:** `nix-build`, `nix-store`, `nix-shell`, `nix`, and `nix develop` are
never valid protected-phase seed executables, even if an operator supplies a
matching digest.

**Rationale:** the existing first-bootstrap contract is Nix-free. This change is
a stricter host-tool-free mode, not a path to reintroduce Nix through the seed
inventory.

### 9. Promote verified generated outputs into the protected policy

**Choice:** V4 uses verified-output promotion, not direct replacement of
`bootstrap/seed.ncl` with an operator-supplied toolchain root. The initial
inventory declares the sandbox entry, sandbox shell, and operator-supplied seed
inputs. When crunch fetches or builds a protected-phase output such as
`musl-seed-toolchain`, it must validate the output against its declared
source/hash contract, enumerate the executable files that are permitted to run in
the protected phase, compute their BLAKE3 digests, append promotion records to
the audit, and update the supervisor policy before any child `execve` can run
those paths.

**Rationale:** the current bootstrap files import `bootstrap/seed.ncl`, which
materializes a `musl-seed-toolchain` store output. Consuming an operator root
directly would create a second bootstrap path that drifts from the normal proof.
Verified-output promotion keeps the existing bootstrap graph while making the
trust transition explicit and reviewable.

### 10. Make protected source extraction rules typed enough to audit

**Choice:** source inventory entries use ordered, non-empty extraction-rule
strings with a checked `key=value` grammar. The initial accepted keys are
`format` (`tar`, `tar.gz`, `tar.xz`, or `plain`), `strip-components` (base-10
u32), and `root` (relative output root without `..`). Additional keys require a
new spec update and validation test. Promotion records must reference the source
entry and extraction rules that produced the promoted executable set.

**Rationale:** a source URL and digest alone do not explain how bytes become the
filesystem tree whose executables are later promoted. A small typed grammar keeps
proof evidence deterministic without inventing a full archive policy language.

### 11. Make proof evidence durable, not inferred

**Choice:** successful V4 proof bundles must copy the concrete stage0 inventory
into the bundle, hash the copied file, include declared seed entries, include
actual seccomp supervisor event records, summarize blocked host commands, record
fallback markers, and record fixed-point status. The audit result must be
derived from the observed proof outcome; a hard-coded success string is not V4
evidence by itself.

**Rationale:** reviewers must be able to inspect the bundle without relying on
ambient operator paths, hidden local files, or a success field written before the
proof outcome is known.

## Implementation Sketch

1. Introduce a `ProtectedExecPolicy` pure core that classifies executable paths.
2. Define the stage0 inventory schema and make it the single source of truth for
   executable seed artifacts, source artifacts, and protected-phase ownership.
3. Restrict protected-phase executable seed roles to `sandbox-entry`,
   `sandbox-shell`, `bootstrap-toolchain-tool`, and `bootstrap-build-tool`;
   reject any other executable seed role before execution.
4. Route self-build/proof protected-phase process launches through one non-shell
   protected process-launch adapter that enforces `ProtectedExecPolicy` before
   any crunch-owned `execve`.
5. Run declared seed sandbox children under the exec-audit supervisor so every
   protected child `execve` is validated against the same inventory and audit
   stream until the bwrap/busybox transition.
6. Resolve seccomp-trapped executable paths through the tracee root/mount
   namespace, record tracee path plus resolved host path, and fail closed for
   unsupported `execveat` cases.
7. Copy the concrete `stage0-inventory.ncl` used by no-host-tools mode into the
   proof bundle and hash the copied file.
8. Thread actual `ProtectedSeccompAuditEvent` records from the stage0 supervisor
   into `SelfBuildReport` and the proof bundle.
9. Emit declared seed entries (id, role, phase, path, digest, provenance,
   required flag) into `protected-exec-audit.json`.
10. Implement verified-output promotion for protected-phase fetched/built
   outputs before they execute.
11. Validate the protected source extraction-rule grammar and include the rules in
   promotion audit records.
12. Add an operator-facing inventory generation/preflight command or helper that
   only consumes explicit seed paths and never searches `PATH` or `/nix/store`.
13. Run the full ignored proof through `./scripts/prove-self-hosting.sh
   --no-host-tools --stage0-inventory <concrete-inventory>` only after the
   evidence hardening lands.

## Traceability

| Requirement ID | Design coverage | Validation coverage |
| --- | --- | --- |
| `bootstrap.hosttoolfree.exec.boundary` | Decisions 1, 2, 4, 5, 6, 7, 8, 9, and 10 define protected phase classification, inventory schema, seed roles, Nix rejection, namespace-aware exec supervision, and generated-output transition rules. | Unit tests for policy/inventory validation, fake-PATH runner tests, forbidden seed entries, digest/provenance/source URL checks, namespace path-resolution tests, and transition checks. |
| `bootstrap.hosttoolfree.sandbox.entrypoint` | Decisions 3, 5, 6, 7, and 9 define declared seed sandbox/shell provisioning, digest validation, supervised child execution, and verified promotion or direct consumption of seed toolchain executables. | Declared seed sandbox acceptance/reporting tests, digest mismatch tests, missing seed fail-closed tests, host `bwrap` rejection tests, and full no-host-tools proof. |
| `bootstrap.hosttoolfree.proof.mode` | Decisions 2, 5, 7, 9, 10, and 11 define no-host-tools mode, audit events, inventory copying, fallback markers, summary fields, fixed-point evidence, and proof bundle output. | Full no-host-tools self-hosting proof plus hidden-host-tool failure fixture and proof bundle inspection. |

## Risks / Trade-offs

**Exec supervision is Linux-specific and invasive.** Keep it scoped to
no-host-tools proof mode and fail closed when seccomp user notification is not
available.

**Tracee path resolution can be subtle.** Absolute paths must be resolved via the
tracee root/mount namespace, and unsupported `execveat` dirfd cases should fail
closed instead of guessing.

**Dynamic bootstrap outputs need explicit trust transition.** The proof cannot
predeclare paths and digests for every executable generated by derivations unless
it either consumes an operator seed root directly or promotes verified outputs
into the policy at a documented boundary.

**Some host tools may be hidden in libraries.** The exec-audit supervisor must
catch child process execution in addition to crunch-owned launches; fake-PATH
tests remain a regression rail for common helpers.

**Strict mode may be Linux-specific.** That is acceptable; bootstrap sandboxing
already targets Linux.

## Validation Plan

- Unit tests for allowed/forbidden executable classification, inventory field
  validation, missing required entries, digest mismatches, non-`blake3` digest
  acceptance only with an interoperability reason, provenance category rejection,
  undeclared source URLs, missing provenance/allowed reason, forbidden Nix seed
  entries even with matching digests, seccomp/supervisor-unavailable fail-closed
  behavior, tracee namespace path resolution, and the protected-phase transition
  after verified `bootstrap/bwrap.ncl` plus `bootstrap/busybox.ncl` outputs.
- Runner tests with fake host tools on `PATH`, including `git`, `tar`, `cp`,
  host `sh`, `cargo`, host `bwrap`, and Nix commands.
- Proof bundle tests showing copied concrete inventory, declared seed records,
  actual supervisor events, blocked command set, and derived result.
- Full no-host-tools self-hosting proof using a concrete generated inventory and
  explicit seed strategy.
- `openspec validate host-tool-free-first-bootstrap --strict`.
