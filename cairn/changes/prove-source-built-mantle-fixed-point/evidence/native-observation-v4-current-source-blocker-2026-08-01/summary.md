# Native observation v4: current source-closure blocker

## Result

Pueue task `7839` failed closed after `9,605s`.
The protected StageX transition and StageX provider publication completed.
The provider validation report had status `complete`, seven allowed events, all positive and negative runtime results, and no fallback events.
The normalized StageX provider digest remained:

```text
9e992a41dac86e4e069e3d5dcfdcb9512f2fb572639a3512aa0400895bffb269
```

Native construction did not start.
Its offline preflight reported one network-required source:

```text
fixed-url-82f9536a4fee1d373564718adcdd825712ef0cb0b8bee13b09beb69d7c2b37a3
```

That record is the recursive source identity for `patch-2.5.9-src` from `https://ftpmirror.gnu.org/patch/patch-2.5.9.tar.gz`.
The v18 profile carried an older native source closure that was internally valid but no longer matched the current `bootstrap/seed-full-toolchain.ncl` graph.

The initial durable blocker said that the report lacked a build-report `schema` field.
The preserved transcript showed the actual `mantle-source-offline-preflight-v1` blocker.

## Repair

Pueue task `7895` exported the current native source closure after seeding a fresh source state from v7.
The connected export fetched and fixed-output-validated the missing source.
Pueue task `7900` independently verified the new bundle as ready.

The proof report parser now checks failed command status before it requires a successful build-report schema.
For offline-preflight failures, it preserves readiness and bounded blocker identities in the durable attempt status.
Pueue task `7913` passed the focused positive and malformed-output regression.
Pueue task `7911` passed the focused shell tests, strict first-party Clippy, and focused diff checks.

## Preserved evidence

The failed staging directory is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-runs-v4/.native-digest-observation-v4-20260801.source-built-fixed-point-staging-2818502
```

Its StageX provider receipt records lineage status `complete`, normalized provider digest `9e992a41...`, and zero fallback events.
Its `transcripts/full-source-native-provider.json` records the exact missing source identity.

## Non-claim

This attempt proves combined StageX transition and provider publication through the bounded validation surface.
It does not prove native-provider construction or admission, the Rust provider, the Mantle fixed point, or release eligibility.
