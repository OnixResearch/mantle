# Runtime source profile

Date: 2026-09-04

## Accepted profile

- Build-relevant source commit: `97f47ae2a644651734324f44991cd4cae847499a`.
- Orchestrator BLAKE3: `3a8e1b46c634cb31f44b8f63ecd9d675dce2a1c236d051b91a938697e2f8f40c`.
- Manifest BLAKE3: `23d48a82ace18957fc18a7ab1973172aff499dd440d9b4dcfff34b236f38af07`.
- Profile readiness: `Ready`.
- Missing, stale, unsupported, and untrusted record counts: zero.
- Source transfer: checksum-exact from a Git archive of the implementation commit.
- Vendor transfer: checksum-exact from the current two-root directory source.

The profile preserves the immutable V98 source records. It replaces only the
Mantle source and vendor records with the current build inputs.

## Failed attempts

1. The first refresh rejected a missing `vendor-deps/` directory.
2. The second refresh rejected the copied V98 vendor tree because the lockfile
   contained duplicate `artifact-auth-core 0.1.0` sources.
3. The third refresh rejected the then-current single-root vendor tree for the
   same reason.

The accepted rerun used ADR 0120's separate Cargo directory roots. No proof
attempt started before profile readiness passed.

## Boundary

The profile proves declared source availability and identity. It does not prove
stage execution, provider correctness, fixed-point equality, or release
eligibility.
