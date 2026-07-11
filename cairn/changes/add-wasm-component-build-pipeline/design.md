## Context

The active Bytecode Alliance component toolchain divides responsibilities cleanly: `wkg` resolves and publishes WIT/component libraries through OCI; Rust `wasm32-wasip2`, wit-bindgen, and wasm-component-ld produce components; WAC composes components; WASI-Virt reduces or virtualizes WASI imports; wasm-tools validates and inspects outputs; Wizer preinitializes portable Wasm; Wasmtime can precompile portable components into target-specific native artifacts.

Mantle should orchestrate those tools as explicit derivations and evidence stages. It should not absorb their configuration formats as Mantle's canonical policy or rely on ambient registry/toolchain state.

## Decisions

### 1. Nickel owns the component build manifest

**Choice:** Define a typed Nickel manifest for package sources, WIT package/world, implementation source, compatibility cohort, target/profile, dependency requirements, composition graph, WASI virtualization policy, Octet artifact-profile identity, expected runtime-profile identity, optional Wizer/AOT stages, outputs, and non-claims. Deterministic exports produce tool-native `wkg.toml`, WAC source/AST input, and command plans where required; generated files carry source/export receipts.

**Rationale:** Humans need typed mergeable configuration, while WIT/WAC/OCI tools must continue receiving their native formats.

### 2. Package resolution is a source-acquisition boundary

**Choice:** `wkg` resolution occurs in a network-admitted shell/fetch stage with an explicit config path, registry mapping, package requirements, credential handles, and retry policy. It writes a checked `wkg.lock`, downloads exact WIT/component-library bytes into immutable Mantle store inputs, and records registry/package/version/digest metadata. Ordinary component builds run offline from those inputs.

**Rationale:** Registry access is analogous to a fixed-output source fetch, not an ambient build capability.

### 3. Digest roles remain explicit

**Choice:** Preserve OCI and `wkg.lock` SHA-256 values exactly as protocol-required external digests. Compute BLAKE3 over the fetched bytes, lock bytes, WIT source, build inputs, WAC plan, WASI-Virt plan, portable components, transformed components, precompiled outputs, and receipts. Never substitute an external SHA-256 for Mantle artifact identity.

**Rationale:** Interoperability requirements do not change the stack-owned hashing default.

### 4. The entire component toolchain is one cohort

**Choice:** Pin Rust target/toolchain, wit-bindgen, wasm-component-ld, wasm-tools, WAC, WASI-Virt, Wizer, and Wasmtime versions plus target/feature configuration as one identified cohort. Cohort upgrades invalidate generated bindings, validation baselines, composition outputs, transforms, and precompiled outputs until fixtures rerun.

**Rationale:** These tools share evolving WIT and Component Model assumptions.

### 5. WAC composition consumes exact local dependencies

**Choice:** Resolve every composition dependency to an exact store object before invoking WAC. The composition plan binds package/world identities, local artifact BLAKE3 values, instantiation/wiring edges, exported world, embed/import posture, and output. Archived Warg lookup and unpinned implicit registry resolution are denied.

**Rationale:** A composition must be reproducible without live resolver state.

### 6. WASI-Virt starts from deny-all

**Choice:** The pure plan starts every subsystem denied, then applies explicit allow, ignore, fixed-value, virtual mount, or reviewed passthrough rules. Direct WASI-Virt library use must override its pass-through defaults before any rule is applied. Embedded files and generated virtual values become identified build inputs. The final artifact is re-inspected to verify remaining imports.

**Rationale:** Build-time virtualization reduces ambient authority only when defaults and embedded inputs are explicit.

### 7. Portable output validation has independent build and policy layers

**Choice:** Validate each portable component with the pinned build-local wasm-tools cohort before it is published or used as input to Wizer/AOT stages. Consumer and release profiles then invoke the pinned Octet artifact rail over the exact final portable bytes and declared Octet profile, retaining both reports and their independent cohort identities. Mantle orchestrates and binds the Octet result but does not reinterpret its findings or replace Octet policy with build-manifest fields.

**Rationale:** Build-local validation catches malformed or mismatched output immediately, while an independent Octet policy result prevents every producer from inventing a different static acceptance vocabulary.

### 8. Wizer is an optional deterministic transform

**Choice:** Run Wizer only in a bounded build sandbox with imported functions denied unless deterministic virtual implementations are declared. Execute repeated clean transforms when the profile requests deterministic eligibility and require exact output BLAKE3 agreement. Record original, initialization entrypoint, virtual inputs, tool cohort, output, and non-claims.

**Rationale:** Preinitialization can capture ambient state or secrets and changes artifact bytes.

### 9. Wasmtime precompilation creates a native trust artifact

**Choice:** Precompile only a portable component that passed validation. The receipt binds source component BLAKE3, output `.cwasm` BLAKE3, full Wasmtime configuration, target, CPU features, cohort, WIT profile, and build inputs. Release manifests label `.cwasm` as target-specific trusted native code and require exact verification before consumers deserialize it.

**Rationale:** Wasmtime cannot fully validate arbitrary precompiled native bytes at load time.

### 10. A materialization bundle is the consumer handoff

**Choice:** Emit a versioned component materialization bundle that binds exact store objects and BLAKE3 identities for WIT/package inputs, source closure, lock, final portable component, build cohort, expected Octet and runtime profile identities, every stage receipt, and optional Wizer or `.cwasm` outputs. Store paths and package names are locator metadata only. Consumers remeasure exact bytes and perform their own admission, but production components are not rebuilt ad hoc in runtime, adapter, benchmark, or exploration repositories.

**Rationale:** One rehashable handoff prevents each consumer from rebuilding nominally identical components with different tools or losing the derivation chain between portable and native outputs.

### 11. Evidence follows the build stage graph

**Choice:** Build reports and attestations expose package resolution, lock, source, binding generation, compilation, composition, virtualization, build-local validation, Octet validation, Wizer, AOT, and materialization-bundle nodes plus typed parent edges and non-claims. Valence sidecars remain opaque external evidence when bundled; Mantle does not interpret their semantics.

**Rationale:** A component filename, store path, or WIT world alone cannot identify what was materialized or which downstream admission remains required.

## Functional core / imperative shell split

- **Pure core**: manifest/lock validation, digest-role separation, cohort identity, source and command planning, composition graph validation, deny-all virtualization planning, build/Octet result binding, Wizer comparison, AOT manifest checks, materialization-bundle construction, and report construction.
- **Imperative shell**: registry/network access, credential-handle resolution, file/materialization effects, tool and Octet execution, sandboxing, byte hashing, store persistence, signing, and report output.

## Risks / Trade-offs

- Component tooling evolves quickly and can produce large dependency closures. Cohort pins and dedicated update changes make that cost visible.
- Bundle consumers and independent Octet/Aspen profiles can evolve at different rates. Exact referenced profile identities and explicit incompatibility failures are preferable to hidden fallback.
- OCI registries vary in authentication and metadata behavior. Keep resolution explicit and fail closed on digest/version/config drift.
- WASI-Virt support is evolving and build-time virtualization is not a complete runtime sandbox.
- Repeated Wizer transforms and AOT matrices can be expensive. Keep optional fast/deep profiles and reuse only exact admitted outputs.

## Non-Goals

- No component runtime, hostcall authorization, distributed transport, or application semantics in Mantle.
- No replacement of Mantle store/release artifacts with OCI deployable artifacts; `wkg` is for WIT and reusable component-library packages in this pipeline.
- No archived Warg registry dependency and no cargo-component requirement for ordinary native `wasm32-wasip2` builds.
- No claim that WIT conformance, deterministic bytes, an Octet pass, a materialization bundle, Wizer output, or AOT provenance proves component behavior, runtime authority, or release eligibility.
