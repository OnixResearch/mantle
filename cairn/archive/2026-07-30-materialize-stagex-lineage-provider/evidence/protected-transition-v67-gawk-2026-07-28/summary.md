# Protected StageX transition v67: GNU Gawk 3.0.4

## Result

Pueue task `3602` passed the complete protected transition test.

The transition completed 83 planned stages. It recorded 2,275 allowed execution events and no other decisions.

The report contains no fallback events. The StageX provider receipt remains blocked and was not promoted.

## Command

```text
nix develop --option substituters https://cache.nixos.org -c env \
  MANTLE_STAGE_X_SOURCE_BUNDLE=/home/brittonr/.cargo-target/stagex-source-closure-v22-diffutils-20260728.json \
  MANTLE_STAGE_X_TRANSITION_SCRATCH=/home/brittonr/.cargo-target/stagex-protected-transition-v67-gawk-final-20260728 \
  cargo test -p mantle --bin mantle \
    stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem \
    -- --exact --nocapture
```

The captured result is:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1839 filtered out; finished in 1255.83s
```

See `full-transition-test.log` for the captured pueue transcript.

## Bound identities

- Lineage manifest BLAKE3: `60237483ba78dbc51a1fa4ab83296f2901f8c425ab7a6778ee0c7a4ba05f78b0`
- Source-bundle manifest BLAKE3: `541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab`
- Source-bundle file BLAKE3: `205a3c7e405d832133b7dbb5617c75c3f4e0efadbe283fd4ef1d9209a6ff1c98`
- Source-state BLAKE3: `18b54c3ac3fdd7931453a849f15c76ea642d248d2b597b1b8c7849ae97f4a772`
- Plan BLAKE3 identity: `f60d3bd28a72ce686167f671d1577e134ea6a40cf6b61445733e092ab6b9d7a5`
- Plan file BLAKE3: `03409ffc7817813ed349d452f2aa7293ae56dd74b6b4f76cb730963265b25fea`
- Report file BLAKE3: `3582afc4e6df2daa869840490f68a240e82e25ffef996668eea55846f6b0cee5`
- Audit file BLAKE3: `7ca1588094835dcaf45622db40db9e63192e8c323bf2178f1992f4b4ea5b93a0`

## GNU Gawk boundary

The source record binds GNU Gawk 3.0.4 content BLAKE3 `db0f59d2c7905c1e16d8c45df3548e304a593bdd193444da7e550ff109deb9aa`.

The checked recipe BLAKE3 is `1d35ef5716aea8d04a916ec05d3b4345256d4c4313205fd360b119aebb54b637`.

Rust orchestration extracts the exact `config.h` and `mantle_decimal.c` heredocs from that recipe. Pure transformations require the exact occurrence count for every checked source rewrite.

TinyCC musl-v2 compiles 16 relative object paths and links them against protected native musl. The orchestration adds owner read/write bits to each object, matching the checked `chmod u+rw` operation. `gawk-object-modes.txt` records the resulting mixed `0640` and `0650` modes.

The protected Gawk identities are:

- Configured source: `ccf0a5c408f8d60f4c8d72413c4c0ec0f8a49e29884c54c229ede61284dc7988`
- `gawk` and `awk`: `1608d2ac4b0f9faaecd91494502942fccfcec3fd6298aa610f2dfd4308aaf5d1`
- Behavior observation: `e8d40a8383b8b65278313f2589e148f294e19cba489e3517f3153c38d8603f7f`
- Numeric-array observation: `c95e800034a073abe91702b1faca0c3bf102d53fdebcd4d0cf46b39c524a98ba`
- Decimal observation: `49124bf4f7f37328738ac34216a60dcd5f58bb198c5c3f6719b6becafb7e7882`
- String-length observation: `b9a1a3183dd350f0e896d0f4b59c87e7bda8b1ed3a1af76afc86c1cb8f7cbbde`
- Hexadecimal observation: `320b59884ca4a418f85cef0916f3761d8a7225abda1b1069cb2182a9a15bdde9`
- Malformed-program diagnostic: `9633428410d0cea43777d103fa8f7aa73048977e6b3421ee6d8ea663f7722e65`
- Installed behavior observation: `b59a7a3ea06ce3059bdb45f57197d14ea5d2ecae9fb8b840cf7f486fc9685f33`

`gawk-inventory.json` records 17 build commands and seven smoke commands. The malformed program exited with status 1.

Retained build probes `stagex-gawk-build-probe2`, `stagex-gawk-build-probe3`, and `stagex-gawk-build-probe4` reproduced these identities.

## Validation

Pueue task `3598` passed the root-package formatting check.

Pueue task `3600` passed strict first-party Clippy. Its only warning came from vendored `snix-castore`.

Pueue task `3601` passed six Gawk tests. Two retained-input tests stayed ignored because the full proof covers those inputs.

Pueue task `3599` passed all 32 focused transition tests.

Pueue tasks `3603` and `3604` passed the bootstrap evaluation and parity suites.

Pueue task `3605` proved that the Nickel export matches the checked JSON. It also passed `git diff --check`.

Pueue task `3607` passed the bootstrap source-pin checker through the pinned nightly Cargo runner.

Pueue task `3613` passed Nix-backed Cairn validation with `valid: true`.

Pueue tasks `3615`, `3614`, and `3616` passed the Nix-backed proposal, design, and tasks gates.

The Cairn checks used the current sibling Cairn policy explicitly. Mantle's local generated policy still needs the new nominal-identity schema field.

Pueue task `3606` did not pass the broad Tiger Style check. The check stopped on the pre-existing assertion-density finding in `src/protected_exec.rs::collect_plan_source_ids`, which this checkpoint does not modify.

## Bounded claim

This evidence proves the declared protected transition through GNU Gawk 3.0.4 and its bounded generator observations.

It does not prove general AWK behavior, Bison, Flex, binutils, native TinyCC, a normalized provider, compiler correctness, or provider admission.
