# Remote build pools

`mantle build --builder-pool NAME` dispatches the ready goals of one native
Linux build across the endpoints of an operator-declared worker pool. This page
describes the contract the current implementation enforces. It is an
experimental surface: every statement below is bounded by the non-claims at the
end.

## Command line

```bash
mantle build TARGET.ncl \
  --builder-pool native-pool \
  --builder-config remote-builders.ncl \
  --builder-ticket-fd worker-a=9 \
  --builder-ticket-fd worker-b=8 \
  --remote-binding-config remote-endpoint-binding.json \
  --store /path/to/store --state-dir /path/to/state
```

- `--builder-config` and at least one `--builder-ticket-fd` are required with
  `--builder-pool`. The ticket flag repeats per endpoint as `ENDPOINT=FD`; each
  FD must be caller-owned and already open in the invoking process.
- `--remote-binding-config` must resolve every selected endpoint's current
  binding by name.
- The clap surface rejects `--builder-pool` with `--builder`, `--watch`,
  `--plan`, `--impure`, `--strict-hermetic`, `--evaluation-stream`,
  `--compile-driver-runtime`, `--rust-wait-diagnostics`,
  `--offline-source-preflight`, and `--fix`. `run_build_command` repeats the
  watch, impure, evaluation-stream, and non-Linux checks for callers that bypass
  the CLI.

## Pool configuration

The configuration is an evaluated Nickel document owned by
`lib/remote-builders.ncl`:

```nickel
{
  pools = [{
    pool_id = "native-pool",
    endpoints = [{
      endpoint_id = "worker-a",
      transport = 'stdio,
      credential_reference = "worker-a-bearer",
      transport_program = "/run/current-system/sw/bin/mantle",
      transport_program_digest_blake3 = "<64 hex chars>",
      transport_args = [
        "--state-dir", "/var/lib/mantle/state",
        "--store", "/var/lib/mantle/store",
        "remote", "serve",
        "--endpoint-id", "worker-a",
        "--binding", "stdio-once",
        "--executor", "local-build",
        "--execution-state-dir", "/var/lib/mantle/worker-a",
      ],
      profile = {
        system = "x86_64-linux",
        sandbox_mode = 'practical,
        network_mode = 'none,
        worker_generation = 1,
        max_concurrency = 1,
        resource_inventory = {
          total = { cpu_units = 4, memory_bytes = 8589934592, scratch_bytes = 10737418240 },
        },
      },
      trusted_output_keys = [{
        public_key_name = "cache.example.com-1",
        public_key_base64 = "<key material>",
      }],
    }],
    session = { max_concurrency = 2, job_count_max = 8, session_bytes_max = 1048576 },
  }],
}
```

Admission rules enforced before dispatch:

- At most 64 endpoints per pool; endpoint ids and credential references must be
  unique across the pool. Each endpoint must declare at least one trusted output
  key, and its key names and key materials must each be unique within that
  endpoint.
- `transport` must be one of `stdio`, `ssh-stdio`, or `p2p`; only `stdio` is
  exercised end to end.
- The session policy must be bounded: `max_concurrency` 1..=1024,
  `job_count_max` 1..=1_000_000, `session_bytes_max` 1..=1 TiB.
- A session with `max_concurrency > 1` requires at least one endpoint to declare
  a resource inventory; legacy endpoints with unknown inventory cannot fit a
  concurrent session.
- Every selected endpoint needs a current binding and an owned ticket
  descriptor. Ticket uses remain per admitted goal under the existing ticket
  policy; a pool selection does not consume one use for the whole session.
- `transport_program` is measured against its declared BLAKE3 identity at
  admission. Nothing is resolved through `PATH`.

## Execution and outputs

Each ready goal travels to one endpoint as the resolved derivation closure plus
the concrete request. Dependencies contribute their built output paths, not
their `.drv` paths. Content-addressed derivations are supported: a resolved
action carries its own action path, and dependent goals resolve realized CA
outputs from the closure.

The receiver rebuilds the closure registry, verifies the request's plan
identity, and executes the action in its local sandbox. The client then:

- validates the signed result against the admitted output keys (full key
  material, not the name alone);
- verifies expected outputs, NAR size and SHA-256, and the declared content
  address before ingesting;
- exports only admitted outputs to the client's physical store and lets the
  ordinary goal/store path persist PathInfo.

## Failures

A failed exchange (transport death, build failure, untrusted key, invalid
output) settles its attempt as a retryable failure and releases the endpoint
concurrency slot before the executor returns. Retry and timeout budgets are the
existing remote attempt policy; pool mode adds no speculative retries and no
autoscaling.

Re-running a goal whose coordinator job already reached a terminal attempt is
not retried by a later invocation; that is the existing attempt-policy
boundary.

## Non-claims

- Same-host `stdio` loopback is the only transport exercised end to end;
  `ssh-stdio` and `p2p` are admitted by configuration but unproven here.
- No cancellation, last-owner teardown, stale-fence, duplicate-result, capacity
  shrink, or worker-death-mid-exchange case is covered by the current pool
  evidence.
- A successful pool build is not a release, self-hosting, distributed
  determinism, or physical-target claim.
