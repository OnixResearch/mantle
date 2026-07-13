## Phase 1: Profile and compiler materialization

- [x] [serial] Define the typed beta experiment profile, source/compiler/toolchain/kernel refs, selected output classes, expected generated files, named bounds, and non-claims. r[kernelscript_experiment.profile]
- [x] [serial] Add pinned fixed-output KernelScript source and locked OCaml/dune/opam compiler derivations with offline build/codegen behavior and BLAKE3 identities. r[kernelscript_experiment.compiler]
  - Evidence: the official `v0.1.2` archive is fixed by upstream SHA-256 and measured BLAKE3; the compiler and codegen run offline from the locked nixpkgs OCaml/dune/menhir/library closure with no ambient opam or upstream binary. Compiler-binary and closure path-set BLAKE3 identities are recorded in `evidence/probe-production-evidence.md`. Core admission remains a separate unchecked boundary below.
- [x] [parallel] Add compiler/source drift, missing dependency closure, network attempt, unsupported target, and bound-exceeded negative fixtures. r[kernelscript_experiment.compiler] r[kernelscript_experiment.verification]

## Phase 2: Code generation and explicit build planning

- [ ] [serial] Implement the separate codegen derivation and pure generated-project manifest parser/classifier with exact expected-file checks. r[kernelscript_experiment.codegen]
  - Partial: the pure exact classifier and offline codegen-plan identity exist, and the pinned Nix route executes compiler codegen for both reviewed fixtures. The production shell still duplicates exact-shape accounting instead of invoking the pure classifier, so this task remains unchecked. See `evidence/probe-production-evidence.md`.
- [x] [serial] Implement a Mantle-owned pure compilation planner for allowlisted userspace, eBPF, optional module, and test steps; retain but never execute generated Makefiles. r[kernelscript_experiment.artifacts]
- [x] [serial] Implement target kernel-build/BTF/header/config/architecture admission and fail closed on absent, ambient-only, or mismatched inputs. r[kernelscript_experiment.kernel_inputs]
- [ ] [serial] Build and statically inspect each selected output class independently with exact member and receipt identities. r[kernelscript_experiment.artifacts]
  - Partial: independent bounded ELF/BTF/module shape inspection exists in the core. The pinned Nix route builds and externally inspects the probe eBPF object, skeleton, and loader, but it does not invoke the core inspection/receipt path and does not build the module/test classes. See `evidence/probe-production-evidence.md`.

## Phase 3: Candidate handoff and evidence

- [x] [depends:mantle.add-onix-kernel-bundle-oci-projections] Emit frontend-neutral experimental ModulePack/BPF Pack candidate projections with target bindings and no deployability claim. r[kernelscript_experiment.handoff]
- [x] [serial] Add codegen/build receipts binding source, compiler closure, target inputs, plans, generated manifest, outputs, inspections, blockers, and non-claims with BLAKE3. r[kernelscript_experiment.evidence]
- [ ] [parallel] Add positive fixtures for a small userspace+probe program and one separately gated kfunc/module case under an exact kernel cohort. r[kernelscript_experiment.verification]
  - Partial: reviewed `.ks` fixtures now generate under the locked Linux `6.18.20` cohort, and the probe object/loader pass structural plus exact-kernel verifier/load/attach/detach VM gates. The checked route does not build or VM-load the module/kfunc case, and reported OnixOS Git identifiers are metadata rather than materialized authority. See `evidence/probe-production-evidence.md`.
- [x] [parallel] Add negative fixtures for generated-file drift/extra files, forbidden Makefile execution, unknown command/path, missing BTF/headers, kernel mismatch, BPF/module compile failure, malformed ELF/BTF metadata, stale output, receipt leak, and production overclaim. r[kernelscript_experiment.verification]

## Phase 4: Validation and closeout

- [x] [parallel] Document the pinned cohort, beta status, reproducible command path, generated-source review, pack handoff, ChaosControl dependency, upgrade procedure, and non-claims. r[kernelscript_experiment.profile] r[kernelscript_experiment.evidence]
- [ ] [serial] Run focused profile, compiler, codegen, planner, build, static-inspection, schema/receipt, formatting, clippy, and dependency-audit checks. r[kernelscript_experiment.verification]
  - Partial: pure-core, Nickel integration, wasm no-std, formatting, and core clippy checks pass. The pinned compiler/codegen/probe structural derivation and exact-kernel probe VM gate also pass. Full validation remains blocked on wiring production execution through the pure core and on checked module/kfunc build/load evidence. The Nickel formatter is unavailable in the current dev shell. Direct `cargo-deny check` remains workspace-red on pre-existing `RUSTSEC-2026-0204` (`crossbeam-epoch 0.9.18`) and an unallowlisted `winx 0.36.4` license expression; neither dependency is in this core crate's locked dependency tree. See `evidence/probe-production-evidence.md`.
- [ ] [serial] Run Cairn validation and proposal/design/tasks gates; sync and archive only with positive/negative evidence and without promoting experimental artifacts. r[kernelscript_experiment.verification]
  - Cairn validation and the tasks gate were refreshed after the probe evidence update and pass, but this closeout task remains unchecked and sync/archive are forbidden for this session. Core-backed production admission and checked module/kfunc build/load evidence remain absent.
