# ADR 0053: Isolate irreversible seccomp listeners by proof stage

## Status

Accepted (2026-08-01)

## Context

The source-built fixed-point proof runs two independently protected operations before native construction:

1. the complete StageX transition;
2. StageX provider relocation and runtime validation.

Both operations install a current-thread seccomp user-notification filter for `execve` and `execveat`.
A seccomp filter cannot be removed from its installing thread.
Linux rejects a second filter with `SECCOMP_FILTER_FLAG_NEW_LISTENER` when that thread already has a listener filter.

The first combined proof that reached provider publication failed with `SECCOMP_SET_MODE_FILTER: Device or resource busy`.
Running provider validation under the transition policy is not valid because the two stages have different executable authority.
Installing the provider policy on the orchestrator thread would also leave its narrow filter active during later native builds.

## Decision Drivers

- Keep each protected stage under its own exact executable policy.
- Keep the proof orchestrator unfiltered for later native and Rust construction.
- Do not weaken either protected policy or merge unrelated authorities.
- Preserve panic and error propagation from each worker.
- Keep the number of protected worker lineages fixed and reviewable.

## Decision

The source-built fixed-point shell runs each operation in a distinct fresh scoped operating-system thread.

The first worker runs the complete StageX transition, including descendant reaping and audit quiescence.
The shell joins that worker before it starts the next operation.
The second worker runs complete StageX provider publication and runtime validation.
The shell joins it before native-provider construction starts.

The orchestrator thread does not install either seccomp filter.
Threads created by the unfiltered orchestrator after a worker completes do not inherit the completed worker's filter.
Each worker returns its ordinary typed result to the orchestrator.
A worker panic becomes a proof error.

The current seccomp supervisor uses a process-scoped raw listener file descriptor and detached listener thread.
Those resources remain until process exit.
The fixed-point proof creates exactly two protected worker lineages.
Both operations already require adopted-descendant reaping and audit quiescence before return.
This bounded process-lifetime retention is accepted for this proof command; it does not permit an unbounded loop of listener installations.

A subprocess kernel test verifies three facts on the running kernel:

- two sequential fresh worker threads can each install a listener;
- a second listener on one fresh worker fails with `EBUSY`;
- the orchestrator remains able to execute an undeclared probe after the workers finish.

## Alternatives Considered

### Install both policies on the orchestrator thread

Rejected because the kernel rejects the second listener and the first filter is irreversible.

### Merge transition and provider executable authority

Rejected because it expands both protected stages beyond their declared needs and weakens audit meaning.

### Run provider validation under the transition supervisor

Rejected because provider relocation creates new paths and identities that require a separate promoted-output policy.

### Disable provider runtime validation in the combined proof

Rejected because it removes required positive and negative runtime evidence.

## Consequences

- The proof can compose two independently protected stages without seccomp listener conflict.
- Later native, Rust, and fixed-point work runs from an unfiltered orchestrator thread.
- Protected stage errors and panics remain fail-closed.
- Two detached supervisor threads and listener descriptors remain until the proof process exits.
- Future protected stages must use another fresh bounded worker or redesign the supervisor lifecycle; they must not install another listener on an already filtered thread.
- This decision does not prove StageX completion, provider admission, compiler correctness, the Mantle fixed point, or release eligibility.
