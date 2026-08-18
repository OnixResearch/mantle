## Context

Remote-build specs already state that tickets authorize resource use, not output trust. The implementation needs one admission path that both the planner and importer can use, so remote output acceptance cannot drift between CLI flags, coordinator decisions, stdio clients, and later P2P clients.

## Decisions

### 1. Output admission is independent of access admission

**Choice:** Ticket redemption and trusted-client authorization are checked before queue admission. Output acceptance is checked separately using trusted key material, attestation policy, requested identity, and content facts.

**Rationale:** A valid ticket proves permission to spend builder resources; it does not prove the returned bytes are acceptable.

### 2. Key material beats key name

**Choice:** Trust checks deduplicate and compare signer keys by public key material digest, not just signer name.

**Rationale:** Fresh stores can reuse default signer names; name-only matching admits wrong-key or drops valid same-name alternatives.

### 3. Preflight when possible, final admission always

**Choice:** If the builder advertises signing keys or attestation authorities, the client should reject remote routing before dispatch when no acceptable output trust path exists. Final returned outputs are still verified before persistence.

**Rationale:** Preflight saves remote work, while final admission protects against stale or tampered responses.

## Risks / Trade-offs

- Strict trust preflight can reject a build that might later return a valid receipt through an unadvertised path; such behavior should require explicit policy.
- Trust reports must identify enough evidence for debugging without revealing private key paths or bearer secrets.
