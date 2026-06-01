## ADDED Requirements

### Requirement: Source-built Rust toolchain closure proof

r[rust_package_planning.source_built_toolchain_closure] Mantle MUST provide a separate audit-grade proof before claiming that a Cargo-free self-build or fixed-point run used a source-built compiler/toolchain closure.

#### Scenario: toolchain closure is receipt-bound

GIVEN Mantle is launched in source-built toolchain closure proof mode
WHEN it selects compiler and native toolchain inputs for Rust topology execution
THEN the proof bundle MUST record each compiler, linker, C toolchain, sysroot, crt object, runtime library, and native helper tool with role, source identity, build receipt identity, execution path, BLAKE3 content digest, and trust classification.
AND every non-seed toolchain member MUST have source provenance that is reachable from the proof bundle.

#### Scenario: ambient host toolchain is rejected

GIVEN source-built toolchain closure proof mode is active
WHEN Mantle would use a host `rustc`, Cargo, linker, C compiler, pkg-config, Nix profile tool, PATH helper, or undeclared sysroot member that is not listed in the receipt-bound toolchain closure
THEN Mantle MUST fail before executing the affected Rust unit with a deterministic host-tool-leakage blocker.
AND it MUST NOT report a source-built toolchain closure claim.

#### Scenario: seed exceptions are explicit

GIVEN the proof needs an initial seed or trust root
WHEN the proof bundle is written
THEN every seed exception MUST be named, justified, BLAKE3 hashed, trust-classified, and excluded from the source-built portion of the claim.
AND placeholder providers, missing source-root contracts, or unverified seed metadata MUST fail closed.

#### Scenario: fixed-point stages use the receipt-bound closure

GIVEN source-built toolchain closure proof mode is combined with the Cargo-free fixed-point command
WHEN stage1 and stage2 Mantle builds execute
THEN both stages MUST use the receipt-bound toolchain closure policy rather than ambient host toolchain discovery.
AND the proof MUST report success only when both stages succeed, both stage Cargo guards remain untriggered, both stages record the same closure policy digest, and the stage1/stage2 Mantle binary BLAKE3 digests match.

#### Scenario: bounded non-claims remain visible

GIVEN a source-built toolchain closure proof completes successfully
WHEN the audit bundle is reviewed
THEN it MUST state any remaining non-claims, including whether the proof is not full release reproducibility, not full Cargo compatibility, and not a fully minimized bootstrap trust root when seed exceptions remain.
AND existing Cargo-free fixed-point proofs that lack source-built toolchain closure evidence MUST continue to state `not-source-built-toolchain-closure`.
