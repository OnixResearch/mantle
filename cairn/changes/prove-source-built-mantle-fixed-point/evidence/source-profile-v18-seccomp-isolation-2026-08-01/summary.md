# Source-built fixed-point profile v18

## Result

The regenerated profile contains 68 source records.
Its manifest BLAKE3 is:

```text
6725fab7d286b7338c6a0d74f92230e95ee89c032a8a0a8fb252c9911209563b
```

The profile path is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v18-seccomp-isolation-20260801.json
```

## Reason for regeneration

Commit `0fa4270e` changed the declared Mantle source input.
The proof now runs the StageX transition and StageX provider publication on separate fresh worker threads.
Profile v17 remains evidence for its prior source state, but it cannot authorize the isolated proof shell.

## Validation

Pueue task `7793` generated the profile in `2,751s`.
Pueue task `7833` independently verified the profile in `991s`.
The verifier reported:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

The remote-ref warnings in the raw command logs used existing cached refs.
They did not change the final ready result or the profile identity.

## Non-claim

The profile proves source availability and identity only.
It does not prove provider construction, the Mantle fixed point, or release eligibility.
