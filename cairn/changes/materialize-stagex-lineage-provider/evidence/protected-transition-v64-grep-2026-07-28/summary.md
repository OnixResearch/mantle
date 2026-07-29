# Protected StageX transition v64: GNU grep bridge runner

## Result

Pueue task `3612` passed the complete protected transition test.

The transition completed 81 planned stages. It recorded 2,251 allowed execution events and no other decisions.

The report contains no fallback events. The StageX provider receipt remains blocked and was not promoted.

## Command

```text
nix develop -c env \
  MANTLE_STAGE_X_SOURCE_BUNDLE=/home/brittonr/.cargo-target/stagex-source-closure-v22-diffutils-20260728.json \
  MANTLE_STAGE_X_TRANSITION_SCRATCH=/home/brittonr/.cargo-target/stagex-protected-transition-v64-grep-final-20260728 \
  cargo test -p mantle --bin mantle \
    stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem \
    -- --exact --nocapture
```

The captured result is:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1830 filtered out; finished in 1256.36s
```

See `full-transition-test.log` for the captured pueue transcript.

## Bound identities

- Lineage manifest BLAKE3: `acd107888638cce6dbbc14832227395a0ab93dfe87d088c382fbd68069c908d7`
- Source-bundle manifest BLAKE3: `541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab`
- Source-bundle file BLAKE3: `205a3c7e405d832133b7dbb5617c75c3f4e0efadbe283fd4ef1d9209a6ff1c98`
- Source-state BLAKE3: `18b54c3ac3fdd7931453a849f15c76ea642d248d2b597b1b8c7849ae97f4a772`
- Plan BLAKE3 identity: `7f64555dac7f88375dc1d71e78ef2b5ae51ae0b166db1ecefe97dc562a17c0a6`
- Plan file BLAKE3: `ccea277f96aa3488b144d59e9ebe11969b738cd4114ad3656afbdc876c5194d8`
- Report file BLAKE3: `4e8914d41d2727304a9dddb6dc1474ed2657cc93cf2bab9e0d7edcf8577bf132`
- Audit file BLAKE3: `9412647ba5c0ac1f94f02a0838ba839e5a14ecbd6ce363d12ec0646372ed809c`

## GNU grep bridge boundary

The source record binds GNU grep 2.4 content BLAKE3 `b40c519f58ef370f44bbafc37a76ae843a3cf621a44ef83158294fb29ada14d0`.

The checked recipe emits a shell bridge with a `/bin/sh` shebang. Protected execution does not execute that script.

The exact script remains mode-`0644` data. Its BLAKE3 is `3da22b084c98e1ebef50ddcbe404c681d6a7d81b29389d26151729b1c74127de`.

ADR `0036` records why a native subset runner preserves this boundary without widening source-built Bash.

TinyCC musl-v2 compiles the runner and links it against protected native musl. The protected grep identities are:

- Runner source: `926cba44f7f96479db1e73a85e428346eccc380ca463704fa7f2e68905e9363f`
- Configured source: `d86dde390ef4ab7b4456f0371cbd29afa8b0e31ca297e7acc85b5f9544775c40`
- `grep`, `egrep`, and `fgrep` runner: `b4adac2bf1f29e3e3b8ec736207a0cdfc3eb5b51c658a2f9f8eff85aee4ef71d`
- Literal observation: `8e4c7c1b99dbfd50e7a95185fead5ee1448fa904a2fdd778eaf5f2dbfd629a99`
- Quiet observation: `af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`
- Anchored observation: `aa95faeede7041e63c6056bdcf10e6fbf709a355e539259da51a067e5dd27802`

`grep-inventory.json` records two build commands and three recipe-matching smoke commands.

Pueue task `3610` separately proved that an empty pattern exits with status 2 and writes no output.

## Validation

Pueue task `3625` passed all 31 focused transition tests.

Pueue task `3626` passed four grep tests. Two retained-input tests stayed ignored because the full proof covers those inputs.

The bounded-stdin tests passed. The retained probes reproduced the corrected runner identity.

Pueue tasks `3627` and `3624` passed the bootstrap evaluation and parity suites.

Pueue task `3615` passed strict first-party Clippy. Its only warning came from vendored `snix-castore`.

Pueue task `3623` passed the root-package formatting check.

Pueue task `3617` passed strict host-Clang checks for the C runner.

Pueue task `3616` passed the bootstrap source-pin checker through the pinned nightly Cargo runner.

Pueue task `3618` proved that the Nickel export matches the checked JSON. It also passed `git diff --check`.

Pueue task `3619` passed Nix-backed Cairn validation with `valid: true`.

Pueue tasks `3621`, `3620`, and `3622` passed the Nix-backed proposal, design, and tasks gates.

The Cairn checks used the current sibling Cairn policy explicitly. Mantle's local generated policy still needs the new nominal-identity schema field.

## Bounded claim

This evidence proves the declared protected transition through the checked GNU grep bridge subset.

It does not prove full GNU grep, parser generators, binutils, native TinyCC, a normalized provider, compiler correctness, or provider admission.
