# Native observation v11: GCC pass4 relocation

## Result

The preserved strict offline proof state reached GCC pass4 and binutils pass4.
The first repair made only the final linker wrapper relocation-stable.
That repair built `gcc-4.0.4-musl-pass4-v6`, but binutils configure still rejected the compiler.

A direct compiler probe isolated the remaining cause.
The published `bin/as` wrapper still selected `/tmp/gcc-4.0.4-musl-pass4/binutils/bin/as` after that build directory had gone away.

Version v7 now keeps temporary assembler and linker wrappers only during the GCC build.
Before publication, it rewrites both wrappers to use explicit overrides or the logical source-built StageX tools:

```text
${MANTLE_GCC_ASSEMBLER:-$STAGEX/bin/x86_64-linux-musl-as}
${MANTLE_GCC_LINKER:-$STAGEX/bin/x86_64-linux-musl-ld}
```

## Validation

Pueue task `7462` passed the focused positive and negative wrapper contract test:

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 31 filtered out
```

Pueue task `7467` rebuilt the handoff in the preserved v11 state.
The report records strict hermeticity, no hermeticity audit events, one built root, no cache hit, and no failure.
It published:

```text
/mantle/store/pxbkz1sp4v14kgpmv3icsqi38b4vkag0-gcc-4.0.4-musl-pass4-v7
/mantle/store/f5vgnr1i4d253j78fm19g7kcs0dh77af-binutils-2.30-gcc-pass4-v16
```

Pueue task `7468` passed `cargo fmt -p mantle -- --check` and `git diff --check`.
The full `bootstrap_eval` test had the same unrelated pre-existing failure before and after this repair.
Task `7427` reported 30 passed and one failed before the repair.
Task `7502` reported 31 passed and one failed after the new test was added.
Both failed only `gcc_native_diagnostic_uses_runtime_tcc_without_autotools_claims` because its stale marker expects `tcc-musl-native.ncl`.

## Non-claim

This observation proves the bounded GCC pass4-to-binutils pass4 handoff in one preserved strict offline state.
It does not prove the later compiler chain, the source-built Rust provider, a Mantle fixed point, compiler correctness, or release eligibility.
