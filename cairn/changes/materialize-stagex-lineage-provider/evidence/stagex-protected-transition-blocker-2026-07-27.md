# StageX protected-transition blocker — 2026-07-27

## Question

Can the current source-built native artifacts become a complete StageX lineage provider without relabeling host-assisted effects?

## Inspected evidence

### Current receipt and manifest state

- `bootstrap/evidence/stagex-lineage-provider-receipt.json` has `lineage_receipt_status = scaffold-only` and placeholder digests.
- No real StageX lineage manifest exists outside synthetic Rust test fixtures.
- `src/main.rs::cmd_bootstrap_stagex_lineage` validates a manifest, then returns `STAGEX_LINEAGE_PROVIDER_NOT_MATERIALIZED`.
- No protected-exec audit exists under `bootstrap/evidence/`.
- No exported `hex0-seed` binary exists under `/home/brittonr/.cache/mantle-full-source-20260718`.
- A built `stage0-posix` output exists at `/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store/wisq2xjp7b646gvscc09qad8svi0a3qc-stage0-posix`.
- That output contains produced transition tools. It does not preserve the audited seed as a separately verified input artifact.

### Host-assisted lineage construction

The required chain still uses ambient sandbox paths and store discovery:

- `bootstrap/stage0-posix.ncl`: `builder = "/bin/sh"` at line 68, `BB=/bin/busybox` at line 73, and `$NIX_STORE` discovery at line 77.
- `bootstrap/mes.ncl`: the same dependencies appear at lines 28, 33, and 36.
- `bootstrap/tinycc-mes.ncl`: the same dependencies appear at lines 33, 39, and 42.
- `bootstrap/tinycc.ncl`: the same dependencies appear at lines 37, 43, and 46.
- `bootstrap/gcc-10-final.ncl`: the same dependencies appear at lines 14, 20, and 30.
- `bootstrap/seed-full.ncl`: the same dependencies appear at lines 39, 45, and 56.
- `bootstrap/busybox.ncl` and `bootstrap/bwrap.ncl` also use `/bin/sh`, `/bin/busybox`, and `$NIX_STORE` discovery.

The repository-wide search found this pattern throughout the Mes, TinyCC, musl, GNU, GCC, binutils, provider, and self-build chain.

### Existing evidence is narrower

- The admitted full-source provider binds its final tree and authenticated source closure.
- Its attestations bind artifacts to immediate recipes and build inputs.
- Those records do not contain a one-way protected transition or per-stage protected-exec decisions.
- `bootstrap/evidence/full-source-provider-fixed-point.json` has `provider_kind = full-source`.
- The same report records `stage0.fallback_event_count = 2`.
- Authenticated persisted outputs therefore remain full-source artifacts. They are not resumable StageX completion evidence.

### Baseline validation

- Pueue task `1283`: `bootstrap_source_root` passed 26 tests with zero failures.
- Pueue task `1285`: `protected_exec_seccomp` passed 11 tests with zero failures.
- Pueue task `1284`: `self_build` passed 206 tests with zero failures.
- Pueue task `1286`: `bootstrap_parity` passed 88 tests with zero failures.

The local adversarial review found no valid promotion path from the current artifacts. It agreed that complete per-stage lineage and protected-exec evidence is missing. This review is advisory only.

## Decision

Do not replace the scaffold receipt. Do not publish a StageX provider from the existing persisted native outputs.

The current artifacts cannot meet the accepted requirement. Their construction still depends on host shell, BusyBox, sandbox, and path-discovery effects after the intended transition.

## Exact completion blocker

A complete run needs all of these missing mechanisms and artifacts:

1. An exported audited `hex0-seed` input with checked seed bytes, BLAKE3, audit bound, and source-to-byte transcript.
2. A real bounded StageX manifest with immediate predecessor edges, source identities, output roles, limits, and a declared transition point.
3. Post-transition orchestration that does not execute host `/bin/sh`, BusyBox, bwrap, compiler, linker, or path-discovery fallbacks.
4. Per-stage observed reports that bind exact executable paths, roles, source stages, BLAKE3 values, inputs, and outputs.
5. A protected-exec audit for every post-transition executable decision.
6. Independent provider and receipt validation before create-new publication.

Replacing only the scaffold JSON would create false evidence. Reusing the existing full-source provider would relabel host-assisted effects as StageX lineage.

## Owner

Mantle StageX lineage-provider implementation.

## Next action

First replace the post-transition shell and sandbox orchestration with a source-produced runner. Then rebuild from the exported audited seed and retain every protected stage report.

I1 through V3 remain unchecked. The fixed-point and parity-promotion successor changes remain blocked by this change.

## Lifecycle validation

Pueue tasks `1287` through `1290` validated this exact blocker packet.

- Cairn validation: `valid: true`, `issues: []`, 35 specs validated.
- Proposal gate: `PASS`, receipt hash `5bd3935488d3f8b6ec3e5ae3133811dc5f4d56370dcd3a36b526b6a64d8a53a7`.
- Design gate: `PASS`, receipt hash `9a6d746e3fcee4bf3958d57462b83dfe6dfc66112677c2283dbab62858536504`.
- Tasks gate: `PASS`, eight tasks remain unchecked, receipt hash `c76a9f464b17102361d8428a04d9790dd093c5a55aad0ffa651287fb8bbd741d`.

The stderr files are empty. Passing scaffold gates validates the change packet. It does not establish StageX provider completion.
