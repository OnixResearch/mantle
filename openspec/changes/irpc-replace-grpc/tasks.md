## Phase 1: Add irpc, remove protoc dependency

- [ ] Add `irpc`, `postcard`, `serde_error` to workspace Cargo.toml dependencies
- [ ] Remove `tonic`, `tonic-build`, `tonic-health`, `tonic-reflection`, `prost`, `prost-build`, `prost-types` from all vendor crate Cargo.toml files
- [ ] Delete `build.rs` proto compilation steps in snix-castore, snix-store, snix-build
- [ ] Delete all `.proto` files under `vendor/proto/` and per-crate `protos/` dirs
- [ ] Delete generated code files (`src/generated/*.rs`) in snix-castore, snix-store, snix-build
- [ ] Verify `cargo check` fails only on missing irpc service defs, not on protoc/tonic

## Phase 2: Add serde derives to wire types

- [ ] Add `Serialize, Deserialize` derives to `PathInfo`, `NarInfo`, and related structs in snix-store
- [ ] Add `Serialize, Deserialize` derives to `Directory`, `Node`, `Entry`, blob types in snix-castore
- [ ] Add `Serialize, Deserialize` derives to `BuildRequest`, `BuildResult` in snix-build
- [ ] Add `#[serde(with = "serde_bytes")]` to digest and chunk byte fields
- [ ] Verify postcard round-trip for each type with a unit test

## Phase 3: Define irpc services

- [ ] Define `BlobServiceRequest` enum with `Stat`, `Read` (server stream), `Put` (client stream) in snix-castore
- [ ] Define `DirectoryServiceRequest` enum with `Get` (server stream), `Put` (client stream) in snix-castore
- [ ] Define `PathInfoServiceRequest` enum with `Get`, `Put`, `CalculateNAR`, `List` (server stream) in snix-store
- [ ] Define `BuildServiceRequest` enum with `DoBuild` (unary) in snix-build
- [ ] Write irpc client wrappers implementing the existing `BlobService`, `DirectoryService`, `PathInfoService`, `BuildService` traits

## Phase 4: Replace gRPC impls

- [ ] Delete `grpc.rs` in snix-build/src/buildservice/
- [ ] Delete `grpc.rs` in snix-castore/src/blobservice/
- [ ] Delete gRPC directory service impl in snix-castore (if separate file)
- [ ] Delete gRPC pathinfo service impl in snix-store
- [ ] Delete `proto/grpc_buildservice_wrapper.rs` and `proto/mod.rs` gRPC re-exports in snix-build
- [ ] Remove gRPC-related entries from `mod.rs` and `from_addr.rs` in each service module
- [ ] Remove tonic re-exports from snix-tracing propagate module

## Phase 5: Verify and clean up

- [ ] `cargo build` succeeds without protoc on PATH
- [ ] `cargo test` passes for all workspace crates
- [ ] Update AGENTS.md to remove protoc from build requirements
- [ ] Update any `from_addr` parsing that referenced `grpc+` URL schemes
- [ ] Remove `vendor/proto/` directory if empty
