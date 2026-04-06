## Context

snix inherited gRPC from tvix for daemon-to-daemon communication. crunch
vendors snix-build, snix-castore, and snix-store, all of which carry tonic
and prost dependencies plus `.proto` files that require `protoc` at compile
time. crunch currently uses only the in-memory implementations of these
services. The gRPC code paths are dead weight.

iroh's `irpc` crate provides typed RPC with zero-cost in-process channels
and optional QUIC transport for cross-process/cross-machine calls. It uses
postcard for serialization (serde-based, no external codegen tool).

## Goals / Non-Goals

**Goals:**
- Remove protoc build requirement.
- Remove tonic/prost/h2/tower dependency subtree.
- Define irpc service enums for Blob, Directory, PathInfo, Build.
- Preserve all existing service trait interfaces unchanged.
- In-process calls go through tokio channels with zero serialization.

**Non-Goals:**
- Wire compatibility with upstream snix/tvix gRPC endpoints.
- Distributed build scheduling (future work that benefits from irpc, but
  not part of this change).
- Changing the data model — `PathInfo`, `Directory`, `Node`, `BlobChunk`
  structs keep their fields and semantics.

## Decisions

### 1. irpc over quic-rpc

**Choice:** Use `irpc` (the newer crate), not `quic-rpc` (the older one).
**Rationale:** irpc is iroh's second-generation RPC crate, designed after
extensive experience with quic-rpc. The iroh team uses irpc for all current
protocols. It's simpler (one macro vs. multiple traits), lighter on deps
when used in-process, and has cleaner streaming support.
**Alternative:** quic-rpc — rejected because iroh's own docs recommend irpc
as the replacement.

### 2. One service crate or per-crate definitions

**Choice:** Define irpc services inside each vendor crate, replacing the
existing `grpc.rs` files in-place.
**Rationale:** Keeps the change scoped — each crate already owns its service
trait and its gRPC impl. Swapping gRPC for irpc in the same location
minimizes cross-crate churn. A shared `crunch-rpc` crate is unnecessary
since there's no shared transport config.
**Alternative:** Centralize all service definitions in a new `crunch-rpc`
crate — rejected because it creates a new dependency edge from every vendor
crate back into crunch's own crate graph.

### 3. Serde derives on existing types vs. new wire types

**Choice:** Add `#[derive(Serialize, Deserialize)]` to the existing Rust
types (`PathInfo`, `Directory`, `Node`, `NarInfo`, etc.) rather than
defining separate wire types.
**Rationale:** The protobuf-generated types were already thin wrappers that
mapped 1:1 to the "real" types via `From`/`TryFrom` impls. Cutting out
the intermediate layer removes conversion boilerplate. postcard handles
enums, Vec, Option, bytes natively.
**Alternative:** Define dedicated wire structs — rejected because it
replicates the proto→native conversion overhead irpc is meant to eliminate.

### 4. Streaming patterns

**Choice:** Map proto streaming RPCs to irpc channel types:

| Proto pattern | irpc channel | Example |
|---|---|---|
| Unary | `oneshot::Sender<R>` | `BuildService::DoBuild` |
| Server stream | `mpsc::Sender<R>` | `BlobService::Read`, `DirectoryService::Get`, `PathInfoService::List` |
| Client stream | `mpsc::Receiver<R>` | `BlobService::Put`, `DirectoryService::Put` |
| Client stream + unary response | `rx = mpsc::Receiver<R>, tx = oneshot::Sender<Resp>` | `BlobService::Put` (stream chunks, get digest back) |

**Rationale:** Direct mapping from the existing proto service definitions.
The irpc macro enforces correct channel types at compile time.

### 5. Error handling

**Choice:** Use `serde_error` for serializing errors across the wire
boundary during initial development. Migrate to concrete error types with
snafu once the protocol stabilizes.
**Rationale:** Per irpc docs, starting with anyhow+serde_error avoids
premature error type design. The in-process path doesn't serialize errors
at all (they pass through the tokio channel as-is), so this only matters
for the cross-process path we're not using yet.

### 6. Data format: postcard

**Choice:** postcard (irpc's built-in serializer). No configurability.
**Rationale:** irpc hardcodes postcard — there's no choice here, and
that's fine. postcard is compact, fast, no-std compatible, and handles
`#[serde(with = "serde_bytes")]` for efficient byte slice encoding. The
blob/NAR data paths already chunk into ~64 KiB pieces, so varint length
prefixes add negligible overhead.

## Risks / Trade-offs

**[No gRPC interop]** → crunch cannot talk to existing snix/tvix store
daemons. Acceptable: crunch is its own stack and doesn't need to interoperate
with Nix's daemon protocol (that's what nix-daemon and snix-daemon are for).

**[irpc maturity]** → irpc is newer than tonic. Mitigated by: iroh uses it
in production for blobs, docs, and all iroh protocols; the crate is small
enough to vendor if upstream goes dormant.

**[Serde on internal types]** → Adding Serialize/Deserialize to types like
`PathInfo` and `Directory` means their in-memory representation becomes part
of the wire format. If fields change, cross-process clients break.
Mitigated by: we control both sides; postcard's format is append-friendly
for optional fields.
