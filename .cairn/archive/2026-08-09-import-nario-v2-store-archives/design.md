# Design: bounded Nario v2 read compatibility

## Context

Mantle owns a streaming archive format with bounded list, import, and export behavior. Its accepted specification forbids unproven Nario compatibility claims.

Determinate Nix Nario v2 places path metadata before each NAR payload. This allows bounded skipping when a path already exists.

The Mantlepkgs converter needs concrete recipes from derivation export. Nario can supply payloads, but it cannot replace those recipes.

## Decisions

### Decision: Pin the external format authority

**Choice:** Select one exact Determinate Nix source revision and Nario v2 producer version. Retain generated positive and negative fixtures with source identities.

**Rationale:** Compatibility claims need a stable byte-level authority. A blog description is not a complete format contract.

### Decision: Support read operations only

**Choice:** Add `nario-v2` as an explicit format for archive list and import. Keep Mantle-native archive behavior unchanged and keep Nario export unsupported.

**Rationale:** The converter needs to consume producer archives. Nario export is not required for the first useful path.

### Decision: Share one pure framing and validation core

**Choice:** A pure core decodes bounded byte frames, validates record state transitions, normalizes metadata, checks named limits, and produces deterministic diagnostics.

The shell owns stream reads, seeking or draining, staged NAR ingestion, store queries, signature policy, materialization, and report output.

**Rationale:** Format parsing and state decisions must be testable without a store, filesystem, or asynchronous reader.

### Decision: Direct store import preserves exact path identity

**Choice:** Direct Nario store import accepts a record only when its logical store prefix matches the target store configuration.

The importer verifies NAR size and hash, references, signatures, supported content-addressed metadata, and exact path identity before PathInfo admission.

**Rationale:** A store archive transport must not silently rename paths. Path rewriting belongs to foreign graph compilation and source materialization.

### Decision: Foreign source preparation is a separate projection

**Choice:** Source preparation may select a Nario record only when an admitted executable plan names that exact foreign source path.

The adapter verifies the original record, decodes its NAR, canonicalizes the source tree, and creates a target source record for the plan's mapped path.

The resulting source record binds the Nario archive BLAKE3, original PathInfo facts, target source identity, plan identity, and source requirement identity.

**Rationale:** Original Nix signatures authenticate the original path. They do not authenticate a new Mantle target path.

### Decision: Do not project arbitrary built outputs as sources

**Choice:** Nario source projection is limited to exact non-derivation source requirements from the admitted plan.

Built tools, libraries, and package outputs require direct store import under matching identity or normal Mantle graph rebuilding.

**Rationale:** Relabeling an arbitrary built output as source would bypass recipe, reference, and authority checks.

### Decision: Keep memory and mutation bounded

**Choice:** Named policy limits bound metadata, records, references, signatures, payload size, chunks, total bytes, diagnostics, and staged paths.

The shell stages archive records until it reaches a valid end of stream. Staged content cannot create usable PathInfo or source records.

After complete validation, the shell publishes all new PathInfo records under one mutation boundary. A commit failure must roll back visible path state.

**Rationale:** Nario exists partly to avoid memory use proportional to its largest payload. Staging preserves that property while preventing partial archive visibility.

### Decision: Keep trust claims separate

**Choice:** Format compatibility proves parsing and checked field preservation for the pinned Nario version. Store trust still comes from explicit Mantle policy.

Source projection records original signature evidence as provenance only. Mantle applies its own target source and PathInfo admission rules.

**Rationale:** Byte compatibility does not grant trust or prove package correctness.

## Risks / Trade-offs

- Determinate Nix can change Nario after the pinned revision.
- The format can contain metadata that Mantle does not support in the first version.
- Full-archive staging can require disk space even when memory stays bounded.
- Source projection adds decoding work that direct Nix store import does not need.
- Nario does not remove the need for derivation graphs, package indexes, policies, or producer receipts.
