## ADDED Requirements

### Requirement: Source-built Rust bootstrap patch-plan boundary

r[rust_package_planning.source_built_toolchain_closure.bootstrap_patch_plan_boundary] Mantle MUST isolate source-built Rust compiler-bootstrap repair decisions in a deterministic patch-plan boundary before provider materialization mutates source trees or emits generated shell.

#### Scenario: patch-plan core derives operations from explicit facts

GIVEN Mantle selects a source-built Rust provider route with explicit Rust version, mrustc version, source identities, compiler-host triple, provider-target triple, and route capabilities
WHEN Mantle determines compiler-bootstrap repairs for that route
THEN it MUST derive an ordered patch plan from those explicit facts.
AND the patch-plan derivation MUST avoid filesystem reads, process execution, environment reads, clocks, and mutation.
AND the patch plan MUST carry a deterministic BLAKE3 digest over canonical plan inputs and ordered operation outputs.

#### Scenario: provider shell applies operations fail-closed

GIVEN a source-built Rust provider materialization receives a patch plan
WHEN the provider shell applies source, makefile, wrapper, relink, helper-object, or metadata operations
THEN it MUST apply the operations in order.
AND it MUST verify every expected anchor or source identity before mutation.
AND it MUST fail closed with deterministic diagnostics when an expected anchor, source identity, or supported operation kind is missing.

#### Scenario: patch-plan evidence is receipt-bound

GIVEN a source-built Rust provider materialization applies a patch plan
WHEN Mantle records provider evidence or metadata
THEN it MUST bind the patch-plan input digest, output digest, operation summaries, route facts, compiler versions, and source identities.
AND downstream claims about the provider MUST be reviewable without reading ad-hoc generated shell fragments.

### Requirement: Source-built Rust provider contract independence

r[rust_package_planning.source_built_toolchain_closure.provider_contract_independence] Mantle MUST keep downstream Rust planning, topology execution, and self-build consumers coupled to a stable source-built toolchain provider contract rather than compiler-bootstrap implementation details.

#### Scenario: downstream consumers use normalized provider capabilities

GIVEN Mantle has materialized a source-built Rust provider
WHEN native Rust planning, topology execution, or self-build code consumes that provider
THEN those consumers MUST use normalized provider capabilities such as executable paths, host and target triples, sysroot root, proc-macro runtime support, static and dynamic linking policy, source provenance, and provider digests.
AND those consumers MUST NOT branch on mrustc, minicargo, LLVM, or generated-wrapper repair internals.

#### Scenario: compiler-bootstrap repairs stay behind the provider boundary

GIVEN a compiler-bootstrap repair changes for mrustc, minicargo, LLVM, dynamic musl `rustc`, proc-macro loading, or runtime wrapper behavior
WHEN the source-built Rust provider still satisfies the same normalized provider contract
THEN Mantle MUST NOT require native Rust planning, topology execution, or self-build consumer code changes solely because that repair changed.
AND any provider-contract migration MUST be explicit, versioned, and receipt-bound.
