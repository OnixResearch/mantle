# Design: Native Rust registry transitive topology execution

## Current state

Mantle already records native registry source facts for declared vendored registry packages and uses those facts to gate bounded topology execution. Existing coverage proves local roots with a single vendored registry package, registry-backed build-script producers, and registry-backed proc-macro producers on the unified topology rail.

What is not yet explicitly covered is a registry package whose own dependency is another vendored registry package. Without that proof, registry execution can regress to leaf-only behavior while receipts still look healthy for simple fixtures.

## Approach

Add a narrow Cairn-backed implementation slice for a transitive registry topology:

```text
local app -> registry crate A -> registry crate B
```

The implementation should:

1. Build a positive CLI fixture with two vendored registry crates represented in `Cargo.lock` and declared through supported `.cargo/config.toml` directory source replacement.
2. Require `native_registry_source_planning.ready == true` and ready source facts for both registry crates before any registry-backed unit is executed.
3. Preserve explicit native unit/derivation graph evidence for the complete closure.
4. Execute units in dependency order: registry B, registry A, local app.
5. Bind registry B's produced artifact into registry A's `--extern`/dependency material, then bind registry A's produced artifact into the local app.
6. Record deterministic output and dependency artifact BLAKE3 digests in topology receipts.
7. Add a negative fixture where a transitive registry dependency has missing/stale/unsupported vendored material and assert execution blocks before any dependent `rustc` invocation.

## Receipt expectations

The CLI JSON evidence should retain the existing `rust_plan` receipt and topology execution receipts. Assertions should prove:

- `native_registry_source_planning.ready == true` for the positive fixture.
- Both registry packages appear in native registry source facts with checksum/source digest evidence.
- Unit executions are ordered producer-first for the transitive registry closure.
- Downstream executions contain dependency artifact digest evidence for the producer they consume.
- The negative fixture reports deterministic native-registry blockers and does not execute unsupported/missing transitive units.

## Failure behavior

The rail must fail closed before `rustc` when any registry package in the transitive closure lacks ready registry source facts, has an unsupported vendored layout, lacks checksum/source identity, or would require network/Cargo cache fallback.

## Verification

Use the focused Rust package-planning verification sequence:

```bash
STATIC_BUSYBOX=$(test -x /run/current-system/sw/bin/busybox-static && echo /run/current-system/sw/bin/busybox-static || printf '%s\n' /nix/store/*-busybox-static-*/bin/busybox | head -1)
SNIX_BUILD_SANDBOX_SHELL="$STATIC_BUSYBOX" CARGO_TARGET_DIR="$PWD/target-rust-plan-test" nix develop -c cargo fmt --check
SNIX_BUILD_SANDBOX_SHELL="$STATIC_BUSYBOX" CARGO_TARGET_DIR="$PWD/target-rust-plan-test" nix develop -c cargo test --bin mantle rust_plan
SNIX_BUILD_SANDBOX_SHELL="$STATIC_BUSYBOX" CARGO_TARGET_DIR="$PWD/target-rust-plan-test" nix develop -c cargo test --test rust_plan_cli
nix run 'git+ssh://git@github.com/OnixResearch/cairn.git#cairn' -- validate --root .
nix run 'git+ssh://git@github.com/OnixResearch/cairn.git#cairn' -- gate tasks native-rust-registry-transitive-topology-execution --root .
git diff --check
rm -rf target-rust-plan-test
```
