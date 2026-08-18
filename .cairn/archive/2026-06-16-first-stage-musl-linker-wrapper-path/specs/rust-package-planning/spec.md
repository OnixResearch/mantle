# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: First-stage musl linker wrapper path

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_linker_wrapper_path] Mantle MUST place generated private musl linker wrappers on PATH before first-stage Rust provider `run_rustc` target links.

#### Scenario: Plain cc resolves to private musl wrapper

GIVEN the generated first-stage script selects the musl target linker wrapper branch
WHEN it creates `$BUILD_DIR/target-linker-bin/cc`
THEN it MUST prepend that private alias directory to PATH before invoking `run_rustc`.
AND a child `rustc` link that asks for plain `cc` MUST resolve inside the generated private alias directory.

#### Scenario: Wrapper remains script-scoped

GIVEN the generated first-stage script prepends the private target linker alias directory
WHEN the script exits
THEN Mantle MUST NOT persist that private `cc` alias as an ambient host tool.
AND target tool exposure MUST remain limited to the generated script process tree.
