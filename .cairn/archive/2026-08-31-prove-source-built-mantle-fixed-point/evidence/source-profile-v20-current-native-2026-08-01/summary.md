# Source-built fixed-point profile v20

## Result

The regenerated profile contains 60 records.
Its manifest BLAKE3 is:

```text
0a9b1df6d0d1c5578b5c2a88e0d86e6e0200cbf30b832cb612f363d88c41f1c7
```

The profile path is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v20-current-native-20260801.json
```

## Bound source change

The profile binds native source closure v8 and commit `e5fc5331`.
It retains the complete native manifest as a separate authority.
Its materialized source records exclude the two empty StageX store-path references and include the exact deduplicated fixed-fetch union.

## Validation

Pueue task `7117` generated the profile in `2,467s`.
Pueue task `7136` independently verified the profile in `872s`.
The verifier reported:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

The remote-ref warnings in the raw command logs used existing cached refs.
They did not change the final ready result or the profile identity.

## Non-claim

The profile proves source availability and identity only.
It does not prove provider construction, the Mantle fixed point, or release eligibility.
