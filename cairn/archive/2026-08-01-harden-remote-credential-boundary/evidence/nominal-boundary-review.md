# Remote credential nominal-boundary review

## Question

Which remote credential values need checked nominal types before the credential hardening change enters implementation?

## Inspected evidence

- `src/remote_build.rs:325-363` stores ticket IDs, bearer secrets, timestamps, use state, build-time limits, upload limits, and endpoint bindings as primitives.
- `src/remote_build.rs:1974-1994` compares presented secret text directly, then evaluates expiry, revocation, use, and endpoint policy.
- `src/remote_build.rs:11415-11435` exposes several adjacent `u64` and `u32` credential parameters through `TicketCreationFn`.
- `src/remote_build.rs:11444` uses saturating addition for issuance time plus TTL.
- `src/remote_build.rs:11991-12004` validates nonzero TTL and uses, build-time bounds, and upload bounds after primitive construction.
- `src/remote_build.rs:11394-11412` uses separate redacted and revealed views, while secret-bearing records still derive ordinary `Debug`.
- `crates/crunch-build/src/distributed/remote_attempt.rs:30-135` provides nearby typed identity and generation patterns.

## Decision

The existing `harden-remote-credential-boundary` change owns credential-specific nominal types. Structural protocol and legacy-state DTOs can remain primitive for bounded diagnostics. One pure admission boundary must construct checked core values before verifier or policy logic runs.

Secret-bearing values must not derive ordinary `Debug`, implement ordinary `Display`, or support unrestricted serialization. TTL conversion must use checked arithmetic. Remaining-use state must be distinct from a nonzero issuance limit because valid exhausted state is zero.

## Owner

`harden-remote-credential-boundary`

## Next action

Implement tasks 2.6 through 2.8 and 5.5 after the recorded baseline tests. Keep the public wire and migration diagnostics compatible unless a separate versioned schema change approves a difference.

## Non-claims

This review does not prove ticket entropy, constant-time verification, migration safety, secret deletion, service deployment, or release readiness.
