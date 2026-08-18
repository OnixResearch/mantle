# Musl target fixed-point proof attempt

Task-ID: V2
Covers: rust_package_planning.source_built_toolchain_closure.target_aliases

## Result

Passed for the target-alias claim.

Mantle completed a Cargo-free fixed-point proof for `x86_64-unknown-linux-musl` using the source-built Rust provider plus an explicit receipt-bound toolchain closure manifest. The manifest kept generic `cc` / `ld` host-oriented and exposed target-prefixed musl tools through member-name/native-helper aliases.

This proof is intentionally bounded: the native C/linker/binutils tools in the manifest are `seed-exception` entries, so the proof still carries `not-source-built-toolchain-closure`.

## Command

Pueue task 19:

```text
/home/brittonr/.cargo-target/debug/mantle self-build --cargo-free --fixed-point \
  --out /home/brittonr/git/mantle-native-toolchain-target-alias-fixed-point-2026-06-16 \
  --rust-source-provider /home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider \
  --toolchain-closure /home/brittonr/git/mantle/target/receipt-bound-native-toolchain-target-aliases/toolchain-closure-host-and-target-aliases.json \
  --target x86_64-unknown-linux-musl
```

## Output excerpt

```text
Cargo-free fixed-point: success
bundle: /home/brittonr/git/mantle-native-toolchain-target-alias-fixed-point-2026-06-16
stage1_binary_blake3: a3a6117fedb7237a0da0dbeff6687680f9914f407079c645b295c83f2e051375
stage2_binary_blake3: a3a6117fedb7237a0da0dbeff6687680f9914f407079c645b295c83f2e051375
```

## Bundle facts

From `/home/brittonr/git/mantle-native-toolchain-target-alias-fixed-point-2026-06-16/meta.json`:

```text
status: success
fixed_point: true
stage1.execution_status: success
stage1.failed_unit_count: 0
stage1.unit_count: 686
stage2.execution_status: success
stage2.failed_unit_count: 0
stage2.unit_count: 686
source_built_toolchain_closure.status: validated-enforced
source_built_toolchain_closure.claim: false
source_built_toolchain_closure.member_count: 10
source_built_toolchain_closure.seed_exception_count: 8
source_built_toolchain_closure.source_built_member_count: 2
```

The generated guard PATH contained both host generic aliases and target-prefixed aliases, including:

```text
stage1/cargo-guard-bin/cc
stage1/cargo-guard-bin/ld
stage1/cargo-guard-bin/ld.lld
stage1/cargo-guard-bin/x86_64-linux-musl-gcc
stage1/cargo-guard-bin/x86_64-unknown-linux-musl-gcc
stage2/cargo-guard-bin/cc
stage2/cargo-guard-bin/ld
stage2/cargo-guard-bin/ld.lld
stage2/cargo-guard-bin/x86_64-linux-musl-gcc
stage2/cargo-guard-bin/x86_64-unknown-linux-musl-gcc
```
