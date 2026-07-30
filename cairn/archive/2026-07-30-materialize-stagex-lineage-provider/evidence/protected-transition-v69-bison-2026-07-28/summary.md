# Protected StageX transition v69: GNU Bison 2.3

## Result

Pueue task `3663` passed the complete protected transition test.

The transition completed 85 planned stages. It recorded 2,336 allowed execution events and no other decisions.

The report contains no fallback events. The StageX provider receipt remains blocked and was not promoted.

## Command

```text
nix develop --option substituters https://cache.nixos.org -c env \
  MANTLE_STAGE_X_SOURCE_BUNDLE=/home/brittonr/.cargo-target/stagex-source-closure-v22-diffutils-20260728.json \
  MANTLE_STAGE_X_TRANSITION_SCRATCH=/home/brittonr/.cargo-target/stagex-protected-transition-v69-bison-final-20260728 \
  cargo test -p mantle --bin mantle \
    stagex_transition::tests::protected_transition_reproduces_seed_and_builds_kaem \
    -- --exact --nocapture
```

The captured result is:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1848 filtered out; finished in 1278.98s
```

See `full-transition-test.log` for the captured pueue transcript.

## Bound identities

- Lineage manifest BLAKE3: `cbba4ba5c02fa594d928cbfd9e581cdbfd92547b1befdd59101fa3074432202e`
- Source-bundle manifest BLAKE3: `541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab`
- Source-bundle file BLAKE3: `205a3c7e405d832133b7dbb5617c75c3f4e0efadbe283fd4ef1d9209a6ff1c98`
- Source-state BLAKE3: `18b54c3ac3fdd7931453a849f15c76ea642d248d2b597b1b8c7849ae97f4a772`
- Plan BLAKE3 identity: `0fc6327783dbe711acfb84746c4d8e97e5f05f5164d9e8552e6a8ab5ec4a8157`
- Plan file BLAKE3: `d01c9781513ec6668d44474e90294b98c735cd0238e5a5721e495e3b56566c53`
- Report file BLAKE3: `4c2328d478afeeb7f211b4ae6ee5f0a854798eb7e07a5f76f6a923b5b8f36951`
- Audit file BLAKE3: `dad8a597535f63bbe2358db03a92fb5a9b35dc62f2ed0dcdd0cf01f9f8684e7a`

## GNU Bison boundary

The source record binds GNU Bison 2.3 content BLAKE3 `45e40c482750cf4aa35637a7f62ee4f865a8a6d5dcf06aa557be8797c7bc4c62`.

The checked recipe BLAKE3 is `a086f71d144ee17961fe8462fee3b1d9184af885e45ee0200aa133c9fd54bd63`.

Rust orchestration applies every bounded hash-table patch hunk with an exact source-shape check. It extracts the compatibility runtime from the checked recipe.

Compiled M4 and runtime-data fallback paths are deterministic unavailable paths. Each Bison execution receives explicit `M4` and `BISON_PKGDATADIR` values from protected artifacts.

The protected Bison identities are:

- Configured source: `7b9699a83f7028fb22c8aa5bffaa5e314e2a9e1ad8625c0304c2a7bbbf37f121`
- Bison binary: `3db09397aa2752eea9c438a4cb1c5ebd27605f58b56ec43781baf411c3f2b983`
- Runtime data tree: `f3d4a63ba0b8049557c690ad7db8d48d954c3dd6e45cfdd9488fe6d75d01bc7c`
- Version observation: `a83fcaecff55230ffd6fb2ff09b2018392e8c403a3e2a8087d2cb68115f383c3`
- Generated parser source: `bf73d7f6db6f0dd4a96706a1efb9e23dd287cbf847badd17f64d4c7ed235c923`
- Generated parser header: `afa05cc327a73b17e33baf53fb44030b1a111aa993db60f01565a2a80dc8f913`
- Malformed-grammar diagnostic: `d0c674aaeab59c3efab1090e74cb598ae4df44bbcd42bc451a0a3f1a1257dcb4`

`bison-inventory.json` records 56 source compilations, 57 build commands, and three Bison smoke commands. The valid grammar adds one protected M4 child execution. The malformed grammar exited with status 1.

Retained build probes `stagex-bison-build-probe1` and `stagex-bison-build-probe2` reproduced these identities.

## Validation

Pueue task `3677` passed the root-package formatting check and `git diff --check`.

Pueue task `3681` passed strict first-party Clippy. Its only warning came from vendored `snix-castore`.

Pueue task `3680` passed six Bison tests. Two retained-input tests stayed ignored because the full proof covers those inputs.

Pueue task `3678` passed all 33 focused transition tests.

Pueue task `3652` passed the bootstrap evaluation suite. Pueue task `3679` passed the bootstrap parity suite.

Pueue task `3653` proved that the Nickel export matches the checked JSON.

Pueue task `3682` passed the bootstrap source-pin checker through the pinned nightly Cargo runner.

Pueue task `3687` passed Nix-backed Cairn validation with `valid: true`.

Pueue tasks `3689`, `3690`, and `3688` passed the Nix-backed proposal, design, and tasks gates.

The Cairn checks used the current sibling Cairn policy explicitly. Mantle's local generated policy still needs the new nominal-identity schema field.

The broad Tiger Style check still stops on the existing assertion-density finding in `src/protected_exec.rs::collect_plan_source_ids`, which this checkpoint does not modify.

## Bounded claim

This evidence proves the declared protected transition through GNU Bison 2.3, including one declared source-built M4 child execution.

It does not prove arbitrary grammar semantics, Flex, binutils, native TinyCC, a normalized provider, compiler correctness, or provider admission.
