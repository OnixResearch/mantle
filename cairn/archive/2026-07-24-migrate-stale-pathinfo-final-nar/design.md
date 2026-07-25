## Context

The CA archive repair separates marker-normalized CA path identity from final NAR identity and makes archive export reject stale final-NAR metadata. Historical signed records remain immutable evidence of what was signed: changing their NAR facts invalidates every existing signature. The castore node, references, CA metadata, deriver, and logical store path can still be valid and complete, so rebuilding is not always necessary if an explicit local signer accepts responsibility for a measured metadata migration.

Success requires observable dry-run non-mutation, exact-path selection, independent CA-path validation, complete local content, freshly rendered final NAR facts, replacement rather than appended signatures, preserved artifact-attestation claims/graph, and post-write verification. False completion includes signing stale facts, accepting a fragment or `--all`, changing content or CA metadata, preserving invalid signatures, generating a key during dry-run, silently dropping attestation provenance, or reporting success after partial persistence.

## Decisions

### Decision: Use one exact-path, dry-run-first repair command

**Choice:** Add `store repair-final-nar` with a required full logical path, dry-run default, and explicit `--execute`. Do not support fragments, batches, or `--all`.

**Rationale:** Historical repair is exceptional authority, not ordinary maintenance. Exact selection and explicit execution bound operator intent and avoid a mass resigning surface.

### Decision: Derive the repair plan in a pure core

**Choice:** A deterministic planner accepts recorded PathInfo facts, observed final NAR facts, signature count, CA-path validity, and sidecar facts, then returns `Current`, `WouldRepair`, or a typed rejection. The async shell owns PathInfo lookup, castore completeness, NAR rendering, key loading, staged sidecar I/O, persistence, and post-write verification.

**Rationale:** The safety policy can be exhaustively tested without opening databases or standing up castore services, while the shell remains explicit and bounded.

### Decision: Replace all stale signatures with one selected local signature

**Choice:** Execution clears every existing signature and signs the repaired Nix fingerprint exactly once with the selected key. It never appends a repaired signature beside signatures over stale facts.

**Rationale:** Nix signatures bind store path, final NAR SHA-256/size, and references. Old signatures are invalid after metadata changes and retaining them would misrepresent authority.

### Decision: Preserve and refresh existing artifact evidence

**Choice:** If an artifact attestation exists, mutate only its observed content digest to the freshly rendered final NAR digest, preserving output name, claims, nodes, and edges. Stage canonical replacement bytes before PathInfo mutation, atomically publish the sidecar after PathInfo persistence, and attempt PathInfo rollback if sidecar publication fails. Missing sidecars remain missing and are reported.

**Rationale:** Re-synthesizing without original provenance would erase claims and graph facts. Leaving the old digest would create stale evidence. Staging before database mutation rejects predictable filesystem failures without partial state.

### Decision: Share CA path-identity validation with archive transport

**Choice:** Extract the existing pure CA-derived path check into a crate-private shared module used by both archive and repair paths.

**Rationale:** Migration must not legitimize corrupt or unsupported CA metadata, and duplicated identity logic would drift.

## Alternatives considered

- **Rebuild only:** strongest provenance, but may be impossible for retained historical environments and does not provide the requested explicit metadata migration.
- **Archive export/import:** archive export intentionally rejects stale records, so using it would weaken the repaired fail-closed boundary.
- **Extend `store sign`:** would mix ordinary signature management with metadata mutation and make accidental repair authority too broad.

## Risks / Trade-offs

- The PathInfo database and artifact sidecar are not one transactional store. Staging plus atomic sidecar publication and best-effort PathInfo rollback bounds but cannot mathematically eliminate a catastrophic double-write failure; diagnostics MUST name such an inconsistency and MUST NOT report success.
- A local signer assumes responsibility for the newly measured facts. Migration does not recover or transfer the authority of discarded historical signatures.
- The command proves local content/metadata consistency only. It does not prove the builder, source, compiler, output semantics, historical provenance, or release eligibility.
