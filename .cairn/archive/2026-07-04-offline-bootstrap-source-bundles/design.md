## Context

Mantle's source-bundle machinery can already represent bootstrap archives and provider/toolchain source roots, but bootstrap commands do not yet present an end-to-end offline source bundle path. Operators can fetch, reduce, and self-build, yet they lack one deterministic way to verify “all bootstrap inputs are locally imported and pinned before the build starts.”

The change should extend the source-bundle/preflight model into bootstrap workflows without widening bootstrap claims. A complete bundle makes source acquisition offline; it does not make the reduced provider inherently trustworthy or prove compiler correctness.

## Decisions

### 1. Bootstrap gets a named source-bundle profile

**Choice:** Define a profile that enumerates required source records for the selected bootstrap mode: pinned provider archive, provider manifest/reducer metadata, bootstrap source archives, Mantle source tree, vendored Cargo inputs, and optional proof inputs.

**Rationale:** A named profile gives operators and tests a stable checklist instead of ad hoc `--source` arguments.

### 2. Commands consume imported source state before fetchers

**Choice:** `mantle bootstrap --fetch`, bootstrap validation, and self-build entry points check imported/pinned source state first when an offline bootstrap source-bundle mode is selected. Live fetchers remain explicit non-offline behavior.

**Rationale:** Offline behavior should be opt-in and fail closed. Existing online bootstrap fetch behavior can remain available for connected hosts.

### 3. Provider metadata is validated at the bundle boundary

**Choice:** Bootstrap source records carry provider kind, source identity, fixed-output hash, reduction metadata, and expected normalized seed contract facts. The offline preflight rejects records that do not match the selected provider/proof mode.

**Rationale:** A source bundle that contains “some toolchain” is not enough. The records must match the bootstrap contract Mantle will consume.

### 4. Vendored Cargo inputs share the same source-state path

**Choice:** Self-build source/vendor material is represented as package-manager or local-path source records and pinned with the bootstrap profile when used in an offline proof path.

**Rationale:** Vendored inputs are just as important to offline self-build as the seed provider. They should not remain a separate untracked prerequisite.

### 5. Claims stay source-acquisition-bounded

**Choice:** Reports say the bootstrap input bundle is complete, imported, and identity-matched. Stronger self-build or release claims still require the existing proof commands and evidence gates.

**Rationale:** This follows Mantle's proof-before-claim rule and avoids turning input availability into bootstrap correctness.

## Risks / Trade-offs

- Bootstrap modes have different required inputs; profile generation must avoid over-requiring proof inputs for ordinary diagnostics and under-requiring them for proof admission.
- Provider metadata has evolved; the validator should reject unsupported versions with clear remediation instead of guessing.
- Large source/vendor bundles may be expensive to export; progress reporting and bounded limits should be explicit.
