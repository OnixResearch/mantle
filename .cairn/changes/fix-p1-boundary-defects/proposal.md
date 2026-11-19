# Proposal: Fix the P1 boundary defects

## Why

The open P1 issues are boundary defects, not independent bugs. Secret parsing
can echo key material, GC asserted a signing invariant that explicit unsigned
imports violate, a panic or unobserved teardown could park a worker forever,
sandbox output capture was unbounded, tar acquisition had no byte budgets, and
the flake still resolved a transitive SSH source. Each one lets an untrusted or
merely large input cross a boundary without a bounded, typed outcome.

## What Changes

- Admit signing-key files through the shared private-file discipline and make
  the parser redact decode failures and verify the public/secret half match.
- Preserve unsigned retained PathInfo during GC while keeping the build-output
  signing refusal at `persist_and_export_signed_output`.
- Complete cancellable worker tasks on panic or unobserved teardown with a
  failed outcome; let the service own quarantine until teardown is observed.
- Drain sandbox stdout/stderr under a per-stream bound, with explicit
  truncation markers and pre-spawn environment byte validation.
- Enforce compressed, decompressed, per-entry, and cumulative fetch/tar byte
  budgets with bounded spooling and exact-limit overflow probing.
- Redirect the pinned valence transitive artifact source from SSH to HTTPS so
  flake evaluation is credential-free.
- Publish first-run signing keys through atomic no-replace temporary-file
  persistence so concurrent callers share one complete private key.
- Strip setuid, setgid, and sticky bits from untrusted tar entries and fail
  extraction when a required permission change fails.
- Create sandbox work roots under the private state directory with explicit
  owner/type/mode admission and an operator override.

## Impact

- **Immediate outcome**: the six P1 issues have code and regression coverage.
- **Durable capability**: untrusted acquisition, secret loading, diagnostics,
  and shutdown all produce bounded, classified outcomes.
- **Maintenance owner**: Mantle store and fetch boundary owners; the vendored
  `nix-compat` and `snix-build` changes are local patches that must survive
  upstream syncs.
- **Repeatability evidence**: focused tests recorded under `evidence/`.
- **Compatibility**: private keys must now use private modes; unsigned Snix
  PathInfo remains supported; Casita keeps its stronger destination policy.

## Out of Scope

- The P2 issues in the same review series.
- A replacement tar parser or REAPI backend.
- Claiming arbitrary sandbox, key-management, or fetch correctness.
