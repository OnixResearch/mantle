# Protected StageX transition v60: GNU M4

## Result

Pueue task `3552` passed the complete protected transition test.

The transition completed 77 planned stages. It recorded 2,214 allowed execution events and no non-allowed events.

The report contains no fallback events. The StageX provider receipt remains blocked and was not promoted.

## Command

```text
nix develop -c env \
  MANTLE_STAGE_X_SOURCE_BUNDLE=/home/brittonr/.cargo-target/stagex-source-closure-v21-m4-20260728.json \
  MANTLE_STAGE_X_TRANSITION_SCRATCH=/home/brittonr/.cargo-target/stagex-protected-transition-v60-m4-20260728 \
  cargo test -p mantle --bin mantle \
    stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem \
    -- --exact --nocapture
```

The captured result is:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1812 filtered out; finished in 1389.21s
```

See `full-transition-test.log` for the captured pueue transcript.

## Bound identities

- Lineage manifest BLAKE3: `2048e9511cf8c170969bbd553061049f861778b98ba54175968dfac1824cece6`
- Source-bundle manifest BLAKE3: `bc58825cd1432e48cfd11e61dc2d0e72945ea9f5f9f6f68b9d8dc79ed2e0ad06`
- Source-bundle file BLAKE3: `1292937861da64fe09f626739575a1513fa1192285f663610310e8abd20b2e3a`
- Source-state BLAKE3: `7d2a26a2fd1f7b494bcfa504742a734b29fd58f4c3ff5ca875c01e05fb16aaa2`
- Plan BLAKE3 identity: `dd67aa68f991c1ba80384406371bee8ec1acf3713e1b6505b944125478afe8d3`
- Plan file BLAKE3: `f8292730f688897e6f49769b16ec9e627bb10daa27f71944f1fe3adc8ecb864c`
- Report file BLAKE3: `fb9b31478dbbabfcf6df49ddaf5fe1aba6666284a43ff2d5b4bf8376e3fa0ecf`
- Audit file BLAKE3: `04d036b9e35feda1c621d1745c3821e4d48cf47989b9038ca2e9e9640634c21f`

## GNU M4 boundary

The source record binds GNU M4 1.4.7 content BLAKE3 `1acf045348ef14f9fb38dc1e23fec5146d32b4fb6b7bb5c4dfb9742da12ce0d1`.

The Rust materializer compiles 29 fixed sources and performs one link. It uses TinyCC musl-v2 and the protected native-musl runtime.

The source transform removes `syscmd` and `esyscmd` from the M4 builtin table. The macro observation confirms that `syscmd` is unavailable.

The protected GNU M4 identities are:

- Configured source: `dbd75c146b108dfe62f23aeb29675b68d0d41bc8ca5b5dfd137d346a284376ac`
- Binary: `3dfd2a1223ff6e0c2c540bc2507a097948ab24e2465f232e2363944e5b704741`
- Version observation: `c9b69698f54279bd5ce4b9cba7442ba625e7ba92717ee139d3152093331c0e75`
- Macro and shell-disable observation: `f737ffa0d1bffb14295a97110f43d0f9cf059d5cbffc7ff6187b2182250d34ca`
- Prefix observation: `6277dd4b0cbd4b080f3a4c4e561d05f128ffee445dadefa2116cc8e3b079e26c`
- Malformed-input rejection: `675bd6ec5c382c78909a394f1f5a59e8b51e7e617f9ad9537e466fd799553420`

`m4-inventory.json` records 30 build commands and five smoke commands. It records protected execution and no fallback events.

## Native-musl parity correction

The native-musl source set now preserves `pthread_testcancel.c` and `pthread_setcancelstate.c`.

The corrected identities are:

- Configured source: `25136ba596733447cbb6259862f4e0b37dd557fc4dd8cc4392912e2f34880f18`
- Self-host compile count: 746
- Static libc: `a9aa627c3a70fce68d928e0a7ccd423370472a87ae9fde795a85db4d0899366c`

Two independent native-musl probes reproduced these identities before the complete transition.

## Validation

Pueue task `3546` passed focused formatting, Clippy, M4 tests, native-musl tests, transition tests, and `git diff --check`.

Pueue task `3551` passed the canonical predecessor-authority test and all 29 focused transition tests.

Pueue task `3557` passed Cairn validation with `valid: true`.

Pueue task `3558` proved that the Nickel export matches the checked JSON. It also passed `git diff --check`.

Pueue task `3559` passed the checked-in first-party strict Clippy gate.

## Bounded claim

This evidence proves the declared protected transition through GNU M4 under the recorded inputs and policies.

It does not prove parser generators, binutils, native TinyCC, a normalized provider, compiler correctness, or provider admission.
