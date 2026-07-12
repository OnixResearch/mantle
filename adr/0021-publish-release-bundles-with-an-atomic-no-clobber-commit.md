# ADR 0021: Publish release bundles with an atomic no-clobber commit

## Status

Accepted

## Context

Release creation previously copied artifacts directly into the requested public directory and wrote `manifest.json` after those copies. Any copy, hash, serialization, or verification error could leave a partial public tree that blocked retry and was observable as if it were a bundle.

Publication needs a transaction boundary without moving filesystem authority into the no-std release core or treating a rename as stronger evidence than it is.

## Decision Drivers

- Keep the final destination absent until one complete production-verified bundle is ready.
- Never overwrite a preexisting or concurrent file, symlink, empty directory, or nonempty directory.
- Keep artifact layout, input identity, policy binding, and state transitions pure and deterministic.
- Keep temporary randomness, filesystem capabilities, verification I/O, cleanup, and rename in the shell.
- Make ordinary failures retryable without allowing broad deletion of similarly named siblings.
- Preserve explicit non-claims for crash durability, artifact correctness, and release eligibility.

## Decision

Mantle computes a `mantle-release-publication-plan-v1` value in `crunch-release-core`. The plan sorts and validates the release-relative artifact layout, pre-copy sizes and BLAKE3 identities, and normalized policy facts. Its BLAKE3 identity binds those facts and the release id but excludes the absolute destination and random staging names.

The shell creates a mode-restricted sibling stage through an opened parent capability. Bundle-relative file and tree writes execute beneath the stage capability. The canonical manifest is serialized and written only after every planned artifact has been copied and rehashed against the plan. The normal production bundle verifier then validates the staging root.

Only a verified pure state is commit-eligible. On Linux, the shell commits by calling `renameat2` with `RENAME_NOREPLACE` and the same opened parent directory for source and destination. A concurrent winner therefore causes a commit-race failure and remains unchanged. Success output occurs only after this rename returns successfully.

A current failed stage is removed through the parent capability when possible. Interrupted stages retain a bounded canonical ownership marker that binds the full plan identity, final name, and random stage name. Retry quarantines only an exact marker match; malformed, mismatched, symlinked, or merely similarly named siblings remain untouched. A fresh retry uses a new random stage while preserving the same deterministic plan identity.

## Alternatives Considered

### Assemble directly in a pre-created empty destination

Rejected because partial state becomes public before verification, retries require manual deletion, and an empty-directory check does not provide an atomic commit boundary.

### Check destination absence and use ordinary rename

Rejected because checking and renaming are separate operations and ordinary rename may replace a concurrent winner.

### Encode random stage names in release evidence identity

Rejected because retry-local shell randomness is not release evidence and would make identical publication plans nondeterministic.

### Recursively delete every matching staging-prefix sibling

Rejected because names alone do not establish Mantle ownership or plan/final-destination linkage.

## Consequences

- `mantle release create` now requires an absent final destination; pre-created empty directories are compatibility errors.
- Readers observe no Mantle-created final path before commit and one complete verified bundle after commit.
- Source drift between planning and staging is detected by staged-artifact comparison before manifest publication.
- Failed current stages are isolated, and exact owned stale stages can be quarantined without touching unrecognized siblings.
- Platforms without a no-replace atomic rename primitive fail closed rather than weakening the contract.
- Atomic rename proves local visibility and no-clobber behavior only. It does not by itself prove filesystem crash durability, power-loss persistence, artifact correctness, release eligibility, or reproducibility.
