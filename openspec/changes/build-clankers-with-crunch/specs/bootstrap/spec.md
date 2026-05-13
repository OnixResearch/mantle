## ADDED Requirements

### Requirement: External Rust workspace builds use fixed offline source closures

Crunch MUST support an external Rust workspace build pattern where every workspace source, path dependency, git dependency, registry crate, and tool input is represented by an explicit fixed source closure before derivation execution.
ID: bootstrap.external-rust-workspace.offline-source-closure

The build MUST run Cargo with network disabled, a writable sandbox-local `CARGO_HOME`, a deterministic `CARGO_TARGET_DIR`, and a checked `.cargo/config.toml` or equivalent source replacement that points only at fixed inputs. Host checkout-relative paths MAY be used only to construct the fixed source closure outside the derivation; derivation execution MUST NOT read undeclared sibling checkouts, live git remotes, or ambient Cargo caches.

#### Scenario: Offline source closure builds a simple crate

- GIVEN a fixed source closure for an external Rust workspace
- AND the closure includes registry crates, git dependencies, path dependencies, and workspace sources required by a selected package
- WHEN Crunch builds the package derivation with `CARGO_NET_OFFLINE=true` and `cargo build --locked --offline -p <package>`
- THEN Cargo does not access the network or ambient Cargo caches
- AND the selected package builds successfully from declared inputs only

#### Scenario: Missing git dependency is rejected before success is claimed

- GIVEN a selected package depends on a git source that is not present in the fixed source closure
- WHEN the package derivation or its preflight validation runs
- THEN the build fails before any success receipt is written
- AND the diagnostic names the missing git dependency or source replacement

#### Scenario: Ambient sibling checkout is not accepted as derivation input

- GIVEN a workspace package has a path dependency such as `../subwayrat` or `../ratcore`
- WHEN the Crunch derivation executes
- THEN it reads the dependency from a declared fixed input
- AND it does not read the live sibling checkout path from the host filesystem

### Requirement: Clankers build ladder starts with low-dependency crates

Crunch MUST build `../../clankers/` through an ordered Clankers build ladder that proves small workspace packages before attempting the root `clankers` binary.
ID: bootstrap.external-rust-workspace.clankers-ladder

The first rung MUST target a low-dependency package such as `clanker-message`, using `bootstrap/rust.ncl`, a fixed Clankers source closure, and offline Cargo. Each later rung MUST add only the source closure entries and native build tools required by that rung. The root `clankers` binary success claim MUST require an installed binary under `$out/bin/clankers` plus a non-network smoke check such as `clankers --help` or `clankers --version`. Full NixOS VM checks, plugin bundle builds, source-built Rust proof, and optional heavyweight runtime integrations are separate follow-up claims unless they are required for the root binary to compile.

#### Scenario: First rung builds clanker-message

- GIVEN a Crunch derivation for the Clankers `clanker-message` package
- AND a fixed source/vendor closure sufficient for that package
- WHEN `crunch build` runs the derivation with `cargo build --locked --offline -p clanker-message`
- THEN the derivation succeeds
- AND the output records the package name, Cargo command, source closure digest, vendor closure digest, and built artifact path

#### Scenario: Native build-script tool is introduced only when needed

- GIVEN a later Clankers rung fails because a package build script requires a native tool such as `cmake`, `go`, `pkg-config`, a C compiler, or onnxruntime headers/libraries
- WHEN the next derivation revision addresses the failure
- THEN it adds the smallest explicit Crunch input required by that build script
- AND earlier rungs remain buildable without that new input unless Cargo's dependency graph requires it

#### Scenario: Root clankers binary claim requires executable smoke

- GIVEN the Clankers root binary derivation succeeds
- WHEN the output is installed
- THEN `$out/bin/clankers` exists and is executable
- AND a sandbox-local smoke check runs without network access and records the observed `--help` or `--version` output
