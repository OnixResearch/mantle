## Context

`crunch release verify` currently checks that a bundle is internally
consistent. Witness verification can show another rebuild produced matching
published output digests. Neither creates a dedicated reproducibility report that
compares every published artifact byte-for-byte and gates the stronger release
claim.

## Goals / Non-Goals

**Goals**
- Define which release artifacts are subject to byte comparison.
- Produce canonical reproducibility reports.
- Add a CLI workflow that rebuilds into an isolated output area and compares
  bytes against the published bundle.
- Surface reproducibility status separately from witness policy.

**Non-Goals**
- Making every package derivation reproducible by policy.
- Requiring independent witnesses for local byte comparison.
- Treating prerequisite checks as reproducibility proof.

## Decisions

### 1. Compare named release artifacts, not whole directories blindly

**Choice:** use the release manifest's artifact list as the comparison set.

**Rationale:** release bundles contain proof context, logs, and metadata whose
purpose is not necessarily byte reproduction. The release artifact list is the
operator-published surface.

**Alternative:** recursively compare the whole bundle.

**Why not:** that would make logs and host-specific audit context block the
binary reproducibility claim.

### 2. Report length and BLAKE3 digest for every artifact

**Choice:** record both byte length and BLAKE3 digest for expected and observed
artifacts.

**Rationale:** length gives fast human diagnostics; BLAKE3 is the repo default
for content identity.

### 3. Keep reproducibility status orthogonal to agreement

**Choice:** verifier output has a separate reproducibility status.

**Rationale:** a local byte-identical rebuild is not automatically an
independent social agreement, and an independent witness can match digests even
before the publisher packages a reproducibility report.

## Implementation Sketch

1. Add report types and canonicalization in `crunch-release-core` or
   `crunch-attestation-core`, depending on existing ownership seams.
2. Add std adapter code to run the rebuild workflow into an isolated output dir.
3. Compare published artifacts by name, length, and BLAKE3 digest.
4. Let `release verify` load and validate an optional reproducibility report.
5. Add docs and negative tests for one-byte drift, missing artifact, and output
   name mismatch.

## Risks / Trade-offs

**Rebuild workflow can be expensive.** Keep report verification cheap and make
full reproduction an explicit command.

**Packaging metadata may be confused with artifacts.** The spec names the
artifact set from the manifest to avoid over-broad claims.

**Report placement may evolve.** Start bundle-local/file-based to match current
release evidence, leaving hosted publication for later.

## Validation Plan

- Canonical report tests.
- Positive fixture with byte-identical artifacts.
- Negative tests for drift and missing outputs.
- CLI JSON tests for `absent`, `matched`, and `mismatched` status.
- `openspec validate bit-for-bit-reproducible-release-artifacts --strict`.
