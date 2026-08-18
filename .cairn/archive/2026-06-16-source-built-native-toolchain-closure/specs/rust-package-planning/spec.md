## ADDED Requirements

### Requirement: Explicit source-built native closure promotion

r[rust_package_planning.source_built_toolchain_closure.explicit_native_promotion] Mantle MUST promote an explicitly supplied toolchain closure to a source-built toolchain claim only after enforcement proves a zero-seed, all-source-built closure.

#### Scenario: Enforced complete explicit closure claims source-built closure

GIVEN a toolchain closure manifest has no seed exceptions
AND every manifest member is classified as source-built
WHEN Mantle enforces observed Cargo-free execution inputs against that manifest
THEN the resulting source-built toolchain closure status MUST set `claim` to true.
AND it MUST omit `not-source-built-toolchain-closure` from the non-claims.

#### Scenario: Validated-only complete explicit closure does not claim

GIVEN a toolchain closure manifest has no seed exceptions
AND every manifest member is classified as source-built
WHEN Mantle validates the manifest without enforcing observed execution inputs
THEN the resulting source-built toolchain closure status MUST keep `claim` false.
AND it MUST keep `not-source-built-toolchain-closure` in the non-claims.

#### Scenario: Any seed exception preserves the non-claim

GIVEN a toolchain closure manifest contains one or more seed exceptions
WHEN Mantle enforces observed Cargo-free execution inputs against that manifest
THEN the resulting source-built toolchain closure status MUST keep `claim` false.
AND it MUST keep `not-source-built-toolchain-closure` in the non-claims.
AND it MUST report the seed exception count.

#### Scenario: Current native closure frontier is not overclaimed

GIVEN the current source-built Rust provider and receipt-bound musl target aliases
WHEN Mantle attempts a no-seed native closure proof
THEN the evidence MUST record either a successful zero-seed fixed point or the exact deterministic blocker.
AND Mantle MUST NOT relabel Nix clang, glibc, pkg-config, rustup, or other host tools as source-built to satisfy the requirement.
