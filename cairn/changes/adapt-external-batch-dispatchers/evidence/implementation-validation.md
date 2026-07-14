# External batch dispatcher implementation evidence

## Scope and claim boundary

This evidence covers a provider-neutral canonical dispatch protocol, typed
Nickel profiles, confined direct-process execution, an optional fake-tested
`slurm-cli-v1` translation, durable coordinator allocation state, ordinary
worker registration/assignment, bounded plan/status/composition reports,
immutable digest-bound attempt diagnostics, and composition with existing
output-admission facts.

It does **not** claim real Slurm-cluster compatibility, exactly-once physical
termination, shared-filesystem authority, production throughput, or hardware
workload composition.

## I1 extension-seam inventory

| Concern | Existing authority | External adapter seam |
|---|---|---|
| Route/build identity | `normalized_remote_build_key` and ordinary remote request validation | Canonical dispatch identity binds the normalized build key without changing it |
| Jobs, attempts, fences | `RemoteCoordinatorState`, `RemoteAttemptState`, `RemoteFenceGeneration` | Allocation records carry separate fenced allocation identities and later bind the ordinary coordinator job |
| Worker authority | `apply_worker_registration` | `register_external_batch_worker` validates expected endpoint/generation before calling ordinary registration |
| Resources/locality | provider-neutral requirements, inventories, fenced resource leases, receiver probes | Canonical projection carries only normalized scheduling quantities and semantic accelerator classes |
| Input/output transfer | receiver-driven resumable CAS transfer | `authorize_external_batch_transfer` stays closed until registration; ordinary assignment then uses existing transfer |
| Output admission | signed output, PathInfo, attestation, and fenced result admission | Scheduler state remains metadata; composition evidence accepts only an already-admitted output report |
| Persistence/restart | clone → validate → persist → publish coordinator mutations | External operation/response, provider locator, worker, coordinator job, lease ref, and bounded reconcile state persist in the same file |
| Typed profiles | `lib/remote-builders.ncl` and `remote_farm_config.rs` | Dispatcher protocol/operation/provider/executable/environment/bootstrap/redaction policy exports deterministically |
| Diagnostics | bounded remote status and explicit non-claims | Status adds safe dispatcher/job/registration/lease/admission facts without raw provider output |

## Review checkpoint

- **Question:** Can an external allocator be added without transferring worker,
  transfer, resource-lease, or output-admission authority to provider state?
- **Inspected evidence:** `crates/crunch-build/src/distributed/external_batch.rs`,
  `src/external_batch_dispatch.rs`, `src/remote_farm_config.rs`,
  `src/remote_build.rs`, existing resource/transfer/output-admission helpers,
  focused positive and negative fixtures, and the active change specification.
- **Decision:** Yes for the bounded provider-neutral seam. Canonical provider
  state is durable metadata; registration and normal coordinator assignment
  remain mandatory before transfer, and output evidence remains downstream of
  ordinary admission.
- **Owner:** Mantle remote-build coordinator maintainers.
- **Next action:** Keep V6 unchecked until its hardware-workload prerequisite
  lands in this checkout; rerun the all-dependent final rail after that merge.

## Incremental checkpoints

- `43754f97` — canonical protocol, typed profiles, direct runner, and optional
  Slurm translation.
- `6e17a91a` — durable allocation lifecycle, worker-registration gate, normal
  coordinator assignment, cancellation, status, and composition evidence.

## Focused validation

Commands are run from the isolated
`adapt-external-batch-dispatchers` worktree. Because `/tmp` was at its user
quota, validation set `TMPDIR=$PWD/target/pi-tmp`; this changes scratch
placement only.

### Focused Rust and Nickel checks

Pueue task `294` exited successfully after running this fail-fast chain:

- `cargo test -p crunch-build external_batch::tests --lib`;
- `cargo test -p mantle --bin mantle external_batch_dispatch::tests -- --nocapture`;
- `cargo test -p mantle --bin mantle batch_dispatcher -- --nocapture`;
- `cargo test -p mantle --bin mantle remote_build::tests::external_batch -- --nocapture`;
- `cargo test -p mantle --bin mantle remote_build::tests::scheduling_quantities_do_not_change_action_key_but_semantic_classes_do -- --exact --nocapture`.

The final visible result was `test result: ok. 1 passed; 0 failed` followed by
`PASSED:resource-action-identity`; because the commands were joined with `&&`,
the task could reach that marker only after the four preceding focused suites
also passed. The external coordinator filter includes the provider-free child
allocation/CAS import fixture and reported five passing focused tests in pueue
task `223`.

### First-party lint/format

Pueue task `285` passed the checked-in `cargo_fmt_first_party` and
`cargo_clippy_first_party` rails. Clippy completed the first-party workspace
with `-D warnings`; the only printed warning was dependency-owned dead code in
vendored `snix-castore`, outside the strict `--no-deps` first-party surface.

### Cairn lifecycle checks

The repository-compatible Cairn binary
`/nix/store/ccwzfd7gk7npii80f7dikvhxvm01dax8-cairn-0.1.0/bin/cairn` was used
because the newer sibling checkout expects a newer generated policy schema.
Pueue task `210` reported `valid: true` with no issues and Tracey coverage
`140/140 referenced`. Proposal, design, and tasks gates passed in fail-fast
pueue task `236`; after dependency markers were normalized to one target per
marker, task `266` reconfirmed lifecycle validation and the tasks gate with
`valid: true`, `issues: []`, and verdict `PASS`.

## Explicit blockers and unchecked work

- **V6:** `prove-hardware-simulation-build-flow` remains an active scaffold in
  this checkout. Its implementation and verification tasks are unchecked, and
  no hardware crate/package exists here to compose honestly.
- **V7:** The all-dependent final rail remains unchecked because V6 is blocked.
