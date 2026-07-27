## Why

Mantle has two separately proven bootstrap lanes: the selected `seed-full` C/C++ provider reaches GCC 10, musl 1.2.5, and binutils 2.41, while an earlier source-built Rust 1.94 provider reached a Cargo-free Mantle fixed point against a different source-root native closure. The ordinary bootstrap route still fetches a prebuilt Rust toolchain, so no current receipt proves that the admitted full-source native provider built the Rust compiler and sysroots later used to build Mantle.

The first remaining integration boundary is therefore not another compiler version. Mantle must bind the admitted full-source provider to the mrustc-to-Rust route, preserve host/target separation, and fail closed instead of falling back to Nix, rustup, ambient compilers, or the older host-assisted source-root closure.

## What Changes

- Add a full-source-native input mode to the Rust source-provider materializer and bind the selected provider admission, source closure, native tools, runtimes, and Rust stage receipts by BLAKE3.
- Run a bounded discriminating probe for a musl compiler-host route; permit a GNU compiler-host route only if its complete native compiler/libc/linker closure is independently source-built and receipt-bound.
- Materialize a Rust compiler, Cargo, rustdoc, host rustlib, and musl target rustlib without prebuilt Rust or undeclared host compilation inputs.
- Preserve the fetched Rust route only as an explicit compatibility/development path that cannot satisfy full-bootstrap evidence.

## Dependencies

- Accepted `bootstrap_inventory.source_built_seed_provider` evidence.
- Accepted `rust_package_planning.source_built_toolchain_closure.provider_fixed_point` evidence.

## Impact

- **Files**: `bootstrap/{rust,rust-source,rust-source-plan,rust-source-musl-host-plan}.ncl`, `src/{rust_source_provider,source_toolchain_closure,bootstrap_source_root}.rs`, CLI wiring, tests, evidence, and operator documentation.
- **Testing**: positive and negative provider/closure tests; real Rust source materialization; declared-path compiler smoke; source-pin audit; focused first-party checks; Cairn gates.