# Design: Ship delta substitution

## Context

The repo already has two important pieces of groundwork:

- a protocol-level delta transfer spec
- binary-cache requirements that allow delta-aware substitution

What it does not yet have is runtime integration. The cache path still needs a
real negotiation step, a receiver-side compatibility manifest built from local
state, a streamed transfer path, and reporting that tells operators whether a
cache hit used delta reuse or full-artifact fallback.

## Goals / Non-Goals

**Goals:**

- make delta transfer a real HTTP substitution path for trusted caches
- reuse local `PathInfo` and castore presence to build bounded manifests
- preserve ordinary substitution trust and final acceptance semantics
- expose substitution-mode and reuse accounting to operators

**Non-Goals:**

- new trust semantics for accepted cache hits
- a generic P2P transport
- changing the delta protocol version in this change
- forcing every cache to support delta mode

## Decisions

### 1. Delta stays inside the ordinary HTTP substitution path

**Choice:** delta transfer is negotiated from the same trusted HTTP cache
authority already used for ordinary substitution.

**Rationale:** this keeps operator configuration simple and preserves the
existing trust boundary. A cache that supports delta mode is still just a
substituter; it is not a second independent trust source.

**Implementation:** crunch performs ordinary cache discovery first, then asks
whether the same authority supports delta negotiation for the requested output
or closure.

### 2. Receiver compatibility manifests are derived from local facts only

**Choice:** the receiver builds its manifest from local `PathInfo` and castore
presence without downloading content merely to advertise reuse.

**Rationale:** a manifest that requires payload fetches defeats the purpose of
cheap reuse planning.

**Implementation:** only content with both metadata and backing local castore
presence is advertised as reusable. Missing backing content is treated as
absent.

### 3. Final acceptance reuses ordinary substitution semantics

**Choice:** a delta transfer is not accepted on its own. It becomes a cache hit
only after crunch reconstructs the final output, verifies the final signed
`PathInfo`, and records the same local metadata or attestation consequences as a
full substitution.

**Rationale:** delta mode is a transport optimization, not a new trust model.

**Implementation:** the delta path feeds the same final acceptance code path as
ordinary substitution.

### 4. Reporting includes mode, byte counts, and fallback reasons

**Choice:** substitution reporting names whether the cache hit used `delta` or
`full` mode, how many bytes were transferred, and why crunch fell back if delta
negotiation started but did not finish in delta mode.

**Rationale:** if operators cannot see whether delta mode helped, the feature is
hard to tune or trust.

**Implementation:** human output summarizes substitution mode and reuse savings;
JSON build output carries stable fields for substitution mode, bytes
transferred, bytes reused, and optional fallback reason.

## Risks / Trade-offs

**[Manifest cost]**
Receiver-side manifest construction can become expensive for large closures.

**Mitigation:** keep manifests scoped to the requested output or closure and use
existing local metadata instead of deep payload inspection.

**[Transport fallback complexity]**
Mid-transfer fallback can complicate the control flow.

**Mitigation:** centralize final acceptance and keep delta mode as an optional
front-end to the existing substitution path.

**[Opaque behavior]**
A hidden delta path can make cache behavior hard to reason about.

**Mitigation:** report mode, byte counts, and fallback reasons explicitly.
