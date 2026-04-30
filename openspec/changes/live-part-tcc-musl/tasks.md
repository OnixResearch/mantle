## Evidence format

Every task records evidence under `openspec/changes/live-part-tcc-musl/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `musl 1.1.24 and musl_target` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 2m (started: 2026-04-28T23:56:12Z → completed: 2026-04-28T23:56:32Z) [covers=bootstrap.part.tcc.musl] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/tcc-musl.ncl` against the upstream part and record intentional Crunch deviations. ✅ 2m (started: 2026-04-28T23:56:32Z → completed: 2026-04-28T23:56:37Z) [covers=bootstrap.part.tcc.musl] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, musl-linked compiler shape matches but self-host loop and `libtcc1.a` contract need I3/V3 proof.
- [x] I3 Fix `bootstrap/tcc-musl.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 5m (started: 2026-04-30T22:40:00Z → completed: 2026-04-30T22:45:00Z) [covers=bootstrap.part.tcc.musl] [evidence=evidence/I3-fix.md]

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tcc-musl.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:56:22Z → completed: 2026-04-28T23:56:26Z) [covers=bootstrap.part.tcc.musl] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 1 fetch blocks, 0 issues`.
- [x] V2 Deferred to openspec change: `live-part-tcc-musl-runtime-validation` ✅ 0m (deferred; blocked by prerequisite make/tcc/musl runtime validation) [covers=bootstrap.part.tcc.musl] [evidence=evidence/V2-build.md]
- [x] V3 Deferred to openspec change: `live-part-tcc-musl-runtime-validation` ✅ 0m (deferred; depends on V2 output) [covers=bootstrap.part.tcc.musl] [evidence=evidence/V3-smoke.md]
- [x] V4 Deferred to openspec change: `live-part-tcc-musl-runtime-validation` ✅ 0m (deferred; depends on V2 transcript) [covers=bootstrap.part.tcc.musl] [evidence=evidence/V4-host-leakage.md]
- [x] V5 Run `openspec validate live-part-tcc-musl` after evidence is recorded. ✅ 1m (warnings only: delta-spec heading IDs) [covers=bootstrap.part.tcc.musl] [evidence=evidence/V5-openspec-validate.md]
