## Context

Source acquisition and build execution have different trust boundaries. Fetchers intentionally contact the network for declared fixed-output inputs. Ordinary derivation builders should consume only declared inputs and must not repair missing material by reaching out to the network.

## Decisions

### 1. Network policy is part of action identity

**Choice:** Action specs and build receipts bind the requested and enforced network policy.

**Rationale:** A result produced with network access is not equivalent to a result produced offline from declared inputs.

### 2. Fetchers remain fixed-output boundaries

**Choice:** Builtin fetchers may use network only when the URL, hash algorithm, expected digest, mode, and retry policy are declared.

**Rationale:** This preserves source acquisition while keeping input identity explicit and verifiable.

### 3. Compatibility allowances are scoped capabilities

**Choice:** Any build-time network exception is modeled as a per-action sandbox capability with policy basis and audit reporting.

**Rationale:** Global impurity would hide which derivation consumed the exception.

## Risks / Trade-offs

- Some foreign derivations that expect network during build will be unsupported until adapted or explicitly classified.
- Platform-specific sandbox network controls need clear unsupported non-claims.
