## Why

The vendored snix crates use gRPC (tonic + prost + protobuf) for inter-process
communication between the store, castore, and build services. This drags in a
large dependency tree (tonic, prost, prost-build, hyper, h2, tower), requires
`protoc` at compile time, and couples the wire format to protobuf. crunch
doesn't need any of this — it doesn't talk to Nix daemons, doesn't serve
gRPC endpoints, and runs all services in-process today.

iroh's `irpc` crate solves the same problem (typed RPC with request/response,
client streaming, server streaming, bidi streaming) but:

- Uses postcard (serde + varint length prefixes) instead of protobuf — no
  `protoc`, no `.proto` files, no codegen step.
- Has a zero-cost in-process path via tokio mpsc channels, so when services
  are colocated (the common case) there's no serialization overhead.
- Cross-process uses QUIC (via `noq`) instead of HTTP/2 — lower latency,
  built-in multiplexing, no TLS ceremony for local sockets.
- The `rpc_requests!` macro generates the dispatch enum and client wrappers
  from a single enum definition.
- Trivially extends to distributed builds later — same protocol works
  across machines via iroh's P2P connectivity.

Replacing tonic/prost with irpc removes the protoc build dependency, cuts
~30 crates from the dependency tree, and positions crunch for distributed
builds without a second RPC system.

## What Changes

- **BlobService RPC**: Replace `GRPCBlobService` (tonic client wrapping
  `blob_service_client::BlobServiceClient`) with an irpc service definition.
  The trait methods (`has`, `open_read`, `open_write`, `chunks`) map to irpc
  request types with appropriate response channels.

- **DirectoryService RPC**: Replace `GRPCDirectoryService` with irpc. `Get`
  (server streaming) and `Put` (client streaming) become irpc streaming
  requests.

- **PathInfoService RPC**: Replace gRPC `PathInfoService` with irpc. `Get`,
  `Put`, `CalculateNAR`, `List` (server streaming) map to irpc requests.

- **BuildService RPC**: Replace `GRPCBuildService` with irpc. `DoBuild` is
  a simple request/response — one irpc rpc call.

- **Proto files removed**: Delete all `.proto` files under `vendor/proto/`
  and per-crate `protos/` directories. Data types stay as Rust structs with
  serde derives (they already exist as `nix_compat` / snix types).

- **Build system**: Remove `prost-build` and `tonic-build` from build
  dependencies. Remove `build.rs` proto compilation steps. Drop the `protoc`
  requirement from AGENTS.md.

- **Wire types**: The existing protobuf message types (`StatBlobRequest`,
  `BlobChunk`, `PathInfo`, etc.) become plain Rust structs with
  `#[derive(Serialize, Deserialize)]`. The protobuf-generated code in
  `src/generated/` is deleted.

## Capabilities

### New Capabilities

- `irpc-services`: All four service interfaces (Blob, Directory, PathInfo,
  Build) exposed as irpc service definitions that work identically in-process
  and cross-process.
- `no-protoc-build`: The project compiles without protoc in PATH.
- `p2p-ready-rpc`: The RPC layer can reach remote machines via iroh/noq
  QUIC connections with zero additional protocol work.

### Modified Capabilities

- `blob-rpc`: Same logical operations, new transport.
- `directory-rpc`: Same logical operations, new transport.
- `pathinfo-rpc`: Same logical operations, new transport.
- `build-rpc`: Same logical operations, new transport.

### Removed Capabilities

- `grpc-compat`: No gRPC server or client. Cannot talk to upstream
  snix/tvix gRPC endpoints. This is intentional — crunch is its own stack.

## Impact

- **Files**: All `grpc.rs` files under `vendor/snix-{build,castore,store}/`,
  all `.proto` files, all `build.rs` proto steps, all `generated/*.rs` files.
  New irpc service definitions in each vendor crate (or a shared
  `crunch-rpc` crate).
- **APIs**: The `*Service` traits are unchanged. Only the gRPC *implementations*
  of those traits are replaced.
- **Dependencies**: Remove tonic, tonic-build, tonic-health, tonic-reflection,
  prost, prost-build, prost-types, prost-wkt-types. Add irpc, postcard, noq.
- **Testing**: Existing integration tests calling the trait methods continue
  to work (they use in-memory impls, not gRPC). New tests for irpc
  round-tripping each service.
