# Protected StageX transition v74: GNU Flex 2.6.4

## Result

Pueue task `3912` passed the complete protected transition test.

The transition completed 87 planned stages. It recorded 2,367 allowed execution events and no other decisions.

The report contains no fallback events. The StageX provider receipt remains blocked and was not promoted.

## Command

```text
nix develop --option substituters https://cache.nixos.org -c env \
  MANTLE_STAGE_X_SOURCE_BUNDLE=/home/brittonr/.cargo-target/stagex-source-closure-v22-diffutils-20260728.json \
  MANTLE_STAGE_X_TRANSITION_SCRATCH=/home/brittonr/.cargo-target/stagex-protected-transition-v74-flex-final-20260729 \
  cargo test -p mantle --bin mantle \
    stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem \
    -- --exact --nocapture
```

The captured result is:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1856 filtered out; finished in 1274.97s
```

See `full-transition-test.log` for the captured pueue transcript.

## Bound identities

- Lineage manifest BLAKE3: `2db7db24bd9eabd539c886e01885d4ec0df29840b01b121f87f904a3281da4bb`
- Source-bundle manifest BLAKE3: `541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab`
- Source-bundle file BLAKE3: `205a3c7e405d832133b7dbb5617c75c3f4e0efadbe283fd4ef1d9209a6ff1c98`
- Source-state BLAKE3: `18b54c3ac3fdd7931453a849f15c76ea642d248d2b597b1b8c7849ae97f4a772`
- Plan BLAKE3 identity: `54e6883361893d6e67b063afedc5c24031316e23e60c4d69c9500a9161094839`
- Plan file BLAKE3: `36cf712559bf658a3f97fab5c0583bc8c105060daed96be98e297470f89aa117`
- Report file BLAKE3: `76340d004cd4b87fdd58b637c98b9fa6e5689fc7048ecdd01a19f5a6019360b9`
- Audit file BLAKE3: `a27d6a0c724939fe3008b8bc22ad92b5f4e930cb4ceaf03bfc6a75293daf950e`
- Flex inventory BLAKE3: `1880c67a573da5063378e6f2900283c991723e3a9514e5f242f0c5f461be4460`

## GNU Flex boundary

The source record binds GNU Flex 2.6.4 content BLAKE3 `6a84744fa55e734f9640c58c694c509dba34ddff104ab83e0131b9bc2468f8c0`.

The checked recipe BLAKE3 is `b9e11c2ef244ef9fab988d13bda45d0f0bdeabea8913964bb58cb00c9ac9688b`.

Rust orchestration applies each checked source normalization with an exact source-shape check. It extracts the bounded compatibility runtime from the checked recipe.

The build compiles 20 authenticated release-generated sources and one compatibility source with TinyCC musl-v2. It links against protected native musl.

Rust writes `libfl.a` as a one-member archive with fixed fields. A deterministic source-built consumer links and executes against the archive.

The protected Flex identities are:

- Configured source: `d2d7e2a0ed46329fc2e37d58d25dab8f17c01b5c9b7b97de09930ef3e25fe3b7`
- Flex binary: `1adcaf70694b47268ddbaa32c834c78ee11f5a01a5fe45f07857607915b0507b`
- `libfl.a`: `45c3a2d2e338eae19d24fa17df35127f5c61c16dca0ce98dce45efb13b920bfa`
- `FlexLexer.h`: `713ca824326279098cc1eb9b18ddd200e7506335c502cf2d79b4af7aaa330cdf`
- libfl consumer: `9d13005da4640c131d85edff0a255a83471b136d31ce41d2b04534c8a7b533b4`
- Version observation: `3697eccbbb98cf059d94cc199ff12f77bee88f5097ad8f0826c86bcf1fcbd5a7`
- Generated scanner source: `fb2c93883438dca7366f41ba363944da66b78787f76be32767cb697943b26732`
- Malformed-scanner diagnostic: `f1180cfefce605bb56ee38ccfec3bbd99f448e2d5f391bec146f4a02cdff336f`

`flex-inventory.json` records 21 source compilations, 25 build commands, and four explicit smoke commands. The valid and malformed scanner executions each add one protected M4 child execution.

The closed Flex audit suffix therefore contains 31 events. It has 25 TinyCC events, one libfl-consumer event, three Flex events, and two M4 events.

Retained build probes `stagex-flex-build-probe7` and `stagex-flex-build-probe8` reproduced the final identities.

## Validation

Pueue task `3945` passed the root-package formatting check and `git diff --check`.

Pueue task `3947` passed strict first-party Clippy. Its only warning came from vendored `snix-castore`.

Pueue task `3948` passed five Flex tests. Two retained-input tests stayed ignored because the complete proof covers those inputs.

Pueue task `3949` passed all 34 focused transition tests.

Pueue task `3950` passed the bootstrap evaluation and bootstrap parity suites.

Pueue task `3946` passed the bootstrap source-pin checker through the pinned nightly Cargo runner.

Pueue task `3951` proved that Nickel 1.16 exports byte-identical checked JSON. Both files have BLAKE3 `2db7db24bd9eabd539c886e01885d4ec0df29840b01b121f87f904a3281da4bb`.

Pueue task `3955` passed Nix-backed Cairn validation with `valid: true`.

Pueue tasks `3958`, `3957`, and `3956` passed the Nix-backed proposal, design, and tasks gates.

The Cairn checks used the current sibling Cairn policy explicitly. Mantle's local generated policy still needs the new nominal-identity schema field.

Pueue task `3967` ran the focused Tiger Style check. It stopped on the existing assertion-density finding in `src/protected_exec.rs::collect_plan_source_ids`, which this checkpoint does not modify.

## Bounded claim

This evidence proves the declared protected transition through GNU Flex 2.6.4, including its deterministic libfl proof and two declared source-built M4 child executions.

It does not prove arbitrary scanner semantics, binutils, native TinyCC, compiler correctness, a normalized provider, or provider admission.
