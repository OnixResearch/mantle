# Design: Native Rust registry feature-gated topology planning

## Current state

Mantle's registry topology rail now has explicit proof for:

- a local package depending on one vendored registry library;
- vendored registry build-script host producers;
- vendored registry proc-macro host producers;
- transitive vendored registry chains such as `app -> registry A -> registry B`.

The remaining accepted-spec blocker class that blocks many real registry crates is feature handling. Existing native planning intentionally fails closed for unsupported feature surfaces instead of inheriting hidden Cargo resolver behavior.

## Approach

Implement a narrow native feature seam rather than full resolver compatibility.

Supported initial shape:

```text
local app --selected feature--> registry crate A --optional dependency--> registry crate B
```

or an equivalent bounded fixture where selected features are explicit in the package manifests and the resulting graph has one optional registry dependency that is either activated or rejected deterministically.

The implementation should:

1. Parse the minimal supported feature declarations directly from `Cargo.toml`/lockfile/vendor facts.
2. Record selected feature facts in native package/target or adjacent planning receipt evidence.
3. Activate only explicitly supported optional dependencies in deterministic order.
4. Require ready `native_registry_source_planning` facts for every registry package selected into the graph.
5. Feed the selected feature graph into native unit/derivation/topology execution without using Cargo as the build orchestrator.
6. Add a positive CLI fixture proving selected feature activation and dependency artifact binding.
7. Add a negative CLI fixture proving unsupported feature shapes block before downstream `rustc`.

## Receipt expectations

Positive CLI JSON should prove:

- selected feature facts are present and deterministic;
- selected registry source facts are ready for all activated packages;
- unselected optional packages do not enter the executed topology;
- selected optional dependency artifacts are bound into downstream `rustc` material;
- topology execution retains BLAKE3 output/dependency artifact evidence.

Negative CLI JSON should prove:

- unsupported feature syntax or resolver-dependent feature unification produces deterministic blockers;
- blocked feature graphs execute zero affected units;
- receipts do not claim Cargo resolver, network, `$CARGO_HOME`, registry cache, or version-solving fallback.

## Failure behavior

Mantle must fail closed before `rustc` for feature surfaces outside the bounded supported set, including ambiguous feature names, unsupported default-feature behavior, workspace feature inheritance, target-specific feature activation, or optional dependency activation that cannot be derived from explicit native facts.

## Verification

Use the focused Rust package-planning verification sequence:

```bash
STATIC_BUSYBOX=$(test -x /run/current-system/sw/bin/busybox-static && echo /run/current-system/sw/bin/busybox-static || printf '%s\n' /nix/store/*-busybox-static-*/bin/busybox | head -1)
SNIX_BUILD_SANDBOX_SHELL="$STATIC_BUSYBOX" CARGO_TARGET_DIR="$PWD/target-rust-plan-test" nix develop -c cargo fmt --check
SNIX_BUILD_SANDBOX_SHELL="$STATIC_BUSYBOX" CARGO_TARGET_DIR="$PWD/target-rust-plan-test" nix develop -c cargo test --bin mantle rust_plan
SNIX_BUILD_SANDBOX_SHELL="$STATIC_BUSYBOX" CARGO_TARGET_DIR="$PWD/target-rust-plan-test" nix develop -c cargo test --test rust_plan_cli
nix run 'git+ssh://git@github.com/OnixResearch/cairn.git#cairn' -- validate --root .
nix run 'git+ssh://git@github.com/OnixResearch/cairn.git#cairn' -- gate tasks native-rust-registry-feature-gated-topology-planning --root .
git diff --check
rm -rf target-rust-plan-test
```
