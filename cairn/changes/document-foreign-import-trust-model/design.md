## Context

The foreign import core emits receipts that bind graph and policy digests plus explicit non-claims. Operators need a concise guide that explains the difference between admission, planning, realization, substitution, and proof evidence.

This doc should connect to the existing operator proof guide and use the project-facing Mantle name while preserving exact command/schema identifiers.

## Decisions

### 1. Non-claims are first-class documentation

**Choice:** The trust-model guide starts from what a receipt does not prove before describing what it binds.

**Rationale:** The highest risk is overclaiming imported artifacts.

### 2. Trust scopes are separated

**Choice:** Document graph provenance, source content verification, binary cache/substitution trust, sandbox compatibility, and build-output verification as separate trust decisions.

**Rationale:** A valid import receipt should not collapse independent trust boundaries.

### 3. Examples use fixture-level imports

**Choice:** Use small Guix-like and Nix-like hello examples, not full package set claims.

**Rationale:** The examples should teach boundaries without implying full ecosystem support.

### 4. Guard docs against regression

**Choice:** Add a lightweight script or test that checks for required headings and non-claim phrases.

**Rationale:** Trust docs tend to decay when commands evolve.

## Risks / Trade-offs

- Too much caveat text can obscure the useful operator workflow; keep it structured.
- Guard text can become brittle if it matches prose too exactly; prefer required headings and key terms.
- Future CLI changes may require doc updates before the guard passes.
