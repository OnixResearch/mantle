# Design: Signed shared Rust unit action results

## Context

The local Rust unit cache defines canonical action identities, immutable result records, castore output trees, admission, and atomic materialization.

ADR 0024 defines three separate surfaces:

1. immutable content storage;
2. action-result lookup and publication;
3. execution after an admitted miss.

Remote Rust unit reuse must preserve that separation. It must not force Rust unit trees into derivation PathInfo records or grant transport operators output authority.

## Decisions

### Decision 1: Extend action-result transport without changing derivation result semantics

**Choice:** Add a domain-tagged Rust unit result envelope and Rust-specific admission path. Reuse bounded result-source transport and immutable result references where compatible.

Keep the existing derivation action-result schema and PathInfo admission unchanged.

**Rationale:** Compiler unit trees do not have derivation store-path semantics. Shared transport does not require shared semantic records.

### Decision 2: Bind remote authority to canonical record bytes

**Choice:** Sign the canonical Rust unit result envelope. Bind its schema, action reference, result reference, castore root, artifact manifest, producer identity, producer-policy identity, and claim class.

Trust policy will match full verifier key material and policy identity. Signer names remain descriptive only.

**Rationale:** A matching key name or transport source cannot prove record authority.

### Decision 3: Keep discovery bounded and advisory

**Choice:** Query a bounded ordered set of configured result sources by canonical action reference. Enforce named limits for candidate count, metadata bytes, redirects, retries, and transfer bytes.

Index candidates will not skip execution. Rust-specific admission will decide whether one result can be reused.

**Rationale:** Discovery reports claims. It does not establish content or trust.

### Decision 4: Fetch immutable content before final admission

**Choice:** Fetch missing referenced blobs and directories through the object-store interface. Verify every object identity while ingesting it locally.

After transfer, run local result schema, signature, policy, action, manifest, bounds, and complete-tree admission. Materialize only after admission succeeds.

**Rationale:** A valid signed record can still reference missing or corrupt content.

### Decision 5: Publish objects, records, and indexes in dependency order

**Choice:** Publish complete immutable castore objects first. Publish the signed immutable result envelope second. Publish the no-clobber action index candidate last.

Readers must observe either the old complete candidate set or the new complete candidate set. Publication will preserve multiple different result references for one action.

**Rationale:** Last-writer-wins publication can expose partial content and hide nondeterminism.

### Decision 6: Preserve deterministic source and conflict policy

**Choice:** Use configured source priority for equivalent candidates. Do not use response timing as authority.

If multiple admissible result records for one action describe different artifacts, strong reuse will fail with explicit nondeterminism evidence.

**Rationale:** Source order can optimize discovery but cannot resolve conflicting output truth.

### Decision 7: Make offline and failure behavior explicit

**Choice:** Offline mode will not open remote result or object sources. A transport, signature, policy, metadata, completeness, or materialization failure will not fabricate a hit.

When execution policy permits, Mantle can continue to another candidate or compile after an advisory rejection. Strong conflict policy can block instead.

**Rationale:** Shared cache availability must not silently change declared offline or conflict policy.

### Decision 8: Report cache route and transfer facts

**Choice:** Rust topology receipts will identify source, sanitized cache identity, candidate result identity, authority decision, hit or rejection reason, transferred bytes, reused bytes, and compiler execution.

Receipts will not contain credentials, bearer tokens, signed URL queries, raw remote configuration, or private key material.

**Rationale:** Operators need enough evidence to explain a hit without exposing secrets.

## Failure Semantics

- Oversized or malformed candidate lists reject the source response.
- Invalid signatures reject the candidate before content admission.
- Missing or corrupt objects reject the candidate after bounded transfer attempts.
- A materialization failure leaves no successful execution output.
- Remote publication failure does not invalidate a successful local compiler result.
- Conflicting admissible results remain visible and are never resolved by last writer.

## Risks / Trade-offs

- Shared result records add another signed schema and retention surface.
- Clean-client hits can transfer many small compiler artifacts and directory objects.
- Strict conflict handling can stop a build that could compile locally.
- Remote publication can increase compiler miss latency unless publication is bounded and policy-controlled.
