# Tasks: Repair live-bootstrap archive status

## Audit and immediate repair

- [x] I1 Audit archived `live-bootstrap-seed-chain` and `live-bootstrap-intermediate-tools` tasks for false completion signals. [covers=Full-source bootstrap claim requires evidence]
  - Evidence: both archives mark deferred validation as `[x]`; `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, and `bootstrap/binutils-full.ncl` still print placeholder errors.
- [x] I2 Restore `bootstrap/seed-legacy.ncl` to the concrete reduced musl.cc provider. [covers=Legacy seed as development fast-path]
  - Evidence: `rg 'import "seed-legacy.ncl"' bootstrap/seed-legacy.ncl` returns no self-import after restore.
- [x] I3 Deferred full pre-GCC and compiler-transition implementation to openspec change `live-bootstrap-source-chain`. ✅ 0m (deferred) [covers=Full-source bootstrap claim requires evidence]
  - Evidence: `openspec/changes/live-bootstrap-source-chain/tasks.md` splits the former broad work into stage-sized I1-I11 tasks.
- [x] I4 Deferred full-chain validation and final source-built provider proof to openspec change `live-bootstrap-source-chain`. ✅ 0m (deferred) [covers=Full-source bootstrap claim requires evidence]
  - Evidence: `openspec/changes/live-bootstrap-source-chain/tasks.md` carries V1-V7 validation tasks for stage builds, final provider proof, evidence fields, source-pin audit, and status rejection.

## Validation

- [x] V1 Validate the restored legacy seed fast path without source-chain recursion. ✅ 12m [covers=Legacy seed as development fast-path] [evidence=evidence/V1-legacy-seed.md]
  - Commands: `rg 'import "seed-legacy.ncl"' bootstrap/seed-legacy.ncl`; `cargo test -p crunch --test bootstrap_eval -- --nocapture`.
- [x] V2 Verify placeholder, deferred, and archived partial-scaffolding evidence cannot satisfy full-source bootstrap completion status. ✅ 0m (artifact audit) [covers=Full-source bootstrap claim requires evidence] [evidence=evidence/V2-invalid-evidence-rejection.md]
  - Evidence: `evidence/V2-invalid-evidence-rejection.md` records placeholder markers, deferred archive lines, and successor-task transfer; the delta spec forbids prerequisite-only checks, placeholder derivations, deferred tasks, and archived partial-scaffolding changes as claim evidence.
- [x] V3 Deferred final source-built provider proof to openspec change `live-bootstrap-source-chain`. ✅ 0m (deferred) [covers=Full-source bootstrap claim requires evidence] [evidence=evidence/V3-source-proof-deferred.md]
  - Evidence: `evidence/V3-source-proof-deferred.md` names provider kind, manifest digest, provider output digest, proof bundle digest, stage1/stage2 equality, and docs/trust-root separation as required future proof fields.
- [x] V4 Run OpenSpec validation and gates before archive. ✅ 2m [covers=Full-source bootstrap claim requires evidence,Legacy seed as development fast-path] [evidence=evidence/V4-openspec-gates.md]
  - Commands: `openspec validate repair-live-bootstrap-archive-status`; `openspec_gate stage=proposal change=repair-live-bootstrap-archive-status`; `openspec_gate stage=design change=repair-live-bootstrap-archive-status`; `openspec_gate stage=tasks change=repair-live-bootstrap-archive-status`.
