# StageX protected-transition blocker — 2026-07-27

> Status update: superseded through reduced musl 1.1.24 by
> `protected-transition-v43-musl-2026-07-28/`. Mantle now completes protected
> full Stage0, source-built `mes-m2`, NYACC regeneration, Mes runtime archives,
> TinyCC 0.9.26, TinyCC 0.9.27, GNU Make 3.82, GNU patch 2.5.9, gzip 1.2.4,
> GNU tar 1.12, GNU sed 4.0.9, bzip2 1.0.8, bounded coreutils 5.0, oyacc 6.6,
> bash 2.05b, the TinyCC-to-musl preparation compiler, and the reduced first-musl
> static runtime. The current run recorded 1155 allowed events, zero denied events,
> and zero fallback. The remaining blocker starts at musl-linked TinyCC 0.9.27
> and covers later tools plus provider admission.

## Question

Can the current source-built native artifacts become a complete StageX lineage provider without relabeling host-assisted effects?

## Inspected evidence

### Current receipt and manifest state

- `bootstrap/evidence/stagex-lineage-provider-receipt.json` has `lineage_receipt_status = scaffold-only` and placeholder digests.
- `bootstrap/stagex-transition-lineage.{ncl,json}` now binds the protected seed-to-reduced-musl-1.1.24 manifest and exact compatibility sources.
- `src/main.rs::cmd_bootstrap_stagex_lineage` validates a manifest, then returns `STAGEX_LINEAGE_PROVIDER_NOT_MATERIALIZED`.
- The retained v6 evidence has a complete 132-event seed-to-full-Stage0 protected-exec audit.
- No exported `hex0-seed` binary exists under `/home/brittonr/.cache/mantle-full-source-20260718`.
- The repository has the audited seed at `bootstrap/seeds/AMD64/hex0-seed`.
- The protected transition now stages and verifies that checked-in seed directly.
- A built `stage0-posix` output exists at `/home/brittonr/.cache/mantle-full-source-20260718/gcc40-store/wisq2xjp7b646gvscc09qad8svi0a3qc-stage0-posix`.
- That older output remains host-assisted evidence and is not used by the new transition proof.

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

The protected transition now has checked seed bytes, authenticated direct source records, a typed 63-stage plan, exact executable identities, full Stage0 SHA-256 interoperability checks, and an 1155-event observed audit through reduced musl 1.1.24.

The current protected authority has no TinyCC 0.9.27 rebuilt against musl. Therefore, a musl-linked compiler is not yet bound into this transition manifest.

A complete provider run still needs all of these mechanisms and artifacts:

1. Rebuild TinyCC 0.9.27 against the reduced musl outputs, then bind and smoke the musl-linked compiler under exact protected authority.
2. Continue through the remaining conventional GNU tools, musl, GCC, binutils, and completed native-provider stages without ambient execution.
3. Retain per-stage reports for every later exact executable, source stage, input, output, and BLAKE3 value.
4. Preserve a complete protected-exec audit through normalized-provider runtime validation.
5. Independently validate and create-new publish the provider and complete receipt.

No protected musl-linked TinyCC output identity or executable authorization exists in the current graph. Replacing only the scaffold JSON would create false evidence. Reusing the existing full-source provider would relabel host-assisted effects as StageX lineage.

## Owner

Mantle StageX lineage-provider implementation.

## Next action

Build and bind TinyCC 0.9.27 against the reduced musl outputs. Then continue the protected runner with bounded Rust orchestration and no host shell or sandbox tools.

I1 through V3 remain unchecked. The fixed-point and parity-promotion successor changes remain blocked by this change.

## Lifecycle validation

Pueue tasks `1287` through `1290` validated this exact blocker packet.

- Cairn validation: `valid: true`, `issues: []`, 35 specs validated.
- Proposal gate: `PASS`, receipt hash `5bd3935488d3f8b6ec3e5ae3133811dc5f4d56370dcd3a36b526b6a64d8a53a7`.
- Design gate: `PASS`, receipt hash `9a6d746e3fcee4bf3958d57462b83dfe6dfc66112677c2283dbab62858536504`.
- Tasks gate: `PASS`, eight tasks remain unchecked, receipt hash `c76a9f464b17102361d8428a04d9790dd093c5a55aad0ffa651287fb8bbd741d`.

The stderr files are empty. Passing scaffold gates validates the change packet. It does not establish StageX provider completion.
