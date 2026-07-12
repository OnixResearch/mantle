# ADR 0014: Package ast-grep as bounded structural evidence

## Status

Proposed

## Context

Stack repositories need reproducible structural scans and rule tests, but a bare ast-grep executable in an ambient developer environment does not identify the binary that ran. Raw findings also invite semantic overclaims: a structural match or passing rule fixture does not establish source behavior, build correctness, cache correctness, or release eligibility.

Mantle already owns pinned Nix package surfaces, BLAKE3 evidence identities, build-report sidecar discovery, and generic release external-evidence attachment. Repository owners, not Mantle, own ast-grep rules, scan scopes, command normalization, and finding semantics.

## Decision Drivers

- Pin the exact ast-grep package version and identify executable bytes reproducibly.
- Keep rule catalogs and rule semantics in the repository that owns them.
- Keep process execution and filesystem reads outside pure validation logic.
- Bind tool, command, rule bundle, scan input, receipt, and sidecar evidence with BLAKE3.
- Reject stale identities, malformed evidence, and release overclaims deterministically.
- Preserve build and release evidence as structural-tool evidence only.

## Decision

Mantle will expose ast-grep through a flake package named `ast-grep-toolchain` and the default development shell. The package asserts an explicit version pin, copies the executable into its profile, and emits a generated identity record containing the executable BLAKE3. A Nix package-identity check recomputes the digest and runs the packaged version command.

Repository-owned workflows may emit `mantle-ast-grep-structural-evidence-v1` sidecars for `scan` and `rule-test` commands. The sidecar records expected and observed tool, rule-bundle, and scan-input identities; command and receipt identities; output format and bounded finding counts; structural-only labels; and mandatory non-claims.

Pure parsing, validation, canonicalization, and canonical BLAKE3 identity live in the no-std `crunch-release-core` crate. The root-package shell owns bounded filesystem reads, raw-file hashing, build-output discovery, diagnostics, and release attachment. Mantle does not invoke ast-grep automatically and does not interpret findings.

Valid output sidecars appear in `ast_grep_structural_evidence[]`; invalid sidecars appear only in `ast_grep_structural_evidence_diagnostics[]`. Generic release external evidence may carry this schema only after the same sidecar validation and exact non-claim preservation.

## Alternatives Considered

### Add `pkgs.ast-grep` only to the development shell

Rejected because the environment would expose a version but no Mantle-owned executable identity record or focused package smoke.

### Run ast-grep from the pure evidence validator

Rejected because process, filesystem, and environment access would couple validation to ambient state and violate the functional-core/imperative-shell boundary.

### Maintain a Mantle rule catalog

Rejected because rule semantics belong to each source repository and would turn Mantle into a policy owner for unrelated codebases.

### Treat passing scans as build or release correctness

Rejected because structural evidence cannot establish runtime behavior, compiler correctness, output correctness, cache trust, or release eligibility without a separate narrower gate.

## Consequences

- Updating ast-grep requires a deliberate version-pin change and fresh package identity evidence.
- Sidecar producers must define deterministic bytes for argv, rule bundles, scan scopes, and receipts before hashing.
- Build reports gain additive accepted-evidence and diagnostic arrays.
- Release attachment remains identity and shape evidence; downstream policy must make any narrower promotion explicitly.
- Sidecars with stale hashes, wrong tool identity, missing non-claims, unsupported formats, or overclaimed labels fail closed.
