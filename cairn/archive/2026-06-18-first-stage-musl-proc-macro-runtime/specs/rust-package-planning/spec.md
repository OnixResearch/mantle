## ADDED Requirements

### Requirement: First-stage musl proc-macro runtime visibility

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime] Mantle MUST make source-root musl runtime libraries visible to first-stage musl-host proc macros without exposing generic host aliases globally.

#### Scenario: musl proc macros load source-root libc

GIVEN the generated first-stage Rust provider script is building `run_rustc` with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
WHEN mrustc's `run_rustc` build loads host proc-macro shared objects such as `tracing_attributes`
THEN Mantle MUST place the source-root musl `libc.so` in the private first-stage runtime directory.
AND the `run_rustc` compiler-host build MUST search that private runtime directory before its build libdir when loading proc macros.

#### Scenario: runtime normalization fails closed

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN the source-root musl `libc.so` is missing or `run_rustc/Makefile` lacks the expected `RUSTC_ENV_VARS` `LD_LIBRARY_PATH` line
THEN Mantle MUST abort the first-stage build with a deterministic diagnostic before invoking `run_rustc` for the compiler host.

#### Scenario: generic host aliases remain host-oriented

GIVEN the generated first-stage Rust provider script prepares musl proc-macro runtime visibility
WHEN the private runtime search path is installed
THEN Mantle MUST NOT replace global `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases with source-root musl target tools.
AND ambient Cargo rustc wrappers MUST be scrubbed before first-stage build commands run.
