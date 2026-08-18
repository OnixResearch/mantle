## ADDED Requirements

### Requirement: Receipt-bound target tool aliases

r[rust_package_planning.source_built_toolchain_closure.target_aliases] Mantle MUST expose target-prefixed native tool names from the receipt-bound toolchain closure manifest before running Cargo-free topology units.

#### Scenario: Target compiler name is declared by the closure

GIVEN a source-built toolchain closure manifest contains an executable member named `x86_64-linux-musl-gcc`
WHEN Mantle constructs the receipt-bound PATH for Cargo-free topology execution
THEN it MUST create a PATH entry named `x86_64-linux-musl-gcc` that dispatches to the declared executable path.
AND the alias MUST be present even when the declared executable file has a different basename.

#### Scenario: Cargo remains guarded

GIVEN a source-built toolchain closure manifest declares an executable member named `cargo`
WHEN Mantle constructs the receipt-bound PATH
THEN it MUST fail before executing any Rust unit.
AND it MUST NOT replace the Cargo guard shim with a toolchain closure alias.

#### Scenario: Alias conflicts fail closed

GIVEN two executable toolchain closure members declare the same alias name but different executable paths
WHEN Mantle constructs the receipt-bound PATH
THEN it MUST fail with a deterministic alias-conflict blocker.

#### Scenario: Target proof does not inherit ambient PATH

GIVEN a musl-target Cargo-free fixed-point proof is launched with `--toolchain-closure`
WHEN a build script requests `x86_64-linux-musl-gcc`
THEN the request MUST resolve only through the receipt-bound PATH alias or fail as a host-tool-leakage/tool-missing blocker.
AND Mantle MUST NOT satisfy the request from `/nix/var/nix/profiles`, `/run/current-system/sw`, rustup, or another ambient PATH entry.
