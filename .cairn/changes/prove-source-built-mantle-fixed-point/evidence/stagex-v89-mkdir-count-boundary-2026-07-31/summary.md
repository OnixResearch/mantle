# StageX v89 `mkdir` count boundary

## Result

Pueue task `6759` ran the complete protected StageX transition from commit `269255a7`.
The transition reached the final GNU binutils audit and then failed closed.

The exact error was:

```text
GNU binutils exec count drifted for /home/brittonr/.cargo-target/stagex-protected-transition-v89-canonical-native-musl-20260731/coreutils-stage/runtime/output/bin/mkdir: bounds [5385, 5425], observed [5377]
```

The test ran for `1764.25s`.
The retained scratch root is:

```text
/home/brittonr/.cargo-target/stagex-protected-transition-v89-canonical-native-musl-20260731
```

## Comparison

The comparison used the successful v88 audit and the failed v89 audit.
The full-audit path counts were:

| Tool | v88 | v89 | Difference |
|---|---:|---:|---:|
| `mkdir` | 5402 | 5378 | -24 |

All other 19 declared coreutils path counts were equal.
The binutils suffix excludes one earlier coreutils smoke event.
Thus, the suffix counts were 5401 and 5377.

The two binutils inventory files had equal values for these facts:

- nine configure classes
- eight components
- eleven installed tools
- four archives
- 4,891 protected sed invocations
- all listed output byte lengths
- all listed output BLAKE3 digests
- protected execution enabled
- zero fallback events

The v89 suffix passed the total event bounds before the path-specific check.
All exact non-`mkdir` identity counts, unique-identity checks, digest checks, and order checks remained active.

## Decision

Accept `5377` as the new lower `mkdir` count bound.
Keep the upper bound at `5425`.
Do not change the total event bounds or other identity checks.

Positive tests accept both boundary values.
Negative tests reject counts outside the bounds and reject two digest identities for one path.

Pueue task `6817` passed both new tests.
Pueue task `6825` passed the existing total-event boundary test.
Pueue task `6824` passed focused first-party Clippy with `-D warnings`.
Package rustfmt and `git diff --check` passed in pueue task `6823`.

## Non-claim

This failed run does not complete the StageX transition.
It does not publish a provider or prove the Mantle fixed point.
