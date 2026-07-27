## ADDED Requirements

### Requirement: Full-source native provider binds the Rust bootstrap

r[bootstrap_inventory.full_source_rust_provider_binding] Mantle MUST build any Rust provider used for a full-bootstrap claim from the admitted full-source native provider and authenticated Rust sources without prebuilt Rust, host-assisted source-root substitution, or undeclared compiler, linker, libc, runtime, package-tool, or source acquisition fallback.

#### Scenario: admitted native provider constructs Rust

GIVEN the selected full-source native provider and authenticated Rust bootstrap sources are available
WHEN Mantle materializes the full-bootstrap Rust provider
THEN every Rust stage MUST bind the admitted native-provider identity, source-closure identity, selected compiler-host and target triples, declared native tools and runtimes, Rust stage source identities, and produced artifact BLAKE3 digests
AND the final provider MUST contain validated `rustc`, Cargo, rustdoc, compiler-host rustlib, musl target rustlib, metadata, and construction receipts.

#### Scenario: compiler-host strategy fails closed

GIVEN the preferred musl compiler-host route cannot satisfy Rust compiler, dylib, proc-macro, linker, or runtime requirements
WHEN Mantle evaluates an alternate compiler-host route
THEN the alternate route MUST provide its own complete authenticated source-built native closure before execution
AND Mantle MUST reject Nix, rustup, ambient GNU, imported binary, wrapper delegation, or the older host-assisted source-root closure as full-bootstrap completion.

#### Scenario: compatibility Rust remains outside the claim

GIVEN an operator selects the fetched standalone Rust compatibility route
WHEN Mantle builds or reports bootstrap status
THEN the route MUST remain usable only under its explicit compatibility identity
AND receipts and parity reporting MUST state that the result does not satisfy full-source Rust or full-bootstrap evidence.

#### Scenario: Rust bootstrap claim remains bounded

GIVEN the full-source native provider successfully constructs and smokes the Rust provider
WHEN the result is cited
THEN the claim MUST identify the native provider, native source closure, Rust sources, host/target strategy, stage receipts, final artifact identities, environmental assumptions, and smoke results
AND it MUST NOT claim Rust compiler correctness, native compiler correctness, seed correctness, independent rebuild agreement, release reproducibility, deployment success, or full Cargo compatibility.