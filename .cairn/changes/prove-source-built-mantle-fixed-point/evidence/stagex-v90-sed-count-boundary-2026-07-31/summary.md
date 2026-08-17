# StageX v90 sed count boundary

## Result

Pueue task `6894` ran the second fresh protected StageX transition.
The transition failed closed after the authenticated binutils component builds.

The exact error was:

```text
GNU binutils materialization failed: final sed bridge count mismatch: expected one of [4771, 4772], observed 4773
```

The test ran for `1620.26s`.
The retained scratch root is:

```text
/home/brittonr/.cargo-target/stagex-protected-transition-v90-canonical-native-musl-b-20260731
```

## Comparison

The sed audit had one additional accepted empty-input invocation.
A normalized sequence comparison against successful transition A found only this insertion:

```diff
 810 810 0 ok:protected-sed
 810 810 0 ok:protected-sed
 14 4 0 ok:protected-sed
+0 0 0 ok:protected-sed
 6 13 0 ok:protected-sed
 13 14 0 ok:protected-sed
 5 12 0 ok:protected-sed
```

The failed run retained all eight component outputs.
Byte comparisons found equal files for:

- `libiberty/libiberty.a`
- `zlib/libz.a`
- `bfd/.libs/libbfd.a`
- `opcodes/.libs/libopcodes.a`
- `binutils/size`
- `gas/as-new`
- `gprof/gprof`
- `ld/ld-new`

The preprocess count remained `145`.
The archive audit remained complete.
The protected sed audit recorded only accepted events.

## Decision

Accept full-build sed counts `4771`, `4772`, and `4773`.
Accept the corresponding install counts `4891`, `4892`, and `4893`.
Keep the global event bounds, exact executable identities, ordering checks, output digests, and no-fallback checks.

Positive tests accept the three observed values.
Negative tests reject values below and above each closed set.

Pueue task `6970` passed the positive count test.
Pueue task `6968` passed the negative count test.
Pueue task `6959` passed the total-event boundary test and `git diff --check`.
Pueue task `6969` passed focused first-party Clippy with `-D warnings`.

## Non-claim

This failed run does not complete the StageX transition.
It does not publish a provider or prove the Mantle fixed point.
