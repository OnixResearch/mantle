## ADDED Requirements

### Requirement: First-stage musl stage2 prefix runtime visibility

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_prefix_runtime] Mantle MUST include the stage2 rustc prefix runtime directory in source-root musl first-stage Cargo probes.

#### Scenario: stage2 rustc probe can load prefix runtime libraries

GIVEN the generated first-stage Rust provider script is building `run_rustc` with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
WHEN mrustc's `run_rustc` stage2 standard-library Cargo build probes `$(BINDIR_2)rustc` through `rustc_proxy.sh`
THEN the `RUSTC_ENV_VARS` `LD_LIBRARY_PATH` entry MUST include the private source-root musl runtime directory, `$(PREFIX_2)lib`, and `$(LIBDIR)` in that order.
AND the `CARGO_ENV_STAGE2_STD` Makefile environment MUST inherit `$(RUSTC_ENV_VARS)`.

#### Scenario: unexpected runtime line fails closed

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN `run_rustc/Makefile` lacks either the expected original `RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))` line or an already-normalized line including `$(PREFIX_2)lib`
THEN Mantle MUST abort the first-stage build with a deterministic diagnostic before invoking `run_rustc` for the compiler host.

#### Scenario: runtime scope stays private

GIVEN the generated first-stage Rust provider script normalizes stage2 rustc prefix runtime visibility
WHEN the private runtime search path is installed
THEN Mantle MUST NOT replace global `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases with source-root musl target tools.
