## Phase 1: Source identity

- [x] [serial] Bind the accepted artifact-auth publication receipt and exact Radicle HTTPS source. r[mantle.artifact_auth_adoption.radicle_transport]
- [x] [serial] Cut both Cargo manifests and the Nix manifest to Radicle HTTPS, then regenerate `Cargo.lock` and `flake.lock` with their owning tools without revision or content drift. r[mantle.artifact_auth_adoption.lock_agreement]

## Phase 2: Deterministic acceptance

- [x] [parallel] Run focused action-result and shell artifact-auth tests before and after cutover without Rust implementation changes. r[mantle.artifact_auth_adoption.behavior]
- [x] [serial] Prove GitHub fallback, mismatched RID/revision, duplicate or missing packages, and changed lock identity are rejected. r[mantle.artifact_auth_adoption.fallback]
- [x] [serial] Emit typed BLAKE3 cutover evidence, run focused Nix and Cairn checks, sync the accepted spec, and archive at the bounded claim boundary. r[mantle.artifact_auth_adoption.radicle_evidence]
