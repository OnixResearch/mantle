## Why

Mantle can build Rust projects, export Nickel configuration, preserve source closures, and emit build/attestation evidence, but it has no first-class build pipeline for WebAssembly components. Without one, consumers must resolve WIT packages, pin component toolchains, compose dependencies, virtualize WASI, validate outputs, precompile native Wasmtime artifacts, and record digest roles through ad hoc derivations.

Mantle should own this as a build-shaped pipeline while leaving component runtime semantics, authority, Evidence IR interpretation, and lifecycle acceptance to Aspen, Basalt/UCAN, Valence, and Cairn.

## What Changes

- Add a typed Nickel component-build manifest and deterministic generated inputs for the tool-native package and composition formats.
- Integrate `wasm-pkg-tools`/`wkg` for exact WIT and reusable component-library resolution into store inputs and a checked `wkg.lock`.
- Preserve OCI-required SHA-256 as external interoperability metadata while computing Mantle-owned BLAKE3 identities over fetched bytes and all generated artifacts.
- Pin one Rust/wasm32-wasip2, wit-bindgen, wasm-component-ld, wasm-tools, WAC, WASI-Virt, Wizer, and Wasmtime compatibility cohort.
- Use WAC for exact local component composition and explicit dependency graphs; do not depend on archived Warg resolution.
- Apply WASI-Virt from an explicit deny-all plan before reviewed allow/virtualize rules, then validate the final component against the declared world and import policy.
- Run build-local `wasm-tools` validation and the Octet artifact rail over the exact final portable bytes, binding both reports without treating either as runtime authority.
- Optionally emit Wizer-preinitialized portable artifacts and Wasmtime precompiled native artifacts with exact configuration/target evidence.
- Emit one canonical component materialization bundle that binds exact store objects, profile/cohort identities, stage receipts, optional native outputs, and consumer-facing verification inputs.
- Extend build reports and attestations with package, composition, virtualization, validation, transform, precompile, and materialization evidence.

## Impact

- **Surfaces**: Nickel stdlib/contracts, project manifests/locks, source fetchers, component build helpers, WAC/WASI-Virt plans, build reports, attestations, release evidence, fixtures, and documentation.
- **Architecture boundary**: Mantle is the stack-owned production materializer for component bytes, bindings, compositions, transforms, and precompiled outputs. It does not host components, authorize imports, define Preserves semantics, interpret Octet/Valence results, or decide release eligibility.
- **Security**: registry credentials remain shell-owned handles; build sandboxes receive only resolved immutable inputs. Precompiled Wasmtime artifacts are native trusted outputs and never treated as portable validated Wasm.
- **Compatibility**: native tool formats such as WIT, WAC, OCI metadata, and `wkg.lock` remain tool-native; human-authored policy and orchestration stay Nickel-authored.
