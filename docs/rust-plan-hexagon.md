# Rust-plan hexagon

Mantle separates Rust package-planning meaning from host authority.

## Components

`mantle-rust-plan-core` is a `no_std + alloc` functional core. It owns:

- bounded package, target, dependency, feature, toolchain, and oracle facts;
- package and feature closure;
- host and target unit classification;
- deterministic unit topology and native unit identities;
- ordered compiler effects with explicit arguments, environment, inputs, outputs, and limits;
- cache and compiler observation classification;
- compatibility status and receipt preimages.

`mantle-rust-plan` owns application commands and five capability ports:

1. workspace facts;
2. Cargo oracle facts;
3. compiler inspection;
4. unit execution;
5. Rust cache access.

The ports use Mantle-owned values and typed capability errors. They do not expose `RunError`, `PathBuf`, process handles, Snix values, raw store services, or vendor cache types.

## Effect order

The shell obtains bounded workspace, compiler, and optional Cargo-oracle facts. The core then admits the facts and creates the plan.

For each ready unit, the application requests a cache observation. A valid cache hit produces a reused outcome. A miss permits the declared unit effect. A cache fault blocks execution. After execution, the core accepts only an observation with the exact effect identity and configured limits.

The application stops dependent execution after a compiler failure. It publishes only successful execution outcomes to the cache adapter.

## Compatibility

The existing `rust-plan` command and JSON remain the compatibility surface. The mature facade delegates compatibility classification, target selection, host classification, crate-name normalization, and native unit identity to the core.

Golden fixtures cover Cargo-oracle facts, compatibility classes, native unit identity, and receipt-preimage identity. Existing unit and CLI suites remain the behavior baseline.

## Checks

Run focused checks:

```sh
nix develop -c cargo test -p mantle-rust-plan-core -p mantle-rust-plan
nix develop -c cargo check -p mantle-rust-plan-core --target wasm32-unknown-unknown
nix develop -c cargo -Zscript scripts/check-rust-plan-hexagon.rs --self-test
nix develop -c cargo -Zscript scripts/check-rust-plan-hexagon.rs --root .
nix build "path:$PWD#checks.x86_64-linux.rust-plan-hexagon-architecture" --no-link -L --builders ''
```

The architecture check rejects filesystem, process, environment, Cargo-process, Cargo-JSON, rustc-process, store, cache-vendor, path, async-runtime, CLI, rendering, and Snix authority in the core.

## Non-claims

A deterministic plan does not prove that Cargo agrees outside the accepted matrix. An effect does not prove that rustc ran. An observation does not prove compiler or linker correctness.

This boundary does not prove cache correctness, full Cargo support, reproducibility, bootstrap correctness, or release eligibility.
