# Tasks

## 1. Freeze the source and baseline

- [ ] [serial] 1.1 Run existing foreign-import, Nix-producer, direct `.drv`, prefix-aware ATerm, structured-attribute, and hash-domain tests before core changes. r[foreign_derivation_import.nix_derivation_adapter_parity]
- [ ] [serial] 1.2 Record upstream commit `2cfc0f90ed83ea3cc983e5c305f89494a6df073e`, exact crate version `0.1.0`, crates.io checksum, license, and reviewed file set. r[foreign_derivation_import.reviewed_nix_derivation_dependency]
- [ ] [serial] 1.3 Add a deterministic package-source parity guard with positive and negative self-tests. r[foreign_derivation_import.reviewed_nix_derivation_dependency]
- [ ] [serial] 1.4 Record the pre-adoption Mantle revision, dependency state, adapter state, and rollback command. r[foreign_derivation_import.nix_derivation_adapter_rollback]

## 2. Add the pure compatibility adapter

- [ ] [serial] 2.1 Add one private Mantle adapter as the only production import site for `nix-derivation`. r[foreign_derivation_import.nix_derivation_projection_boundary]
- [ ] [serial] 2.2 Keep file reads, backend execution, directory discovery, and artifact writes outside the adapter. r[foreign_derivation_import.nix_derivation_projection_boundary]
- [ ] [serial] 2.3 Derive and validate the out-of-band derivation name from the logical `.drv` identity. r[foreign_derivation_import.nix_derivation_projection_boundary]
- [ ] [serial] 2.4 Map every output and dynamic-input form to accepted graph facts, explicit unsupported records, or stable rejections. r[foreign_derivation_import.nix_derivation_projection_boundary]
- [ ] [serial] 2.5 Preserve byte-valued environments and reject non-UTF-8 graph projections without lossy conversion. r[foreign_derivation_import.nix_derivation_projection_boundary]
- [ ] [serial] 2.6 Add a source guard that rejects direct production imports outside the adapter. r[foreign_derivation_import.nix_derivation_projection_boundary]

## 3. Integrate Nix compatibility paths

- [ ] [serial] 3.1 Route explicit Nix `.drv` file and directory admission through the adapter. r[foreign_derivation_import.reviewed_nix_derivation_adapter]
- [ ] [serial] 3.2 Route backend-produced Nix `.drv` closure admission through the same adapter. r[nix_producer_adapter.reviewed_derivation_admission]
- [ ] [serial] 3.3 Preserve existing byte, collection, field, closure, and publication bounds before projection. r[foreign_derivation_import.reviewed_nix_derivation_adapter]
- [ ] [serial] 3.4 Preserve existing foreign graph, package-index, receipt, and CLI schemas. r[foreign_derivation_import.reviewed_nix_derivation_adapter]
- [ ] [serial] 3.5 Keep Guix and other prefix-rewrite production paths on the existing parser until separate parity passes. r[foreign_derivation_import.nix_derivation_adapter_parity]

## 4. Prove hash and behavior parity

- [ ] [parallel] 4.1 Compare canonical bytes, parsed facts, Nix derivation hashes, output paths, and store paths across pinned Nix 2.34 fixtures. r[foreign_derivation_import.nix_derivation_adapter_parity]
- [ ] [parallel] 4.2 Compare structured attributes, candidate order, duplicate semantics, every output variant, and recursive dynamic inputs. r[foreign_derivation_import.nix_derivation_adapter_parity]
- [ ] [parallel] 4.3 Exercise malformed, truncated, oversized, over-depth, invalid-name, wrong-prefix, wrong-domain, invalid structured-data, and incomplete-closure fixtures. r[foreign_derivation_import.nix_derivation_adapter_parity]
- [ ] [serial] 4.4 Classify every mismatch before cutover and retain the old path when a required case differs. r[foreign_derivation_import.nix_derivation_adapter_parity]
- [ ] [serial] 4.5 Prove that Nix identities use required Nix algorithms while Mantle receipt identities remain BLAKE3. r[nix_producer_adapter.reviewed_derivation_admission]

## 5. Document and cut over

- [ ] [serial] 5.1 Add an ADR for the narrow dependency boundary, maturity posture, and rejected broad replacement. r[foreign_derivation_import.reviewed_nix_derivation_adapter]
- [ ] [serial] 5.2 Update foreign-import and producer guides with supported versions, limits, rollback, and non-claims. r[nix_producer_adapter.reviewed_derivation_admission]
- [ ] [serial] 5.3 Cut over only the accepted `/nix/store` compatibility paths after all positive and negative parity evidence passes. r[foreign_derivation_import.nix_derivation_adapter_parity]
- [ ] [serial] 5.4 Record an adoption receipt that binds source, package, fixture, adapter, and validation identities. r[foreign_derivation_import.nix_derivation_adapter_rollback]

## 6. Validate

- [ ] [serial] 6.1 Run `cargo fmt --all -- --check`. r[foreign_derivation_import.nix_derivation_projection_boundary]
- [ ] [serial] 6.2 Run focused foreign-import and Nix-producer positive and negative tests. r[foreign_derivation_import.nix_derivation_adapter_parity]
- [ ] [serial] 6.3 Run the package-source parity guard and its self-test. r[foreign_derivation_import.reviewed_nix_derivation_dependency]
- [ ] [serial] 6.4 Run `cargo test --workspace`. r[foreign_derivation_import.reviewed_nix_derivation_adapter]
- [ ] [serial] 6.5 Run `cargo clippy --workspace --all-targets -- -D warnings`. r[foreign_derivation_import.nix_derivation_projection_boundary]
- [ ] [serial] 6.6 Run Cairn validation, requirement coverage, proposal gate, design gate, and tasks gate. r[foreign_derivation_import.nix_derivation_adapter_rollback]
