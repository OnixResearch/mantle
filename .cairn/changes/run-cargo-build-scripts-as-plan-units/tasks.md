# Tasks: Run Cargo build scripts as plan units

Task completion is recorded individually. Partial evidence does not accept
build-script support.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Record the baseline in an isolated worktree: unit graphs and `rust-plan` build-script receipts for `proc-macro2`, `serde` with derive, `libc`, and one `-sys` fixture, plus the current lane blockers. r[mantle.rust_unit_plan.build_scripts.run_units]
  - Bounded retrospective recovery: `evidence/source-owned-da00-baseline-2026-10-27.json` and `evidence/source-owned-da00-baseline-isolated-confirmation-2026-10-27.json` bind unchanged original da00 `da00f58425740adea559ef926c9dfa97b2cb8240`, its source-owned locked Nix/Crane cohort and binary BLAKE3 `436ebb09e23d04d8049c212d404843d94337f8cb2a15714ebf79873ed6f74a70`, to four original `rust-plan` planning receipts and four raw Cargo graphs (5/16/4/4 units; custom-build compile/run modes 1/4/1/1 each) executed literally from `/home/brittonr/scratch/mantle-cargo-units-origin-baseline`. Confirmation request BLAKE3 `138780d35d28b49a707236156764ded118d75f51869c96ce09f672515eb7e5ae`, 2026-10-27T03:28:11.138087-04:00–03:28:11.742465-04:00, retains eight exit-0 streams, driver exit 0 and `source_unchanged: true` for six selected identity files. Registry native planning remains blocked on missing declared vendor roots; the path-only sys fixture establishes bounded topology only. Constructed fixture provenance and the current serde lock's difference from the historical 2026-10-14 lock remain explicit; no exact old-lock equivalence or build-script execution is claimed. `evidence/historical-da00-baseline-2026-10-14.md` retains ambient Casita E0599 and privately patched CLI non-acceptance, and the initial source-owned external-supply failure is preserved. Only the authored T1.1 baseline-record requirement is closed; no run-unit semantic, other-task, final-source gate or lifecycle acceptance follows.
- [x] [serial] T1.2 Define the supported directive set, propagation rules, `flags` file formats, the target-facts format, and the native-input Nickel contract with role markers. r[mantle.rust_unit_plan.build_scripts.directive_propagation] r[mantle.rust_unit_plan.build_scripts.native_inputs]

## Phase 2: Core and lowering

- [x] [serial] T2.1 Move directive parsing into the pure core with the enumerated directive set and typed failures. r[mantle.rust_unit_plan.build_scripts.directive_propagation]
- [x] [serial] T2.2 Lower target-description units and build-script execution units with declared environments and `flags` references in dependents. r[mantle.rust_unit_plan.build_scripts.target_facts] r[mantle.rust_unit_plan.build_scripts.run_units]
- [x] [serial] T2.3 Resolve native-input roles in the producer and add them as unit inputs. r[mantle.rust_unit_plan.build_scripts.native_inputs]
- [x] [parallel] T2.4 Add core and lowering fixtures: directive parsing, transitive link propagation, `links` metadata, and shared target units. r[mantle.rust_unit_plan.build_scripts.directive_propagation] r[mantle.rust_unit_plan.build_scripts.target_facts]
- [x] [parallel] T2.5 Add negative fixtures: an unknown directive, an `error` directive, an undeclared role marker, and duplicate `links` keys. r[mantle.rust_unit_plan.build_scripts.directive_propagation] r[mantle.rust_unit_plan.build_scripts.native_inputs]
  - Evidence for T1.2 and T2.1–T2.5: `docs/operator-workflows.md`, `design.md`, `evidence/local-focused-package-2026-10-13.log` (24/24), and `evidence/remote-focused-package-tests-2026-10-13.log` (25/25, including the duplicate-`links` regression).

## Phase 3: Helper and sandbox

- [x] [serial] T3.1 Add target-facts and run-build-script modes to the unit helper, writing typed `flags` outputs. r[mantle.rust_unit_plan.build_scripts.run_units]
- [x] [serial] T3.2 Consume `flags` in dependent compilations through argument and environment files. r[mantle.rust_unit_plan.build_scripts.directive_propagation]
  - Evidence: `evidence/local-focused-package-2026-10-13.log` covers helper directive/flags consumption; `evidence/current-client-real-sandbox-2026-10-13.txt` covers only the isolated signed target-facts execution, not a build-script application.
- [x] [parallel] T3.3 Add real sandbox negative controls: a host-path marker and TCP endpoint both reachable outside the sandbox but not inside it (use the same signed TCP-capable executable on both sides), a nonzero script exit, and a missing native role while its library remains physically present in the client. r[mantle.rust_unit_plan.build_scripts.sandbox_boundary]
  - Bounded evidence: `evidence/current-client-real-sandbox-2026-10-13.txt` retains real signed host-file and TCP controls. `evidence/current-client-singleton-negatives-2026-10-13.txt` records the same-client signed native archive physically verified while a one-unit producer failed with `native-input-role-undeclared`, plus a separately signed actual compiled Rust build script run through `run-build-script` in real bubblewrap that failed with `build-script-nonzero-exit: exit status: 7`. The earlier full-producer missing-role timeout remains unaccepted historical evidence (`evidence/current-client-missing-role-bound-2026-10-13.txt`). These four isolated negatives do not establish a full application or selective native-provider rerun.

## Phase 4: Verification and documentation

- [x] [serial] T4.1 Admit each signed tool, source, vendor, runtime, and current planner root into one preserved client. Require nonzero selection, zero pull rejection counters, trusted physical `store verify` for every prior root after interrupted-import repair, and a real no-substitute bubblewrap Rust app result containing `proc-macro2 serde-derive libc native-sys = 42`. r[mantle.rust_unit_plan.build_scripts.run_units]
  - Bounded acceptance: `evidence/t41-signed-app-2026-10-14.md`, `evidence/t41-signed-app-2026-10-14-manifest.json`, and the exact raw receipts in `evidence/t41-signed-app-2026-10-14/` retain seven nonzero selected signed pulls with zero rejection counters, all 59 prior trusted physical root verifications after nightly repair, separate repaired planner/cfgfix helper/config/lean-toolchain/native-provider verification, and the first real same-client no-substitute bubblewrap app result `cargo-build-script-fixture: proc-macro2 serde-derive libc native-sys = 42`. The app report records 3 succeeded, 1 built, 2 cached, 0 failed; corrected checker/trust task 1055 accepted its already-built outputs without an app rebuild, with four 1/1 trusted output verifications. The 1047 logical/physical-binding and 1051 post-build goal-role checker failures remain preserved. Acceptance is bounded to the frozen signed cohort and 34 unique admitted units/14 slices; it does not accept T4.2 selective reuse, T4.3 matrix support, a global current-source claim, or archive.
  - Prose/evidence checks: `evidence/t41-cairn-gates-2026-10-14.txt` (BLAKE3 `cc88478253dbaea081e15f51207e2085fe270572c259b4b8a137f6e8478f9a8a`) retains actual exit-0 `cairn validate --root .` (`valid: true`) and proposal/design/tasks gates (`PASS`) after the T4.1 receipt capture, before this reference was added. These read-only structural verdicts do not prove artifact truth or semantic task completion, accept T4.2/T4.3, or replace the owner's final lifecycle checks after future edits.
- [x] [serial] T4.2 Build the declared native library in that same client; change only its Mantle-built provider root, then compare unit identities and build outcomes to prove dependent script and app units rerun while independent units are reused. Run T3.3 negatives before accepting the matrix. r[mantle.rust_unit_plan.build_scripts.native_inputs]
  - Bounded acceptance: `evidence/t42-selective-provider-2026-10-15.md`, its exact-byte manifest, and 45 raw files (738,221 bytes) in `evidence/t42-selective-provider-2026-10-15/` retain original 34 actual builds, exactly 31 positive independent admitted-cache events and 3 affected successful builds, changed actual input-addressed output paths/canonical app derivation despite stable nominal unit labels, full 34-unit/14-slice admission, and the executed variant app42 with named trusted physical receipts. Existing tasks 1061–1065 settled Success; strict comparison completed exit 0 at 2026-10-15T00:51:53.065279Z without replaying accepted builds. The prior checker and failed/passing smoke sources/logs are preserved; the corrected checker rejects seven invalid-evidence cases. Frozen T4.1 signed-input/1047/1051 history and T3.3 sandbox negatives remain referenced, not duplicated or broadened. T4.3 matrix acceptance remains separate.
- [x] [serial] T4.3 Update matrix rows only after named fixture outputs, trusted import/verify receipts, sandbox negatives, and selective rerun evidence are retained; extend the drift rail. r[mantle.rust_unit_plan.build_scripts.matrix_evidence]
  - Bounded acceptance: `evidence/t43-verification-2026-10-15.txt`, its exact-byte manifest and raw/encoded receipt directory retain the real production matrix evaluation/contract rejection, 3/3 matrix/no-Cargo rail, 17/17 inventory and 2/2 no-oracle CLI cases, plus the final helper package 27/27 including executed generated-source/cfg/env consumer `generated-script=84`. Matrix rows bind the retained T4.1/T4.2 signed app42, trusted physical verification, sandbox negatives and selective-provider receipts; the declared-prebuilt-C static-link row is supported, not the broader links/pkg-config/native-C group. Final task 1069 used refreshed actual formatted helper subjects and exact staged real compile-time inputs, not the historical 1068 subject snapshot. Whole-repository format remains a separate failed gate.
- [x] [serial] T4.4 Document build-script support, directives, native declarations, and non-claims in the lane documentation. r[mantle.rust_unit_plan.build_scripts.sandbox_boundary]
  - Evidence: `docs/operator-workflows.md`, the root/example READMEs/catalog, and `docs/release-notes/cargo-build-script-units-2026-10-15.md` describe the supported directives, typed `flags`, declared native roles, retained signed T4.1/T4.2 app42/selective receipts and no-FUSE sandbox controls. They distinguish the supported declared-prebuilt-C static-link fixture from the still-blocked full links/pkg-config/native-C group; helper links propagation is not pkg-config execution, and ordinary rails do not replay signed proofs.
  - Reconciliation note: the independent remote-main reviewed links/pkg-config replay against separately Mantle-built libbz2 is also retained in the matrix and lane docs. It is distinct from the app42/provider-selective receipts above; neither establishes native C compilation inside `build.rs` or validates the reconciled source. Existing task acceptance and pending T4.5/T4.6 states are unchanged; ordinary merge verification does not replay either historical campaign.
- [x] [serial] T4.5 Run focused core, lowering, helper, and lane tests, `rust_compatibility_rail`, strict Clippy for touched first-party packages, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.rust_unit_plan.build_scripts.run_units]
  - Evidence: `evidence/remote-focused-package-tests-2026-10-13.log` (25/25), `evidence/remote-focused-gc-tests-2026-10-13.log` (2/2 journal recovery tests), `evidence/remote-rust-compatibility-rail-bwrap-2026-10-13.log` (6/6 with no sandbox skip), `evidence/remote-strict-clippy-unit-pass-2026-10-13.log` and `evidence/remote-strict-clippy-store-pass-2026-10-13.log` (`-D warnings`), and `evidence/cairn-gates-2026-10-13.txt` (validation and all three gates). The first rail's `evidence/remote-rust-compatibility-rail-no-bwrap-2026-10-13.log` records a skipped sandbox check; it does not replace the non-skipped run.
  - Fresh focused 2026-10-15 verification is recorded in `evidence/t43-verification-2026-10-15.txt` and its manifest. Actual final task 1069 passed feature formatting, whole helper package formatting, all 27 helper/package tests (including executed `generated-script=84`), helper `--all-targets` strict Clippy and mantle `--all-targets` strict Clippy with the established explicit GCC linker environment. The 3/3 production matrix/no-Cargo rail, 17/17 inventory and 2/2 no-oracle CLI passes from task 1067 remain preserved and were not rerun. Its failed linker-environment launch, task 1068's canonical format/initial missing-real-input failures, and the exact fifteen-input mirror repair are also retained, not relabeled passing. Final Cairn validation and proposal/design/tasks transcripts are captured after these final prose/task edits. T4.5 stays open because canonical first-party formatting still has a preserved exit-1 failure across 26 out-of-cutover paths; accepted focused checks do not make the whole-repository gate green.
- [ ] [serial] T4.6 PRE-ARCHIVE readiness: synchronize accepted specs and verify readiness through the isolated branch workflow, retaining completion evidence without performing archive as part of this task. r[mantle.rust_unit_plan.build_scripts.matrix_evidence]

## Mandatory lifecycle actions outside task checkboxes

Actual Cairn archive and captured post-archive validation remain mandatory, separate from T4.6. Archive may occur only after all other obligations, required gates, approved isolated source snapshot, and implementation commit hold; this boundary repair does not satisfy or waive them. T4.6 remains unchecked, and no acceptance or archival is recorded here. Approval scope: `evidence/pre-archive-readiness-approval-2026-10-16.md`.

## Before-fix no-Cargo boundary observation (2026-10-27)

`evidence/no-cargo-boundary-observation-2026-10-27.json` records the same
compiled probe outside and inside the current helper's real bwrap subprocess.
With the declared full toolchain root and no native PATH entries, Cargo ran
successfully by both PATH and its absolute store path; the host marker was
hidden inside the sandbox. This is a negative boundary observation, not host
escape or requirement satisfaction. The separate actual Mantle import timed
out before sandbox execution and is not relabeled as a pipeline result.
The no-Cargo exposure must be resolved and negatively exercised before T4.6
or whole-change acceptance. Historical planning-shim and application receipts
remain scoped to their original subjects; none establishes this missing bound.
T4.6 stays unchecked. No accepted spec, archive or source approval is recorded.

## Corrected execution boundary evidence (2026-10-27)

`evidence/compiler-only-boundary-2026-10-27.json` retains Pueue 78's
current-helper bwrap execution: the same compiled script can execute Cargo
outside, but neither PATH nor absolute Cargo launches inside. Real compiler
cfg and pkg-config queries succeed; a separately compiled native archive is
linked through the emitted flags and the consumer prints
`no-cargo-native-answer=42`.

`evidence/frozen-mantle-no-cargo-singleton-2026-10-27.json` retains Pueue
79's actual same-client, no-substitute Mantle build using verified frozen
CLI/helper copies and a fresh signed-input singleton identity: 1 built,
0 cached, 0 failed. Its script executes real compiler cfg while both Cargo
routes and the host marker remain unavailable. Pueue 73's earlier shared-CLI
drift failure remains failed, not promoted to success.

These observations resolve the exposed Cargo execution path for the declared
compiler closure. They do not replay the full dynamic producer/application or
Mantle-built native-provider/selective-rerun campaign. T4.6 remains unchecked;
required gates, approved source/snapshot custody and the separate lifecycle
actions above are not replaced by these bounded runtime results.

