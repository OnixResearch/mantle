## Phase 1: Contract

- [x] [serial] Add the standalone `no_std + alloc` request, observation, identity, and admission core. r[mantle.build_interchange.contract] r[mantle.build_interchange.request] r[mantle.build_interchange.observation]
- [x] [serial] Add exact product and coherent outcome admission. r[mantle.build_interchange.products] r[mantle.build_interchange.observation]
- [x] [serial] Add positive and negative unit tests for schema, identity, linkage, product, cache, builder, and outcome drift. r[mantle.build_interchange.conformance]

## Phase 2: Producer artifacts

- [x] [serial] Add the typed Nickel contract and positive and negative fixtures. r[mantle.build_interchange.contract] r[mantle.build_interchange.conformance]
- [x] [serial] Add deterministic success, failure, and cache-observation producer fixtures with freshness tests. r[mantle.build_interchange.conformance]
- [x] [serial] Document authority, shell, identity, product, cache, and non-claim boundaries. r[mantle.build_interchange.boundary]

## Phase 3: Verification and lifecycle

- [x] [serial] Run focused Rust, Clippy, WebAssembly, Nickel, fixture, Cairn, traceability, and Nix checks. r[mantle.build_interchange.conformance]
- [x] [serial] Record evidence, sync accepted specifications, archive the change, and rerun final validation. r[mantle.build_interchange.conformance] r[mantle.build_interchange.boundary]
