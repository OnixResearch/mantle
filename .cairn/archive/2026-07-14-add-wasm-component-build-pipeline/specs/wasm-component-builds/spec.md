# Wasm Component Builds Specification

## Purpose

Build, resolve, compose, virtualize, validate, transform, optionally precompile, and materialize WebAssembly components as explicit Mantle stages while preserving digest roles and leaving runtime authority and evidence semantics to their owners.

## ADDED Requirements

### Requirement: Component build configuration is Nickel-authored

r[mantle.wasm_component.config] Mantle MUST define component package/build, WIT/world, dependency, composition, WASI virtualization, Octet artifact-profile identity, expected runtime-profile identity, Wizer, AOT, output, and non-claim configuration in typed Nickel and MUST generate deterministic tool-native inputs where external tools require them.

#### Scenario: Component manifest is valid
- GIVEN a complete Nickel component-build manifest satisfies its contracts
- WHEN Mantle plans the build
- THEN it MUST emit one deterministic plan with identified generated tool inputs.

#### Scenario: Generated input is stale
- GIVEN `wkg.toml`, WAC input, command plan, or runtime DTO does not match its Nickel source/dependency identity
- WHEN freshness validation runs
- THEN Mantle MUST fail closed before package resolution or build execution.

### Requirement: wkg resolution produces immutable build inputs

r[mantle.wasm_component.package_resolution] Mantle MUST resolve WIT and reusable component-library packages through an explicit wasm-pkg-tools/wkg configuration, MUST verify package versions and registry digests, MUST persist a checked `wkg.lock`, and MUST materialize exact immutable store inputs before offline component builds.

#### Scenario: Package resolves successfully
- GIVEN an admitted registry mapping, semantic requirement, credential handle when required, and matching downloaded digest
- WHEN resolution completes
- THEN Mantle MUST store exact package bytes and a lock entry that binds registry, package, version, protocol digest, and Mantle BLAKE3.

#### Scenario: Lock or registry result drifts
- GIVEN a selected version, package bytes, registry mapping, or digest differs from the checked lock
- WHEN resolution or offline preflight runs
- THEN Mantle MUST deny rather than silently refreshing or building with new bytes.

### Requirement: Interoperability and canonical digests stay separate

r[mantle.wasm_component.digest_roles] Mantle MUST preserve OCI-required SHA-256 values as external interoperability metadata and MUST use BLAKE3 for Mantle-owned source, package-byte, lock, plan, component, transform, precompile, report, and attestation identity.

#### Scenario: OCI digest and bytes match
- GIVEN fetched bytes match the protocol SHA-256
- WHEN Mantle admits the package
- THEN it MUST also compute and retain a distinct BLAKE3 artifact identity.

#### Scenario: External digest is used as Mantle identity
- GIVEN a manifest places an OCI SHA-256 value in a Mantle BLAKE3 identity role
- WHEN validation runs
- THEN Mantle MUST reject the role confusion.

### Requirement: Component tools form one pinned cohort

r[mantle.wasm_component.cohort] Mantle MUST pin and identify one compatible Rust target/toolchain, wit-bindgen, wasm-component-ld, wasm-tools, WAC, WASI-Virt, Wizer, and Wasmtime cohort for each component build profile.

#### Scenario: Cohort member changes
- GIVEN any component tool, target, feature set, or configuration changes
- WHEN cohort identity is recomputed
- THEN Mantle MUST produce a new identity and MUST invalidate prior generated outputs until conformance validation reruns.

### Requirement: Portable components build offline from declared inputs

r[mantle.wasm_component.build] Mantle MUST compile portable components from declared source, toolchain, WIT, package, and generated binding inputs with ordinary build network access denied.

#### Scenario: Rust component builds
- GIVEN all source, toolchain, WIT, binding, and package inputs are present in the store
- WHEN the component derivation runs
- THEN it MUST build offline and record every input identity.

#### Scenario: Build seeks ambient registry or Cargo state
- GIVEN compilation attempts undeclared network, registry cache, user config, credential, or host tool access
- WHEN sandbox policy evaluates the request
- THEN Mantle MUST deny or classify the output as ineligible under the strict component profile.

### Requirement: WAC composition uses exact local dependencies

r[mantle.wasm_component.composition] Mantle MUST compose components through a declared WAC graph whose dependencies resolve to exact local store artifacts and MUST deny archived Warg lookup, implicit live registry resolution, duplicate bindings, missing imports, and wrong-world edges.

#### Scenario: Composition graph is complete
- GIVEN every WAC node and edge resolves to a matching local component or WIT package
- WHEN WAC composition runs
- THEN the output receipt MUST bind the graph, inputs, WAC cohort, and output BLAKE3.

#### Scenario: Composition would use implicit registry state
- GIVEN a dependency is not present locally and WAC would resolve it through Warg or another live registry path
- WHEN composition preflight runs
- THEN Mantle MUST deny before invoking the composer.

### Requirement: WASI virtualization starts deny-all

r[mantle.wasm_component.virtualization] Mantle MUST construct WASI-Virt plans from an explicit deny-all baseline, MUST identify every allow/ignore/fixed-value/mount/passthrough rule and embedded input, and MUST inspect the final component for remaining imports.

#### Scenario: Reviewed virtual filesystem is embedded
- GIVEN a declared virtual mount contains identified immutable input bytes and no host passthrough
- WHEN virtualization runs
- THEN the receipt MUST bind the mount plan, embedded content identities, virtualizer cohort, and final imports.

#### Scenario: Library defaults would pass through authority
- GIVEN direct WASI-Virt library construction leaves an undeclared subsystem in pass-through mode
- WHEN plan validation runs
- THEN Mantle MUST reject the plan before composition.

### Requirement: Portable outputs pass independent build and Octet validation

r[mantle.wasm_component.validation] Every portable component output, including identity-changing WAC, WASI-Virt, and Wizer outputs, MUST pass pinned build-local wasm-tools validation before publication or later-stage use; consumer and release profiles MUST also run the declared Octet artifact rail over each exact portable output admitted into the materialization bundle and bind its independent profile, cohort, and report identity.

#### Scenario: Output matches both declared profiles
- GIVEN a compiled, composed, virtualized, or Wizer-transformed portable component passes build-local validation and the Octet rail reports that the same exact bytes satisfy the declared artifact profile
- WHEN output validation completes
- THEN Mantle MAY admit those bytes to the next stage or materialization bundle and MUST persist both independently identified reports.

#### Scenario: Link succeeds but a validation layer differs
- GIVEN the compiler/composer exits successfully but build-local facts violate policy, or the Octet report names different bytes, profile, cohort, world, imports, proposals, or resource declarations
- WHEN validation runs
- THEN Mantle MUST deny the artifact from later stages without reinterpreting the Octet finding.

### Requirement: Wizer is a bounded optional transform

r[mantle.wasm_component.wizer] Mantle MUST run Wizer only with imports denied or deterministically virtualized, MUST bind original/input/tool/output identities, and MUST require repeated exact output agreement when deterministic eligibility is requested.

#### Scenario: Repeated Wizer output matches
- GIVEN identical admitted input and virtual state
- WHEN the configured clean transform repeats
- THEN output BLAKE3 values MUST match before the transformed artifact is eligible.

#### Scenario: Initialization observes ambient state
- GIVEN initialization can observe undeclared filesystem, environment, clock, random, network, process, or credential state
- WHEN transform admission runs
- THEN Mantle MUST deny or mark the output diagnostic-only.

### Requirement: Precompiled artifacts are target-specific trusted outputs

r[mantle.wasm_component.precompile] Mantle MUST precompile only validated portable components and MUST bind `.cwasm` bytes to the source component, full Wasmtime configuration, target, CPU features, cohort, WIT profile, and build inputs while labeling the output as target-specific native trusted code.

#### Scenario: Precompile succeeds
- GIVEN a validated component and complete target/runtime configuration
- WHEN Wasmtime precompilation runs
- THEN the receipt and attestation MUST bind source and output BLAKE3 plus every compatibility fact.

#### Scenario: Precompiled artifact is cross-target or tampered
- GIVEN `.cwasm` bytes, target, CPU features, cohort, or runtime configuration do not match their receipt
- WHEN release or consumer verification runs
- THEN verification MUST fail closed.

### Requirement: Consumers receive one rehashable materialization bundle

r[mantle.wasm_component.bundle] Mantle MUST emit a versioned component materialization bundle binding exact store objects and BLAKE3 identities for WIT/package inputs, source closure, lock, final portable bytes, build cohort, expected Octet/runtime profiles, every stage receipt, and any optional Wizer or precompiled outputs; paths and names MUST remain locator metadata only.

#### Scenario: Complete bundle is handed to a consumer
- GIVEN a component pipeline produces all required objects and reports
- WHEN Mantle materializes the consumer bundle
- THEN every object MUST be independently rehashable and linked to its producing stage so the consumer can remeasure and perform its own admission.

#### Scenario: Bundle swaps, omits, or circularly embeds evidence
- GIVEN a bundle omits a required profile/report/parent, names bytes with a mismatched BLAKE3, substitutes a store path for exact identity, or includes a post-materialization Valence/Cairn identity that itself references the bundle
- WHEN bundle verification runs
- THEN Mantle MUST fail closed and MUST NOT publish the bundle as consumer-admissible.

### Requirement: Component build evidence follows the stage graph

r[mantle.wasm_component.evidence] Mantle MUST expose package resolution, lock, binding, compilation, composition, virtualization, build-local validation, Octet validation, Wizer, AOT, and materialization stages as typed build-report/attestation nodes with BLAKE3 parent links and explicit non-claims.

#### Scenario: Build report is reviewed
- GIVEN a component pipeline completes or fails
- WHEN its machine report is rendered
- THEN each attempted stage MUST have a stable status, input/output links, tool/profile identity, and bounded claim.

#### Scenario: Evidence claims runtime authority or correctness
- GIVEN a build artifact claims that WIT conformance, virtualization, matching bytes, or precompilation proves runtime authority, behavior, or release eligibility
- WHEN report validation runs
- THEN Mantle MUST reject the overclaim.

### Requirement: Component planning has a functional core

r[mantle.wasm_component.functional_core] Manifest/lock validation, digest-role separation, cohort identity, source and command planning, composition graph validation, virtualization planning, build/Octet result binding, transform comparison, precompile manifest checks, materialization-bundle construction, and report construction MUST be pure deterministic logic.

#### Scenario: Identical build facts produce identical plan
- GIVEN identical normalized manifest, lock, package, source, cohort, composition, virtualization, and target facts
- WHEN planning runs
- THEN it MUST return the same plan or blockers without filesystem, network, credential, process, clock, sandbox, store, or output effects.

### Requirement: Component pipeline has positive and negative validation

r[mantle.wasm_component.final_validation] The pipeline MUST include positive build/composition/virtualization/materialization cases and negative registry, lock, dependency, world, implicit-resolution, pass-through, import, Octet-report, transform, precompile, bundle, report, and overclaim cases plus focused lifecycle validation.

#### Scenario: Pipeline change is reviewed
- GIVEN manifest, resolver, tool cohort, build, composition, virtualization, validation, transform, precompile, or evidence behavior changes
- WHEN validation evidence is assembled
- THEN it MUST include positive and negative fixtures, focused core/shell tests, Cairn gates, first-party quality checks, and relevant Nix checks.
