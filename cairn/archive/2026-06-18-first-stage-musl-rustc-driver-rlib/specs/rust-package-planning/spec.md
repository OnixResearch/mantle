## ADDED Requirements

### Requirement: First-stage musl rustc-driver rlib normalization

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib] Mantle MUST normalize the Rust 1.90 `rustc_driver` crate type for the source-root musl compiler-host first-stage build.

#### Scenario: musl compiler host uses an rlib driver

GIVEN the generated first-stage Rust provider script is building `run_rustc` with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
WHEN the extracted Rust source contains `compiler/rustc_driver/Cargo.toml` with `crate-type = ["dylib"]`
THEN Mantle MUST rewrite that crate type to `crate-type = ["rlib"]` before invoking the `run_rustc` compiler-host build.
AND the rewrite MUST be private to the generated first-stage source tree.

#### Scenario: non-musl compiler hosts keep upstream driver shape

GIVEN the generated first-stage Rust provider script is building a compiler host other than `x86_64-unknown-linux-musl`
WHEN it prepares the extracted Rust source
THEN Mantle MUST NOT rewrite `compiler/rustc_driver/Cargo.toml` for that host route.
AND generic host aliases such as `cc`, `ld`, and `ld.lld` MUST remain host-oriented.

#### Scenario: unexpected driver manifest fails closed

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN `compiler/rustc_driver/Cargo.toml` is missing or lacks either `crate-type = ["dylib"]` or an already-normalized `crate-type = ["rlib"]`
THEN Mantle MUST abort the first-stage build with a deterministic diagnostic before invoking `run_rustc` for the compiler host.
