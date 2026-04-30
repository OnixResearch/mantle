## Evidence format

Every task records evidence under `openspec/changes/live-part-musl-1-1-24-tcc-musl/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `musl 1.1.24 and musl_target` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 2m (started: 2026-04-28T23:57:35Z → completed: 2026-04-28T23:57:43Z) [covers=bootstrap.part.musl.1.1.24.tcc.musl] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/musl-1.1.24-tcc-musl.ncl` against the upstream part and record intentional Crunch deviations. ✅ 2m (started: 2026-04-28T23:57:43Z → completed: 2026-04-28T23:57:47Z) [covers=bootstrap.part.musl.1.1.24.tcc.musl] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, rebuild order matches but source removals, flags, and crt object contract need I3/V3 proof.
- [x] I3 Fix `bootstrap/musl-1.1.24-tcc-musl.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 5m (started: 2026-04-30T22:27:00Z → completed: 2026-04-30T22:32:00Z) [covers=bootstrap.part.musl.1.1.24.tcc.musl] [evidence=evidence/I3-fix.md]

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/musl-1.1.24-tcc-musl.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:57:31Z → completed: 2026-04-28T23:57:35Z) [covers=bootstrap.part.musl.1.1.24.tcc.musl] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 1 fetch blocks, 0 issues`.
- [x] V2 Deferred to openspec change: `live-part-musl-1-1-24-tcc-musl-runtime-validation` ✅ 0m (deferred; blocked by prerequisite make/tcc/musl runtime validation) [covers=bootstrap.part.musl.1.1.24.tcc.musl] [evidence=evidence/V2-build.md]
- [x] V3 Deferred to openspec change: `live-part-musl-1-1-24-tcc-musl-runtime-validation` ✅ 0m (deferred; depends on V2 output) [covers=bootstrap.part.musl.1.1.24.tcc.musl] [evidence=evidence/V3-smoke.md]
- [x] V4 Deferred to openspec change: `live-part-musl-1-1-24-tcc-musl-runtime-validation` ✅ 0m (deferred; depends on V2 transcript) [covers=bootstrap.part.musl.1.1.24.tcc.musl] [evidence=evidence/V4-host-leakage.md]
- [x] V5 Run `openspec validate live-part-musl-1-1-24-tcc-musl` after evidence is recorded. ✅ 1m (warnings only: delta-spec heading IDs) [covers=bootstrap.part.musl.1.1.24.tcc.musl] [evidence=evidence/V5-openspec-validate.md]
