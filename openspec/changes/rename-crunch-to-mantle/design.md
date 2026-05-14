## Context

Crunch is a Rust/Nickel replacement-for-Nix stack with user-facing names embedded across the binary, package metadata, docs, default logical store prefix, project files, proof receipts, and archived OpenSpec history. A rename after more release evidence ships would make compatibility and proof identity harder to reason about.

## Goals / Non-Goals

**Goals:**
- Establish `Mantle`/`mantle` as the canonical product and command identity.
- Keep the rename auditable with explicit compatibility and migration rules.
- Preserve existing proof boundaries: rename evidence must not be treated as proof of stronger bootstrap correctness.
- Add deterministic checks that prevent mixed or stale user-facing branding.

**Non-Goals:**
- Rewriting archived OpenSpec history except where generated indexes or docs intentionally summarize current identity.
- Changing derivation hashing, scheduler behavior, sandboxing, or bootstrap semantics beyond path/name defaults.
- Removing every internal historical comment in the first implementation if it is not user-facing or artifact-facing.

## Decisions

### 1. Canonical CLI name is `mantle`

**Choice:** The shipped primary binary and documentation use `mantle`.

**Rationale:** The project needs one canonical operator command before broader release evidence and package docs stabilize.

**Alternative:** Keep `crunch` as the binary and only rename the project in prose. Rejected because it preserves the highest-friction user-facing collision.

**Implementation:** Rename package/binary metadata and CLI help; optionally keep a `crunch` wrapper/alias that prints or records a transitional notice and delegates to the same implementation.

### 2. Compatibility is explicit and bounded

**Choice:** Existing Crunch-named project files, directories, and logical store paths are supported only through named compatibility behavior or migration commands.

**Rationale:** Silent mixed identity makes build proofs and operator docs harder to audit.

**Alternative:** Blindly accept both spellings forever. Rejected because it makes tests and docs ambiguous.

**Implementation:** Introduce tests for `mantle` defaults, legacy input handling, and conflict cases where both old and new project files exist.

### 3. Artifact identity changes without proof inflation

**Choice:** Release, attestation, self-build, and parity receipts use Mantle as product identity after migration while retaining existing narrow proof semantics.

**Rationale:** Rename should not imply a new trust root or stronger reproducibility claim.

**Alternative:** Treat rename as a new release-proof milestone. Rejected because it conflates branding with bootstrap correctness.

**Implementation:** Update manifest schemas/fixtures/docs that name the product and add regression coverage for canonical identity fields.

### 4. Prevent stale branding with a repo check

**Choice:** Add a deterministic allowlist-based check for remaining user-facing `Crunch`/`crunch` occurrences.

**Rationale:** A rename is easy to partially complete and hard to review manually.

**Alternative:** Rely on code review grep. Rejected because the repo has many docs, fixtures, and evidence files.

**Implementation:** Script or Rust test scans tracked files, ignores archives/history and explicit compatibility tests, and fails on unclassified user-facing old names.

## Risks / Trade-offs

**Compatibility churn** → Mitigate with a transitional alias and project-file migration coverage.

**Store-path breakage** → Mitigate by testing default `/mantle/store` behavior separately from legacy `/crunch/store` compatibility and `--nix-compat` behavior.

**Proof artifact ambiguity** → Mitigate by requiring canonical product identity in new release/proof manifests while preserving old archived evidence as historical.

**Over-broad mechanical rename** → Mitigate by staged tasks and an allowlist for historical or compatibility-only occurrences.
