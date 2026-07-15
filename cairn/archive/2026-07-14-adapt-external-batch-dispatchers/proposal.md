## Why

Mantle can plan and execute remote builds through provider-neutral worker/coordinator seams, and the active resource work models CPU, memory, scratch, accelerators, locality, and named scarce token pools. It still lacks a stable boundary for handing an eligible fenced attempt to an existing batch scheduler such as Slurm or an operator-owned LSF bridge.

The DVCon hardware-build case study shows why this seam matters: organizations already own compute farms and job dispatchers, and build-system adoption is substantially easier when remote actions can use those resources without requiring a shared filesystem or replacing the dispatcher. Mantle must add that interoperability without making scheduler responses an output-trust root, exposing license credentials, or embedding provider logic in its scheduling core.

## What Changes

- Add a versioned provider-neutral external batch-dispatch adapter protocol for submit, observe, cancel, and reconcile operations over bounded canonical request/response records.
- Bind each submission to Mantle realization, job, attempt, and fence identities; external job ids remain locator metadata and cannot authorize result admission.
- Invoke adapters as exact identified executables with typed argv/stdin/stdout, strict size/time limits, clean environment, and no shell command interpolation.
- Map already-admitted quantified resource requirements and named scarce token labels into adapter requests while keeping token secrets, license credentials, and provider policy out of action payloads and reports.
- Require the scheduled allocation to start or connect an ordinary Mantle remote worker; source/input and output movement continues through Mantle CAS/transfer/admission paths, not an assumed NFS workspace.
- Add one optional `slurm-cli-v1` shell adapter plus test-owned fake `sbatch`/`squeue`/`scancel` fixtures. Other systems, including LSF, can implement the protocol without changing Mantle core.
- Persist bounded external-job lifecycle facts, reconcile coordinator restarts, cancel stale/terminal attempts idempotently, and expose redacted diagnostics.
- Validate the adapter with the bounded hardware-simulation reference after that change is available, while retaining a smaller provider-free fixture for ordinary tests.

## Impact

- **Surfaces**: distributed/core DTOs, coordinator persistence and assignment, worker bootstrap, resource leases, typed Nickel remote-builder profiles, adapter process shell, optional Slurm adapter, remote status/build reports, and integration fixtures.
- **Dependencies**: consumes fenced remote attempts, resumable CAS transfer, immutable attempt logs, current output admission, and `account-scarce-resources-and-locality`; the hardware workload proof depends on `prove-hardware-simulation-build-flow`.
- **Boundary**: Mantle owns build/action identity, transfer, worker protocol, and output admission. The adapter owns provider submission/status/cancellation translation only; the external scheduler owns queue/resource allocation only.
- **Non-claims**: no LSF implementation, no Kubernetes integration, no provider autoscaling, no license-server implementation, no exactly-once physical execution, no trust from scheduler success, no shared-filesystem correctness, and no production-cluster support claim from fake Slurm fixtures.
- **Testing**: canonical protocol tests; malformed/stale/oversized/secret-bearing adapter negatives; fake Slurm submit/status/cancel/restart cases; worker registration and fenced admission; zero-shared-filesystem transfer; hardware workload composition; Cairn gates.
