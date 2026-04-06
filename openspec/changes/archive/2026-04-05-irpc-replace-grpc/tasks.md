## Phase 1: Add irpc, remove protoc dependency

- [x] Add `postcard` to workspace Cargo.toml dependencies ✅ 14m (started: 2026-04-05T12:25Z -> completed: 2026-04-05T12:39Z)
- [x] Remove `tonic`, `tonic-build`, `tonic-health`, `tonic-reflection`, `prost`, `prost-build`, `prost-types` from all vendor crate Cargo.toml files ✅
- [x] Delete `build.rs` proto compilation steps in snix-castore, snix-store, snix-build ✅
- [x] Delete all `.proto` files under `vendor/proto/` and per-crate `protos/` dirs ✅
- [x] Delete generated code files (`src/generated/*.rs`) — replaced with serde-derived types ✅
- [x] Delete gRPC client/server wrappers and `grpc.rs` files ✅
- [x] Replace `prost::Message` encode/decode with postcard serialization ✅
- [x] Replace `tonic::async_trait` with `async_trait` crate (needed for dyn-compatible traits) ✅
- [x] Verify `cargo check` succeeds without protoc on PATH ✅

## Phase 2: Add serde derives to wire types

- [x] Add `Serialize, Deserialize` derives to castore types (Directory, Entry, etc.) ✅ (done in Phase 1)
- [x] Add `Serialize, Deserialize` derives to store types (PathInfo, NarInfo, etc.) ✅ (done in Phase 1)
- [x] Add `Serialize, Deserialize` derives to build types (BuildRequest, BuildResponse, etc.) ✅ (done in Phase 1)
- [x] Verify postcard round-trip for each type with a unit test ✅

## Phase 3: Define irpc services

- [x] Add `irpc` to workspace dependencies ✅ 7m (started: 2026-04-05T12:51Z -> completed: 2026-04-05T12:58Z)
- [x] Define `BlobServiceProtocol` with Has, Read, Put, Chunks in snix-castore ✅
- [x] Define `DirectoryServiceProtocol` with Get, Put in snix-castore ✅
- [x] Define `PathInfoServiceProtocol` with Get, Put in snix-store ✅
- [x] Define `BuildServiceProtocol` with DoBuild in snix-build ✅
- [x] Write irpc client wrappers for all four services (Blob, Directory, PathInfo, Build) ✅

## Phase 4: Replace gRPC impls

- [x] Delete `grpc.rs` in snix-build/src/buildservice/ ✅ (done in Phase 1)
- [x] Delete `grpc.rs` in snix-castore/src/blobservice/ ✅ (done in Phase 1)
- [x] Delete gRPC directory service impl in snix-castore ✅ (done in Phase 1)
- [x] Delete gRPC pathinfo service impl in snix-store ✅ (done in Phase 1)
- [x] Delete `proto/grpc_buildservice_wrapper.rs` and gRPC re-exports in snix-build ✅ (done in Phase 1)
- [x] Remove gRPC-related entries from `mod.rs` and `from_addr.rs` in each service module ✅ (done in Phase 1)
- [x] Remove tonic re-exports from snix-tracing propagate module ✅ (done in Phase 1)

## Phase 5: Verify and clean up

- [x] `cargo build` succeeds without protoc on PATH ✅
- [x] `cargo test` passes for all workspace crates (excluding crunch-glue pre-existing failures) ✅
- [x] Update AGENTS.md to remove protoc from build requirements ✅
- [x] Update any `from_addr` parsing that referenced `grpc+` URL schemes ✅ (done in Phase 1)
- [x] Remove `vendor/proto/` directory ✅ (done in Phase 1)
