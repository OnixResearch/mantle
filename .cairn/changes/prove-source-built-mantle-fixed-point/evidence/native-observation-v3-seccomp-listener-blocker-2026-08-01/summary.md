# Native observation v3: seccomp listener blocker

## Result

Pueue task `7646` failed closed after `5,937s`.
The repaired runtime handoff passed strict source import.
The protected StageX transition completed.
StageX provider publication then failed before runtime validation because the same orchestrator thread tried to install a second seccomp user-notification listener.

The kernel returned:

```text
SECCOMP_SET_MODE_FILTER: Device or resource busy (os error 16)
```

No native provider was admitted.
The failed staging directory remains at:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-runs-v3/.native-digest-observation-v3-20260801.source-built-fixed-point-staging-1086807
```

## Root cause

A seccomp filter cannot be removed from its installing thread.
Linux also rejects a second `SECCOMP_FILTER_FLAG_NEW_LISTENER` filter on a thread that already owns a listener filter.
The combined proof had run both independently protected operations on one thread.
Even if provider publication had passed, its narrow filter would have remained on the orchestrator before native construction.

## Repair

ADR `0053` keeps the proof orchestrator unfiltered.
It runs the complete StageX transition in one fresh scoped worker thread and provider publication in a second fresh scoped worker thread.
Each worker completes descendant reaping and audit quiescence before it returns.
The later native, Rust, and fixed-point stages continue on the unfiltered orchestrator thread.

The current supervisor listeners remain process-scoped until process exit.
They are attached only to the completed worker lineages, and this proof creates exactly two such workers.

## Validation

Pueue task `7782` passed the kernel regression.
Two sequential fresh workers each installed a listener successfully.
A second listener on the same fresh worker failed with `EBUSY`.
The unfiltered parent then executed an undeclared probe successfully.

Pueue task `7781` passed 14 focused shell tests, including positive return, empty-name rejection, and panic propagation for the isolated worker shell.
Pueue task `7788` passed the complete protected-exec seccomp test module, strict first-party Clippy, and focused diff checks.

## Non-claim

This repair proves only listener separation and orchestrator isolation for the two protected operations.
It does not prove StageX provider publication in the combined proof, native-provider admission, the Mantle fixed point, or release eligibility.
