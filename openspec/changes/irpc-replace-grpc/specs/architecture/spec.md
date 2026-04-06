## MODIFIED Requirements

### Requirement: Crate layout

The workspace MUST contain the following crates:

| Crate | Role |
|---|---|
| `crunch` (binary) | CLI entry point, wires everything together |
| `crunch-eval` | Nickel evaluation wrapper — takes a file path, returns structured JSON |
| `crunch-glue` | Converts evaluated Nickel records to Derivation structs, manages KnownPaths |
| `nix-compat` (vendored) | Derivation struct, store path calculation, ATerm, NAR, nixbase32 |
| `snix-build` (vendored) | BuildService trait, BuildRequest, sandbox execution |
| `snix-castore` (vendored) | Content-addressed blob and directory storage |
| `snix-store` (vendored) | PathInfoService, NAR calculation, store construction |

The vendored snix crates MUST use irpc for cross-boundary RPC instead of
gRPC/tonic. The `*Service` traits MUST remain unchanged.

The workspace MUST NOT depend on tonic, tonic-build, prost, or prost-build.
The workspace MUST NOT require protoc at compile time.

## ADDED Requirements

### Requirement: irpc service definitions

Each vendored service crate MUST define an irpc service enum using the
`rpc_requests!` macro:

- `snix-castore`: `BlobServiceRequest`, `DirectoryServiceRequest`
- `snix-store`: `PathInfoServiceRequest`
- `snix-build`: `BuildServiceRequest`

Each request variant MUST specify its response channel type matching the
RPC pattern:

- Unary RPCs use `oneshot::Sender<Result<Response>>`
- Server-streaming RPCs use `mpsc::Sender<Result<Item>>`
- Client-streaming RPCs use `mpsc::Receiver<Item>` on the request

#### Scenario: BlobService irpc definition compiles

- GIVEN the `snix-castore` crate with irpc service definitions
- WHEN `cargo check -p snix-castore` is run
- THEN it compiles without protoc in PATH

#### Scenario: BuildService unary RPC

- GIVEN a `BuildServiceRequest::DoBuild` variant with `oneshot::Sender<Result<BuildResult>>`
- WHEN a client sends a DoBuild request via the irpc client
- THEN it receives a single BuildResult response

### Requirement: In-process zero-cost path

When services are colocated in the same process, irpc MUST use tokio mpsc
channels for dispatch. Request and response types MUST NOT be serialized
in the in-process path.

#### Scenario: In-process blob read

- GIVEN a BlobService irpc client connected via local channel
- WHEN `open_read` is called with a known digest
- THEN the blob data is returned without any postcard serialization

### Requirement: Serde on wire types

All request, response, and streaming item types MUST derive `Serialize`
and `Deserialize` (via serde) to support the cross-process postcard path.

Byte-heavy fields (digests, blob chunks, NAR data) SHOULD use
`#[serde(with = "serde_bytes")]` for efficient encoding.

#### Scenario: PathInfo round-trips through postcard

- GIVEN a `PathInfo` struct with references and NAR info
- WHEN serialized with postcard and deserialized back
- THEN the result is identical to the original

### Requirement: No protoc build dependency

The workspace MUST compile without `protoc` in PATH. All `build.rs` files
that invoke `tonic-build` or `prost-build` MUST be removed or replaced.

#### Scenario: Clean build without protoc

- GIVEN a fresh checkout with no cached build artifacts
- WHEN `cargo build` is run without protoc on PATH
- THEN the build succeeds

### Requirement: Proto file removal

All `.proto` files under `vendor/proto/` and per-crate `protos/` directories
MUST be removed. The `vendor/proto/` directory MAY be deleted entirely if
no other files remain.

All generated Rust files under `src/generated/` in vendor crates MUST be
removed.
