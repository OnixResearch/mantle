# Source-built fixed-point profile v16

## Result

The regenerated profile contains 68 source records.
Its manifest BLAKE3 is:

```text
113a268d337eeda6652740b1f36f2076afa8762550b919ef4613e599c2927ad1
```

The profile path is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v16-m4-object-mode-20260731.json
```

## Reason for regeneration

Commit `23fb181b` changed the declared Mantle source input.
The change normalizes protected GNU M4 object modes before linking.
Profile v15 therefore remained valid evidence for its old source state, but it could not authorize the repaired proof.

## Validation

Pueue task `7294` generated the profile in `2,720s`.
Pueue task `7390` verified the profile in `1,030s`.
The verifier reported:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

## Non-claim

The profile proves source availability and identity only.
It does not prove provider construction, the Mantle fixed point, or release eligibility.
