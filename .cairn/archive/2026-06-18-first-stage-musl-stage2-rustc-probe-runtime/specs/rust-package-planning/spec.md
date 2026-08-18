## ADDED Requirements

### Requirement: First-stage musl stage2 rustc probe runtime visibility

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_rustc_probe_runtime] Mantle MUST make the private source-root musl runtime visible to first-stage stage2 Cargo `rustc` probes.

#### Scenario: stage2 Cargo inherits rustc runtime environment

GIVEN the generated first-stage Rust provider script is building `run_rustc` with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
WHEN mrustc's `run_rustc` stage2 standard-library Cargo build probes `rustc -vV` through `rustc_proxy.sh`
THEN the `CARGO_ENV_STAGE2_STD` Makefile environment MUST include `$(RUSTC_ENV_VARS)`.
AND the probed musl-host rustc MUST see the same private source-root musl runtime search path as later compiler-host Cargo builds.

#### Scenario: unexpected stage2 env line fails closed

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN `run_rustc/Makefile` lacks either the expected original `CARGO_ENV_STAGE2_STD` line or an already-normalized line that includes `$(RUSTC_ENV_VARS)`
THEN Mantle MUST abort the first-stage build with a deterministic diagnostic before invoking `run_rustc` for the compiler host.

#### Scenario: runtime scope stays private

GIVEN the generated first-stage Rust provider script normalizes stage2 Cargo rustc probes
WHEN the private runtime search path is installed
THEN Mantle MUST NOT replace global `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases with source-root musl target tools.
