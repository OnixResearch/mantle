# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Source-root musl Rust provider tools

r[rust_package_planning.source_built_toolchain_closure.source_root_musl_rust_provider_tools] Mantle MUST recognize the source-root musl toolchain layout as a valid tool source for Rust provider bootstrap scripts targeting `x86_64-unknown-linux-musl`.

#### Scenario: Source-root musl prefixes are target aliases

GIVEN a Rust source provider route targets `x86_64-unknown-linux-musl`
WHEN Mantle generates first-stage or Rust-source bootstrap scripts
THEN the scripts MUST search for both `x86_64-unknown-linux-musl-*` and `x86_64-linux-musl-*` target-prefixed tools.
AND they MUST accept both `x86_64-unknown-linux-musl` and `x86_64-linux-musl` `gcc -dumpmachine` values for that target.

#### Scenario: Source-root sysroot is accepted without Nix wrapper metadata

GIVEN the selected target compiler lives under a source-root musl toolchain without `nix-support/orig-libc`
WHEN Mantle configures the Rust source provider target toolchain
THEN it MUST derive the musl sysroot from the selected tool root's `x86_64-linux-musl/` directory.
AND it MUST require libc and CRT files before exporting that target tool configuration.

#### Scenario: Generic host aliases remain host-oriented

GIVEN source-root musl target aliases are available
WHEN Mantle prepares provider bootstrap execution
THEN it MUST NOT replace generic host `cc`, `ld`, or `ld.lld` aliases with musl target tools.
AND target tool exposure MUST remain scoped to target-prefixed names or generated private wrapper directories.
