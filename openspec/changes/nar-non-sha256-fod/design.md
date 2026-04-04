## Context

NAR hashing works by serializing the output path contents in NAR format and
hashing the byte stream. `SimpleRenderer::calculate_nar()` always returns
sha256. For non-sha256 NAR FODs we need to serialize NAR and pipe it through
a different hasher.

`snix-store` exposes `write_nar()` which writes NAR bytes to an arbitrary
`AsyncWrite`. We can wrap a hasher in an `AsyncWrite` adapter.

## Goals / Non-Goals

**Goals:** Verify NAR-mode FODs for all four hash algorithms.

**Non-Goals:** Don't rewrite `calculate_nar`. Don't add NAR-sha512 to the
`NarCalculationService` trait — that's an snix concern.

## Decisions

### 1. Hash-streaming NAR writer

**Choice:** Call `write_nar()` directly, writing into a `HashWriter` that
wraps `tokio::io::sink()` and updates a `digest::Digest` on each write.

**Rationale:** Avoids buffering the full NAR in memory. The `write_nar`
function already exists and handles all node types.

**Alternative:** Buffer NAR bytes in a `Vec<u8>` and hash at the end.
Rejected — wastes memory for large outputs.

### 2. Scope: only verify_fod_hash changes

**Choice:** Keep the change inside `orchestrate.rs`. Add a `nar_hash()`
helper alongside `hash_blob()`.

**Rationale:** Minimal diff. The NAR writer and blob service are already
available at the call site.

## Risks / Trade-offs

**[write_nar visibility]** → `write_nar` must be pub in snix-store. If it's
not, we may need to make it pub in the vendored copy. Low risk since it's
already `pub use`d in `snix_store::nar`.
