# External batch dispatchers

Mantle can allocate remote workers through a bounded external dispatcher while
keeping ordinary coordinator assignment, transfer, and signed-output admission
authoritative.

The canonical control protocol is
`mantle-batch-dispatch-adapter-v1`. Submit, observe, cancel, and reconcile
messages carry BLAKE3 operation and dispatch identities, adapter generation,
job/attempt/fence identities, the expected worker endpoint, provider-neutral
resource quantities, and explicit lifecycle limits. They never carry tickets,
credentials, signing keys, or policy secrets.

## Lifecycle

1. The coordinator normalizes a build request and projects its CPU, memory,
   scratch, accelerator, and named-token requirements.
2. The selected dispatcher submits one idempotent external allocation. Mantle
   persists only bounded canonical state and the external job identifier.
3. The allocated worker must register its endpoint, generation, capabilities,
   resource inventory, and output-signing key IDs before input transfer or
   coordinator assignment is authorized.
4. The registered worker enters the ordinary coordinator placement path. CAS
   transfer, fenced attempt reports, and output admission are unchanged.
5. Observe and restart reconciliation reattach to current allocations. Terminal
   or unknown allocations are retried only within the configured attempt and
   deadline bounds. Stale fences and conflicting external job IDs fail closed.
6. Cancellation or timeout invokes the external cancel operation and marks the
   ordinary coordinator attempt lost. A stale local attempt still cannot mutate
   the current fenced attempt.

External scheduler state is allocation metadata only. It does **not** authorize
workers, transfer output trust, prove execution success, or admit outputs.

## Typed Nickel profile

Dispatcher profiles live beside remote builder pools in
`lib/remote-builders.ncl`:

```nickel
let remote = import "remote-builders.ncl" in
let command = fun program digest => {
  program,
  expected_digest_blake3 = digest,
  args = [],
  timeout_secs = 30,
  stdout_limit_bytes = 4096,
  stderr_limit_bytes = 4096,
} in
let sbatch_digest =
  "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" in
let squeue_digest =
  "123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0" in
let scancel_digest =
  "23456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef01" in
{
  pools = [],
  dispatchers = [({
    instance_id = "slurm-production",
    generation = 1,
    adapter = 'slurm-cli-v1,
    protocol_schema = "mantle-batch-dispatch-adapter-v1",
    allowed_operations = ['submit, 'observe, 'cancel, 'reconcile],
    provider_class = "slurm",
    submit = command "/opt/slurm/bin/sbatch" sbatch_digest,
    observe = command "/opt/slurm/bin/squeue" squeue_digest,
    cancel = command "/opt/slurm/bin/scancel" scancel_digest,
    reconcile = command "/opt/slurm/bin/squeue" squeue_digest,
    worker_program = "/opt/mantle/bin/remote-worker-launcher",
    worker_program_digest_blake3 =
      "3456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef012",
    worker_args = [],
    environment_handles = [{ name = "MANTLE_BATCH_HANDLE_SITE_AUTH" }],
    bootstrap_policy = 'fixed-template-v1,
    redact_provider_output = true,
    startup_timeout_secs = 300,
    terminal_timeout_secs = 86400,
    max_reconcile_attempts = 8,
  } | remote.BatchDispatcher)],
}
```

Each executable path must be absolute. Mantle verifies its configured BLAKE3
digest before every operation, clears the inherited environment, passes
arguments directly without a shell, bounds stdout/stderr, and kills timed-out
children. Static command and worker arguments containing secret markers are
rejected. Rotate an executable by changing its digest and increasing the
profile generation.

The command helper above keeps each operation's executable identity explicit.

## Adapter behavior

### `direct-process-v1`

The adapter writes one canonical request JSON object to stdin and requires one
canonical response JSON object on stdout. Human diagnostics may use stderr but
are bounded and are not persisted as protocol state. Unknown fields, malformed
JSON, identity drift, stale generations/fences, and observation-time drift are
rejected.

### `slurm-cli-v1`

The optional Slurm adapter executes configured `sbatch`, `squeue`, `sacct`,
and `scancel` binaries directly. It projects resource facts only at this adapter
boundary:

- CPU to `--cpus-per-task`;
- memory to `--mem` in bytes;
- scratch to `--tmp`;
- accelerators to `--gres=name:quantity`;
- named scheduling tokens to `--licenses=name:quantity`.

Submission uses `--parsable`; observation requests `%T`. Only bounded Slurm job
identifiers and known state labels are accepted. Empty observation output is
`unknown`; malformed non-empty state output fails closed. The adapter appends
non-secret dispatch, endpoint, generation, and fence facts to the configured
worker launcher.

## Diagnostics and evidence

Remote plan and status reports include bounded external attempt summaries:
dispatch/profile IDs, provider class, adapter generation, allocation
job/attempt/fence, external job ID, scheduler state, canonical resources,
expected worker, registration state, reconcile count, and bounded reason code.
The immutable attempt log receives a digest-bound diagnostic record rather than
raw provider output. Raw stdout/stderr, environment values, and configuration
secrets are not persisted.

Composition/build evidence links the external dispatch and profile to the
registered worker, ordinary coordinator attempt, resource lease, transfer
mode, trust-key ID, and already-admitted output digest. The evidence also
records that scheduler identity does not transfer worker or output authority.
Existing fenced reports, signed `PathInfo`, artifact-attestation, and
output-import checks remain the only output-admission authority.

## Hardware workload composition fixture

The provider-free validation rail consumes the accepted immutable evidence from
`cairn/archive/2026-07-14-prove-hardware-simulation-build-flow/` and binds its
exact profile, cohort, sources, and 13 action identities to the generic external
batch path. The tracked graph contains one generation action, nine compile
actions, one link action, and two smoke actions.

For every tracked action, the fixture:

1. projects four CPU units, bounded memory and scratch bytes, and one opaque
   `verilator-capacity` scheduling token into the digest-pinned fake Slurm
   process;
2. rejects transfer before the expected worker registers under the current
   dispatcher generation and fence;
3. records receiver-verified partial CAS locality and proves normal placement
   selects that worker over an otherwise compatible cold worker;
4. sends only BLAKE3 input refs through the ordinary framed upload path and
   proves a complete receiver manifest has no missing bytes;
5. admits one signed `PathInfo` output through the ordinary fenced result path,
   imports it through the existing CAS/store seam, and then admits a generic
   strong action-result record; and
6. observes provider completion only after output admission, so the external
   job identifier and queue state remain non-authoritative metadata.

The rail also fails closed if the archived stage counts, exact profile/cohort
identities, clean-client shared-result facts, zero-executor-call observation, or
required hardware non-claims drift. This fixture proves deterministic
composition with a fake provider process. It does not claim real-cluster
compatibility, production throughput or speedup, commercial-license behavior,
hardware correctness from elapsed time, or release eligibility.
