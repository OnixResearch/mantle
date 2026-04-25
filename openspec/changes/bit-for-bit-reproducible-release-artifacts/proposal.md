# Bit-for-bit reproducible release artifacts

## Why

Release evidence bundles currently prove bundle-local integrity and proof
context. They do not prove that another environment can recreate the exact same
release bytes. The stronger release milestone is a reproducibility workflow that
rebuilds the published artifacts and compares bytes, not just claims or local
metadata.

## What Changes

- **Define reproducible release artifacts.** Specify which files must be
  byte-identical and which metadata is intentionally canonicalized.
- **Add a reproducibility check workflow.** Rebuild from release source/proof
  context into a separate output area and compare BLAKE3 digests and byte
  lengths for every published artifact.
- **Record reproducibility evidence.** Emit a canonical report that can be
  packaged in release evidence and referenced by release verification.
- **Fail closed on drift.** Detect binary bytes, source archive, manifest,
  proof-linkage, and packaging metadata drift.

## Non-Goals

- Guaranteeing every downstream package built by crunch is reproducible.
- Proving social independence by itself; that comes from independent rebuild
  agreement.
- Changing release attestation signatures unless new report fields require it.

## Capabilities

### New Capabilities
- `release-reproducibility-report`: canonical report for byte-identical rebuild
  evidence.
- `release-artifact-byte-compare`: compare rebuilt artifacts against published
  artifacts by length and BLAKE3 digest.

### Modified Capabilities
- `release-evidence-bundle`: optionally carry reproducibility evidence and
  expose the stronger claim only when it verifies.

## Impact

- **Files**: release evidence core/adapters, release CLI, self-hosting proof
  helpers, docs, release tests.
- **APIs**: likely new `crunch release reproduce` or `crunch release verify
  --require-reproducible` surface.
- **Dependencies**: no non-deterministic packaging tools.
- **Testing**: positive byte-identical fixture and negative one-byte drift,
  missing artifact, and metadata-drift cases.

## Relationship to Other Changes

This change is strongest after `independent-rebuild-agreement`, but the byte
comparison rail can be built first using local fixture rebuilds.

## How to validate

1. `openspec validate bit-for-bit-reproducible-release-artifacts --strict`
   passes.
2. Reproducibility reports serialize canonically.
3. Verification accepts byte-identical artifacts and rejects one-byte drift.
4. Docs reserve the reproducible-release claim for bundles with verified
   reproducibility evidence.
