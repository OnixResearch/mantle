# ADR 0016: Materialize WebAssembly components as build artifacts

## Status

Proposed

## Context

WebAssembly Component Model projects need more than a compiler invocation. A
repeatable producer must resolve WIT and reusable component-library packages,
generate bindings, compile, compose exact component dependencies, reduce WASI
imports, validate every identity-changing portable output, optionally
preinitialize or precompile an artifact, and preserve evidence for the complete
stage graph.

Those steps cross several independently versioned Bytecode Alliance tools and
two trust domains. OCI registries and `wkg.lock` require SHA-256 protocol
digests, while Mantle-owned object and receipt identities use BLAKE3. Portable
components can be validated as build artifacts, but that validation does not
grant runtime authority or decide release eligibility. ADR 0010 already keeps
Mantle's public boundary build-shaped rather than making it an application or
module runtime.

## Decision

Mantle will model component production as an explicit build pipeline.
Human-authored configuration is a typed Nickel component-build manifest. Pure,
no-std Rust logic validates normalized manifests and locks, separates digest
roles, identifies the complete tool cohort, plans exact source and composition
inputs, starts WASI virtualization from deny-all, and constructs bounded report
and handoff DTOs. Imperative adapters alone may read credentials, access a
registry, hash bytes, invoke tools, enter sandboxes, or persist store objects.

The package-resolution shell uses an explicit `wkg` configuration and checked
`wkg.lock`, then hands immutable local package objects to network-denied build
stages. WAC composition receives every dependency through an exact local
binding; missing dependencies do not fall back to Warg or another live
resolver. Each identity-changing portable artifact re-enters pinned
`wasm-tools` validation. Before final portable admission, pinned `wasm-tools
strip` removes non-semantic producer/custom metadata that otherwise duplicates
across nested Rust component modules; the normalized bytes are validated again
and become the exact Octet, runtime-smoke, bundle, and release input. Consumer
and release profiles bind the independent Octet artifact-rail result for those
same bytes without translating Octet findings into Mantle policy.

Rust `wasm32-wasip2`, wit-bindgen, wasm-component-ld, wasm-tools, WAC,
WASI-Virt, Wizer, and Wasmtime form one compatibility cohort. Any member,
target, feature, or configuration change changes the BLAKE3 cohort identity and
invalidates cohort-bound outputs until the relevant fixtures rerun.

Optional Wizer output remains portable but must bind deterministic virtual
inputs and repeated-output evidence when deterministic eligibility is claimed.
Wizer accepts core modules rather than Component Model binaries. Although the
pinned linker can expose its pre-component module with `--skip-wit-component`,
this pipeline does not yet declare or attest that linker split, core object, or
componentization configuration. It therefore denies Wizer execution and records
no transformed artifact instead of introducing an unbound intermediate or
mislabeling component bytes. Optional Wasmtime precompile output is
target-specific trusted native code, not
portable validated Wasm. Consumers receive one versioned materialization bundle
whose exact objects and BLAKE3 identities can be remeasured. Later Valence or
Cairn admission evidence may refer to that bundle but cannot enter its canonical
identity and create a circular claim.

## Consequences

Mantle gains a single production materialization seam while retaining a thin,
build-only public boundary. Configuration, planning, and admission rules can be
tested without registry, process, sandbox, or runtime setup. Tool execution and
compatibility still require an explicitly pinned cohort and end-to-end fixtures;
pure planning success alone cannot claim that a component builds or runs.

Protocol SHA-256 values remain visible and verifiable, but cannot occupy a
Mantle BLAKE3 identity role. Store paths and package names remain locator
metadata rather than artifact identity. Credential handles may cross the pure
planning boundary, but credential values remain shell-owned and must not appear
in generated files or reports.

## Alternatives Considered

### Use ambient cargo-component and registry configuration

Rejected. Ambient caches, credentials, resolver state, and tool versions make
package selection and component bytes non-reproducible.

### Make WAC, WASI-Virt, or Octet policy part of Mantle configuration semantics

Rejected. Mantle should generate and bind tool-native inputs and outputs, not
fork external languages or reinterpret an independent artifact-policy result.

### Let each runtime or application repository rebuild production components

Rejected. Rebuilding nominally identical components with unrelated tools loses
the exact derivation from package inputs to portable and native outputs.

### Treat precompiled Wasmtime output as validated portable Wasm

Rejected. A `.cwasm` artifact contains target-specific native code and requires
stricter target, CPU, configuration, and byte-identity verification.
