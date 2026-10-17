# Same immutable source, locked Rust toolchain, host target and dev profile as
# the opt-in unit plan. The linker shim exposes that plan's pinned GNU cc under
# the fixed executable name required by the default offlineCargoPackage lane.
let mantle = import "../../lib/lib.ncl" in
fun { workspace, rust_toolchain, shim, musl_runtime } =>
  mantle.offlineCargoPackage {
    name = "unit-baseline-app-offline-cargo",
    src = workspace,
    source_name = "baseline-workspace",
    rust = rust_toolchain,
    rust_name = "rust-default-1.96.0-nightly-2026-04-08",
    seed_toolchain = shim,
    seed_toolchain_name = "gnu-linker-shim",
    musl = musl_runtime,
    musl_name = "musl-x86_64-unknown-linux-musl-1.2.5",
    binary = "unit-baseline-app",
    target = "x86_64-unknown-linux-gnu",
    profile = 'debug,
  }
