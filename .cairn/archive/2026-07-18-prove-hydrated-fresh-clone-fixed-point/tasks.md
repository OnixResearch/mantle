## Phase 1: Source authority and fail-closed execution

- [x] [serial] I1 Add a distinct full-proof source profile and pure closure/profile validators while preserving the exact three-record fresh-clone hydration contract. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: `fresh-clone-fixed-point` assembled 15 records while focused tests prove `fresh-clone-inputs` remains exactly three records.
- [x] [serial] I2 Add explicit connected materialization of missing evaluated fixed-fetch records through Mantle's fixed-output verifier, with bounded payload and deterministic failure behavior. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: task `84` exported 12 ready records / 600,587,433 payload bytes; compressed archives replay the bounded extractor and recursive verifier, with deterministic ≤64 MiB chunks.
- [x] [serial] I3 Add fetch-service/pipeline/self-build enforcement so unmatched builtin fetches fail before network acquisition and successful reports bind source-state identity and zero live fetches. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: positive/negative fetch-service tests pass; final stage0 and stage2 each report `require-override`, 12 overrides, the same source-state BLAKE3, and zero live fetches.

## Phase 2: Proof harness and contracts

- [x] [serial] I4 Extend the proof helper/test to seed fresh source-only state for both stages, enforce offline source policy, and preserve contracted hydration/source-policy evidence. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: task `406` executed the complete ignored proof and preserved hydration, stage diagnostics, protected-exec evidence, binaries, summary, and contracted fixed-point report.
- [x] [serial] I5 Add ADR 0032, operator/proof documentation, schemas, fixtures, and machine-contract coverage with explicit claim boundaries. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: ADR 0032 and operator docs describe the workflow and non-claims; schema generation/check and positive/negative fixtures pass at 20 contracted / 49 classified surfaces.

## Phase 3: Verification and lifecycle

- [x] [serial] V1 Preserve pre-change focused source-bundle, self-build, and proof-harness baseline results. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: `evidence/baseline.md` records 55 source-bundle tests, 171 self-build tests, and 51 non-expensive proof-harness tests before core changes.
- [x] [serial] V2 Add positive closure/materialization/offline-stage tests and negative missing, duplicate, stale, tampered, URL/revision mismatch, unsupported kind, and attempted live-fetch tests. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: final focused suites pass with 22 fetcher, 64 source-bundle, 172 self-build, 4 contracted-report, and 53 non-expensive proof-harness tests; details are in `evidence/validation.md`.
- [x] [serial] V3 Produce one real full-proof bundle, hydrate a Git clone with initially empty Cargo/source/proof state, run locked offline Cargo metadata, and execute the complete stage0 → stage1 → stage2 proof with live source acquisition forbidden. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: clone commit `9106845c`; hydration imported 15/new 0/existing records; empty-cache metadata passed; task `406` passed in 3,315.30 seconds with matching binary BLAKE3 `162b38af...f3769d`.
- [x] [serial] V4 Run Rustfmt, strict first-party Clippy, Tiger Style, dependency policy, machine contracts/docs, first-party quality, Nix evaluation, Cairn gates, and Tracey coverage. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: Rustfmt/Clippy/tests task `385`, Tiger task `377`, dependency task `378`, machine-contract tasks `380`/`407`, and Nix evaluation task `384` pass; Cairn task `23` passed validation plus proposal/design/tasks gates, and Tracey task `29` reported `145/145`.
- [x] [serial] V5 Commit implementation before archiving; sync and inspect the accepted requirement; archive exact post-state receipts; commit lifecycle closeout, and push `main` only when explicitly authorized. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
  - Evidence: final implementation commit `9106845c`; sync receipt `448b669f84b7f0b7acaabda4741564483f4f6aa0242ee5f3a61e41861b75139f`; accepted requirement lines 171–208 were inspected intact; archive/post-state receipts are appended to archived evidence before the lifecycle commit. No push is authorized in this session.
