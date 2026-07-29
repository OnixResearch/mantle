# Protected StageX transition v62: GNU diffutils

## Result

Pueue task `3549` passed the complete protected transition test.

The transition completed 79 planned stages. It recorded 2,246 allowed execution events and no other decisions.

The report contains no fallback events. The StageX provider receipt remains blocked and was not promoted.

## Command

```text
nix develop -c env \
  MANTLE_STAGE_X_SOURCE_BUNDLE=/home/brittonr/.cargo-target/stagex-source-closure-v22-diffutils-20260728.json \
  MANTLE_STAGE_X_TRANSITION_SCRATCH=/home/brittonr/.cargo-target/stagex-protected-transition-v62-diffutils-final-20260728 \
  cargo test -p mantle --bin mantle \
    stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem \
    -- --exact --nocapture
```

The captured result is:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1822 filtered out; finished in 1309.27s
```

See `full-transition-test.log` for the captured pueue transcript.

## Bound identities

- Lineage manifest BLAKE3: `ed230a16c39e09bc025263ebcf6047d03ea669a35170ae254208c4e1758574a8`
- Source-bundle manifest BLAKE3: `541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab`
- Source-bundle file BLAKE3: `205a3c7e405d832133b7dbb5617c75c3f4e0efadbe283fd4ef1d9209a6ff1c98`
- Source-state BLAKE3: `18b54c3ac3fdd7931453a849f15c76ea642d248d2b597b1b8c7849ae97f4a772`
- Plan BLAKE3 identity: `e8a86d4326df2d190c9dc45a6206fb16e6da82bdd20308373340c8ee79255324`
- Plan file BLAKE3: `7df45ecd0386310b2353d06eb1dcb07c567984c9984503bae3e1ad3124248acd`
- Report file BLAKE3: `e0917a40c29713d13c7bb9714d0b9851a878ae16ecca2bce500e0d5c9b216e92`
- Audit file BLAKE3: `c4938bfffdd2011e095b5a88e9ba73358abb2d5fe11007dc6369a9797b83df89`

## GNU diffutils boundary

The source record binds GNU diffutils 2.7 content BLAKE3 `36714b7ee4e36f7f39ca3fb32010c5babe8a09532ee991f565771f9ca26daf90`.

The Rust materializer compiles 24 fixed sources and performs two links. It uses TinyCC musl-v2 and protected native musl.

The two links use relative object paths. All 24 generated object files have mode `0644` before each link.

The protected GNU diffutils identities are:

- Configured source: `00d79a6f09d23d7a3430ccfd407a8cac0351aa5ecc6efc122f8cf093989cbbe5`
- `diff`: `054ba2636aef71a1cbda93de3358ff8b8a7c92127ce951dcb7cc5814ac0e2d66`
- `cmp`: `145c05e3557ba74a8ce85912c08feee01b7616fcc273e92279a96dbe18a8793b`
- Version observation: `33510988f98a66cf0bc0e53661c6af77e8ed062faa45eac70734aef40a65ce6c`
- `diff` difference observation: `db3c58f16a007600d256c0be6306d55101919037d37ed4ece231bab4c3bbe2ca`
- `cmp` difference observation: `5d5856604bb2fa260c00b5ed8bcaa9a6b0bb89a644c7b23a2b678bf0e9127f6e`
- Malformed-input rejection: `202fa55131223d7bc743f2ff8d6896e001df815d001f698d58be1bc451ea5725`

`diffutils-inventory.json` records 26 build commands and six smoke commands. It records protected execution and no fallback events.

## Validation

Pueue task `3526` passed the focused positive and negative diffutils tests.

Pueue task `3556` passed all 30 focused transition tests.

Pueue tasks `3520` and `3521` passed the bootstrap evaluation and parity suites.

Pueue task `3527` passed strict first-party Clippy. Its only warning came from vendored `snix-castore`.

Pueue task `3552` passed the root-package formatting check.

Pueue task `3554` proved that the Nickel export matches the checked JSON.

The source-pin checker passed through the pinned nightly Cargo script runner.

Pueue task `3564` passed Nix-backed Cairn validation with `valid: true`.

Pueue tasks `3565`, `3566`, and `3567` passed the Nix-backed proposal, design, and tasks gates.

These checks used the current sibling Cairn policy explicitly. Mantle's local generated policy still needs the new nominal-identity schema field.

## Bounded claim

This evidence proves the declared protected transition through GNU diffutils under the recorded inputs and policies.

It does not prove grep, parser generators, binutils, native TinyCC, a normalized provider, compiler correctness, or provider admission.
