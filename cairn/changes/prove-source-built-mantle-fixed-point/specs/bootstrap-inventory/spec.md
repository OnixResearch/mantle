## ADDED Requirements

### Requirement: Source-built Mantle fixed-point proof

r[bootstrap_inventory.source_built_mantle_fixed_point] Mantle MUST claim a full-source fixed point only when one authenticated proof authority constructs or revalidates the complete StageX native provider and full-source-bound Rust provider, builds stage1 Mantle, and uses stage1 Mantle to build a BLAKE3-identical stage2 through the same Cargo-free, strict-hermeticity, no-fetch, and no-fallback policy.

#### Scenario: proof inputs are sources rather than provider outputs

GIVEN a fresh proof root and independently authenticated source bundle
WHEN the full-bootstrap proof begins
THEN it MUST validate the audited seed, lineage manifest, native and Rust source records, vendor inputs, source-state identity, policy identities, resource bounds, and empty native-provider, Rust-provider, and Mantle output authorities before construction
AND an imported or cache-hit native provider, Rust provider, Mantle binary, missing source, wrong digest, stale state, or producer-checkout path authority MUST fail preflight.

#### Scenario: stage1 uses the constructed closure

GIVEN the StageX native provider and full-source-bound Rust provider were constructed from source inside the current proof authority
WHEN stage1 Mantle is planned and built
THEN every planner, rustc, build script, proc macro, native compiler/linker, runtime, and source read MUST be authorized by the immutable receipt-bound closure and source policy
AND Cargo invocation, live fetch, ambient executable/read discovery, practical hermeticity fallback, or protected-exec violation MUST fail stage1.

#### Scenario: stage1 builds stage2

GIVEN stage1 Mantle builds and smokes successfully
WHEN stage2 starts
THEN the produced stage1 Mantle binary MUST plan and execute stage2 using the same source-state, provider, closure-policy, Cargo-free, hermeticity, and protected-execution identities
AND the host Mantle binary, a different provider/closure, or changed authority plan MUST NOT substitute for stage1.

#### Scenario: fixed point emits v2 evidence

GIVEN both stages complete
WHEN Mantle evaluates fixed-point success
THEN stage1 and stage2 Mantle binary BLAKE3 digests MUST match and a verified `mantle-deterministic-proof-receipt-v2` MUST bind source/rebuild descriptors, authority plan, provider/closure identities, stage receipts, approved reads, effects, audits, outputs, and final proof-bundle digest
AND any mismatch, malformed receipt, fallback, unapproved effect, or incomplete evidence MUST preserve a failed proof without updating successful aliases.

#### Scenario: fixed-point claim remains bounded

GIVEN the full-source fixed point succeeds
WHEN operators or release tooling cite it
THEN the claim MUST identify seed, source, lineage, native/Rust providers, closure policy, orchestration authority, platform, stage outputs, audits, and matching digests
AND it MUST NOT claim compiler correctness, seed correctness, kernel isolation, independent rebuild agreement, bit-for-bit release reproducibility, deployment success, or full Cargo compatibility.