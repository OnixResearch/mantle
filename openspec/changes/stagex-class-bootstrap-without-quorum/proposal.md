# StageX-class bootstrap without quorum

## Why

StageX sets the benchmark for a fully bootstrapped and technically verified
build: an auditable tiny seed, no opaque binary toolchain root, deterministic
rebuild evidence, and release artifacts whose source-to-binary path can be
checked by another operator. Crunch now has source-root provider work,
self-hosting proof bundles, protected host-tool boundaries, and release
reproducibility/witness plumbing, but it still needs one explicit target change
that ties those pieces into a StageX-class claim.

This change defines the engineering target for reaching that claim while leaving
multi-maintainer quorum policy for a later change.

## What Changes

- **Stage0 lineage target.** Define a source-only bootstrap lineage from an
  auditable seed through stage0-posix/live-bootstrap-style tools into the
  normalized seed provider consumed by Crunch. The first accepted seed class is
  exactly `hex0-seed`; it must have an instruction set or bytecode language,
  entry point, I/O contract, host-interface surface, checked-in human-readable
  source, source-to-byte reproduction transcript, manual audit note, and explicit
  byte budget.
- **No host compiler trust.** For the target profile, host compilers, host make,
  host archive tools, Nix, and the legacy musl.cc provider are forbidden as
  trusted bootstrap roots rather than merely recorded as trust notes.
- **End-to-end proof binding.** Extend the proof target so stage0 lineage,
  provider output, self-build fixed point, and reproducibility report are bound
  together before any StageX-class claim appears.
- **Verified-build profile.** Define a release verification profile that requires
  full-source bootstrap evidence plus byte-identical rebuild evidence, but does
  not require social signer quorum yet.

## Non-Goals

- Quorum artifact signing, maintainer threshold policy, or social trust policy.
- Replacing Crunch's derivation/store/scheduler model with OCI packaging.
- Claiming every future package is bootstrapped; this change targets Crunch's
  own bootstrap/release path first.
- Proving firmware, CPU microcode, kernel, or host hardware provenance.

## Capabilities

### New Capabilities

- `bootstrap-stagex-lineage`: audited seed to normalized provider lineage.
- `release-stagex-verified-no-quorum`: technical verified-build profile without
  social quorum.

### Modified Capabilities

- `bootstrap-full-source-root`: tightened from source-root provider evidence to
  source-only lineage evidence for StageX-class claims.
- `release-evidence-reproducible`: used as a mandatory input for the target
  profile rather than optional release evidence.

## Impact

- **Files**: `bootstrap/`, `src/bootstrap.rs`, `src/self_build.rs`,
  `scripts/prove-self-hosting.sh`, release-evidence code, README/bootstrap docs,
  and tests/evidence helpers.
- **APIs**: add `crunch release verify <bundle-dir> --require-stagex-no-quorum`
  and a JSON `stagex_no_quorum` report object for the technical profile.
- **Dependencies**: may import stage0-posix/live-bootstrap source inputs, all
  pinned with BLAKE3 digests where Crunch owns the fingerprint.
- **Testing**: manifest/lineage negative tests, no-host-tool exec-denial tests,
  provider contract tests, full self-build proof with lineage provider, and
  release verification requiring a reproducibility report.

## How to validate

1. `openspec validate stagex-class-bootstrap-without-quorum --strict` passes.
2. Stage0 lineage validation rejects seeds without audit-bound metadata,
   oversized seeds, host compiler, host make/archive tools, Nix, legacy musl.cc
   inputs, missing digests, undeclared generated files, and provider outputs not
   reachable from the declared lineage.
3. A full proof run records the audited seed digest, stage lineage digest,
   normalized provider digest, and stage1/stage2 fixed-point digests.
4. Release verification records both proof bundle digest and reproducibility
   report digest, and reports the StageX-class no-quorum profile as satisfied
   only when full-source lineage proof and byte-identical reproducibility report
   both verify.
5. Docs state that quorum remains unsolved and do not use quorum-satisfied or
   multi-signer language for this profile.
