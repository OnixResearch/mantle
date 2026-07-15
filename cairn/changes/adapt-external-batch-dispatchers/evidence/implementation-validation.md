# External batch dispatcher implementation evidence

## Scope and claim boundary

This evidence covers a provider-neutral canonical dispatch protocol, typed
Nickel profiles, confined direct-process execution, an optional fake-tested
`slurm-cli-v1` translation, durable coordinator allocation state, ordinary
worker registration/assignment, bounded plan/status/composition reports,
immutable digest-bound attempt diagnostics, composition with existing
output-admission facts, and provider-free dispatch of the accepted tracked
hardware-simulation action graph.

It does **not** claim real Slurm-cluster compatibility, exactly-once physical
termination, shared-filesystem authority, production throughput or speedup,
commercial-license behavior, or hardware correctness from elapsed diagnostics.

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
  `src/remote_build.rs`, `src/remote_build/tests/external_batch_hardware_tests.rs`,
  the accepted archived hardware evidence/runtime summary, existing
  resource/transfer/output-admission helpers, focused positive and negative
  fixtures, and the active change specification.
- **Decision:** Yes for the bounded provider-neutral seam. Canonical provider
  state is durable metadata; registration and normal coordinator assignment
  remain mandatory before transfer, and output evidence remains downstream of
  ordinary admission.
- **Owner:** Mantle remote-build coordinator maintainers.
- **Next action:** Run the all-dependent V7 quality and lifecycle rail, record
  exact current outputs, and keep real-provider support as an explicit non-claim.

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

## Hardware workload composition

The prerequisite is accepted at
`cairn/archive/2026-07-14-prove-hardware-simulation-build-flow/`. The V6 rail
loads its immutable `mantle-hardware-evidence-v1` and runtime summary, pins
profile
`mantle-hardware-profile://blake3/29ae88cd1abea071db01d724167876f4caef643344cce36e1d0b6e634ffb5c0f`,
cohort
`mantle-hardware-cohort://blake3/672eb14a9e5494aba76794c35013b656b9d8525d15107ed8ac5f63b0ac78609c`,
and evidence
`mantle-hardware-evidence://blake3/9c581d319c676243a347de04a9da02d3b4f09540fadfa74f0cd4304268adba94`,
and rejects identity or schema drift.

`external_batch_dispatches_tracked_hardware_graph_through_ordinary_cas_and_admission`
dispatches all 13 tracked stage identities through an exact digest-pinned fake
Slurm child process. It proves one generation, nine compile, one link, and two
smoke allocations; CPU/memory/scratch plus `verilator-capacity` projection;
pre-registration transfer rejection; receiver-verified partial locality winning
over a compatible cold worker; BLAKE3 input refs and complete receiver demand;
ordinary fenced signed-`PathInfo` admission and CAS import for every output;
and generic strong action-result admission for every output. Provider completion
is observed only after ordinary output admission. The paired negative test
removes each required hardware non-claim in turn and injects a clean-client
executor call; both drift classes fail closed.

Focused pueue task `1095` reported:

```text
running 3 tests
test remote_build::tests::external_batch_hardware_tests::tracked_hardware_fixture_rejects_non_claim_and_executor_drift ... ok
test remote_build::tests::external_batch_hardware_tests::hardware_named_token_absence_is_ineligible_before_provider_submission ... ok
test remote_build::tests::external_batch_hardware_tests::external_batch_dispatches_tracked_hardware_graph_through_ordinary_cas_and_admission ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1522 filtered out
```

The first V6 run exposed that `admit_external_batch_coordinator_dispatch` checked
the first eligible worker by endpoint order instead of the ordinary ranked
resource/locality selection. The shell now calls `select_coordinator_worker`, so
the external expected worker must be the same worker selected by the normal
policy. The cold-worker fixture is the regression proof.

## V7 final validation

Pueue task `1100` ran the all-dependent focused matrix under the repository Nix
dev shell with `TMPDIR=$PWD/target/pi-tmp`, `CARGO_INCREMENTAL=0`, and
`--option secret-key-files ''`. Its fail-fast chain reached
`V7-FOCUSED-PASSED` with these exact results:

| Surface | Result |
|---|---|
| hardware functional core | 9 passed; 0 failed |
| hardware shell | 8 passed; 0 failed |
| generic action-result core | 6 passed; 0 failed |
| external-batch protocol core | 9 passed; 0 failed |
| resource and locality core | 11 passed; 0 failed |
| resumable transfer core | 11 passed; 0 failed |
| confined dispatcher and fake Slurm process | 5 passed; 0 failed |
| positive/negative Nickel dispatcher contracts | 7 passed; 0 failed |
| coordinator/external/hardware composition | 8 passed; 0 failed |
| scheduling-only resource identity | 1 passed; 0 failed |

The coordinator filter includes all three V6 tests and the earlier five external
allocation/restart/handoff/admission fixtures. The Nickel filter includes typed
round-trip plus type mismatch, shell syntax, secret argument, environment name,
digest, relative program, duplicate instance, and adapter-policy rejection.

Pueue task `1127` passed package-scoped first-party formatting and the checked
strict Clippy command:

```text
cargo clippy --workspace --all-targets --no-deps \
  --exclude fuse-backend-rs --exclude nix-compat \
  --exclude nix-compat-derive --exclude snix-build \
  --exclude snix-castore --exclude snix-store --exclude snix-tracing \
  -- -D warnings
V7-QUALITY-PASSED
```

The only emitted warning was dependency-owned dead code in excluded vendored
`snix-castore`; no first-party warning was accepted.

Pueue task `1128` then passed the repository-compatible Cairn lifecycle chain:

```text
cairn validate --root .
  valid=true; changes=3; specs_validated=30; issues=[]
cairn tracey coverage --root .
  traceability coverage ok: 140/140 referenced (profile mantle-default)
cairn gate proposal adapt-external-batch-dispatchers --root .
  PASS; receipt_hash=5a4474ec789463010d323ab8e42294305cc990336b5669758f65c5915f19b5f8
cairn gate design adapt-external-batch-dispatchers --root .
  PASS; receipt_hash=0e32852e81d404405d9d7148a57662a248bf48a7a05b77a8b7e4b8283f24a61b
cairn gate tasks adapt-external-batch-dispatchers --root .
  PASS; receipt_hash=e789c088f254423958bd84cea3373154426ec04c9350485971fc50dbef607923
V7-LIFECYCLE-PASSED
```

After V7 was checked and this final evidence was assembled, task `1131`
revalidated the completed packet with `valid=true`, `issues=[]`, Tracey
`140/140`, and `PASS` for every gate. Its final recorded receipt hashes were:

```text
proposal 30e7f1cb8e3e748af8672a83165f6d768b7efc0944c37ed1ba7eac0a3d08d431
design   299b28f9b0ad2ec3f972fd2fd7c24ebe422eb8a301ea49217c5a5870b2a7cb24
tasks    c947ce57041a7de211620d5e9638f04e37fff4f0a57d04973a0d05f43848ee47
FINAL-LIFECYCLE-PASSED
```

The broader `check-first-party-quality.sh` was also attempted in task `1108`.
Its required format and strict-Clippy legs passed before its broader workspace
integration leg reached nine `crunch-pipeline` bwrap cases that failed with
`descriptor I/O error: No such file or directory`. Focused task `1124`
reproduced the same environment-level failure on the trivial pipeline test;
task `1126` reproduced it with an existing static BusyBox override. This does
not weaken or relabel those failures, and no broad workspace-integration pass is
claimed. V7 requires the focused distributed/coordinator/resource/transfer/
report matrix plus touched first-party format/lint checks, all of which passed
above.

No real Slurm service, production provider, throughput, commercial-license, or
release-readiness claim is made. The change was not synced, archived, or pushed.
