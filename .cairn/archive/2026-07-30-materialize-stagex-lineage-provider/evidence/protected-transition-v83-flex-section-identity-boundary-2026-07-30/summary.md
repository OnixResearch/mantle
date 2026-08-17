# Protected transition v83 Flex identity boundary

## Status

Diagnostic only. The transition did not complete and does not authorize provider publication.

## Command

Pueue task `3553` ran:

```text
MANTLE_STAGE_X_SOURCE_BUNDLE=/home/brittonr/.cargo-target/stagex-source-closure-v22-diffutils-20260728.json MANTLE_STAGE_X_TRANSITION_SCRATCH=/home/brittonr/.cargo-target/stagex-protected-transition-v83-provider-roles-final-20260730 nix develop -c cargo test -p mantle --bin mantle stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem -- --exact --nocapture
```

## Result

```text
called `Result::unwrap()` on an `Err` value: ProtectedExec("GNU Flex runtime failed: spawning /home/brittonr/.cargo-target/stagex-protected-transition-v83-provider-roles-final-20260730/flex-stage/runtime/output/bin/flex: Permission denied (os error 13)")

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1933 filtered out; finished in 1313.62s
```

The retained protected audit gives the exact denial:

```text
digest mismatch for .../flex-stage/runtime/output/bin/flex: expected 1adcaf70694b47268ddbaa32c834c78ee11f5a01a5fe45f07857607915b0507b, got 975d5864ad3f4a491d6e892a4a265a95f6c75f2630d1605a20552ff334f000f8
```

## Diagnosis

The v81 and v83 Flex executables have the same mode, length, headers, symbols, and runtime bytes. `cmp -l` found one changed byte at one-based file offset `429491`.

The byte is the first byte of a three-byte invalid section name. The section contains the exact 49-byte Flex compatibility syscall runtime. TinyCC emitted the name from unstable state:

- v81 and v74 section name bytes: `3d a3 8d`
- v83 section name bytes: `45 a3 8d`

No symbol or section-layout differences were present. Updating the expected digest would accept process-state variance, so Mantle must not do that.

## Repair boundary

The Flex materializer now applies a pure, bounded ELF transformation after link and before authorization. It requires:

- little-endian x86_64 `ET_EXEC`;
- a valid bounded section table and string table;
- exactly one `SHT_PROGBITS` section with `SHF_ALLOC`, alignment `8`, and the exact 49 runtime bytes;
- an exact three-byte section-name slot.

It replaces only that name with `stx`. It rejects malformed files, missing matches, duplicate matches, changed runtime bytes, and wrong name geometry.

Direct canonicalization of both retained v81 and v83 binaries produced byte-identical files with BLAKE3:

```text
502324a00e1b35d6fc18a6cf6c3a257d3578b7d5fd8bffc27ce41b9da3c1ebbf
```

The canonicalized retained v83 binary printed `flex 2.6.4`. Focused retained-binary validation passed in pueue task `3805`:

```text
test stagex_flex::tests::canonicalizes_retained_flex_runtime_identity ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1941 filtered out; finished in 0.00s
```

This repair proves only stable identity for the exact bounded runtime-section name. It does not prove TinyCC correctness, ELF semantic equivalence in general, Flex correctness, transition completion, or provider admission.
