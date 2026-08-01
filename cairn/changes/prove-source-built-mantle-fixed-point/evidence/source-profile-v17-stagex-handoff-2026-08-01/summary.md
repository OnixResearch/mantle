# Source-built fixed-point profile v17

## Result

The regenerated profile contains 68 source records.
Its manifest BLAKE3 is:

```text
dbc905175d05b97d9c7bbeb72e2b59acf9263e8ba05822f3a50d0961f0c1d7b6
```

The profile path is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v17-stagex-handoff-20260731.json
```

## Reason for regeneration

Commit `b440bb8e` changed the declared Mantle source input.
The proof now preserves the complete StageX execution tree and imports only the eight declared runtime outputs.
Profile v16 remains evidence for its prior source state, but it cannot authorize the repaired proof shell.

## Validation

Pueue task `7524` generated the profile in `3,000s`.
Pueue task `7600` independently verified the profile in `1,092s`.
The verifier reported:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

The remote-ref warnings in the raw command logs used existing cached refs.
They did not change the final ready result or the profile identity.

## Non-claim

The profile proves source availability and identity only.
It does not prove provider construction, the Mantle fixed point, or release eligibility.
