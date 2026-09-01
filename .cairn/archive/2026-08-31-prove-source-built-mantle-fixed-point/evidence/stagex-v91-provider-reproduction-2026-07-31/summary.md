# StageX provider reproduction after native-musl canonicalization

## Result

Two fresh protected transitions produced the same normalized provider identity:

```text
9e992a41dac86e4e069e3d5dcfdcb9512f2fb572639a3512aa0400895bffb269
```

Provider A came from:

```text
/home/brittonr/.cargo-target/stagex-protected-transition-v90-canonical-native-musl-a-20260731
```

Provider C came from:

```text
/home/brittonr/.cargo-target/stagex-protected-transition-v91-canonical-native-musl-c-20260731
```

## Transition evidence

Transition A retained a complete report.
Its provider publication in pueue task `6893` revalidated the transition before publication.
The queue removed the original transition task before its test log was captured.

Transition C ran in pueue task `6976`.
Its focused test passed in `1624.66s`.
The report records `status: complete` and 76,572 allowed protected exec decisions.
The audit contains no denied decision record.

## Provider evidence

Provider A published these run-specific identities:

- output BLAKE3 `ecf73f4b131cc9da36a9ce8b23717e9aaedfb9e9b2c10e1a49faedb7e155ff93`
- bundle BLAKE3 `c247643c9ba88adaa5ff255d37a9fde6448fe04cca5f49c89dca70f0dce57c03`

Provider C published these run-specific identities:

- output BLAKE3 `b4eb6e6517a87e29fedbf840238f4becb67573a2649549bb3df6fa5a889e1c1e`
- bundle BLAKE3 `fcb1138d5b52657057424c23c41d6a367988cc5f476d5c14a6e50c11fb1866dc`

Both publications recorded:

- normalized provider BLAKE3 `9e992a41dac86e4e069e3d5dcfdcb9512f2fb572639a3512aa0400895bffb269`
- validation audit BLAKE3 `b895024ed20acd76d7438ca9e2b7a99566e3c2140e040ca16fc77103d683123c`
- validation report BLAKE3 `a42ae56c41f4d3edd64c24bcd40d136852665d3974f2b41188f0aabd7d444cb6`
- zero fallback events

## Decision

Bind the fixed-point handoff to the reproduced normalized provider identity.
Keep each exact transition, output, audit, and bundle digest as run-specific evidence.

Pueue task `7044` passed the handoff identity test, the StageX source-graph test, rustfmt, and `git diff --check`.
Pueue task `7043` passed focused first-party Clippy with `-D warnings`.

## Non-claim

This result proves only the normalized StageX provider identity across these runs.
It does not prove the full-source native provider, Rust provider, Mantle fixed point, or release eligibility.
