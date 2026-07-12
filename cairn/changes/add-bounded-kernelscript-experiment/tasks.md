## Phase 1: Profile and compiler materialization

- [ ] [serial] Define the typed beta experiment profile, source/compiler/toolchain/kernel refs, selected output classes, expected generated files, named bounds, and non-claims. r[kernelscript_experiment.profile]
- [ ] [serial] Add pinned fixed-output KernelScript source and locked OCaml/dune/opam compiler derivations with offline build/codegen behavior and BLAKE3 identities. r[kernelscript_experiment.compiler]
- [ ] [parallel] Add compiler/source drift, missing dependency closure, network attempt, unsupported target, and bound-exceeded negative fixtures. r[kernelscript_experiment.compiler] r[kernelscript_experiment.verification]

## Phase 2: Code generation and explicit build planning

- [ ] [serial] Implement the separate codegen derivation and pure generated-project manifest parser/classifier with exact expected-file checks. r[kernelscript_experiment.codegen]
- [ ] [serial] Implement a Mantle-owned pure compilation planner for allowlisted userspace, eBPF, optional module, and test steps; retain but never execute generated Makefiles. r[kernelscript_experiment.artifacts]
- [ ] [serial] Implement target kernel-build/BTF/header/config/architecture admission and fail closed on absent, ambient-only, or mismatched inputs. r[kernelscript_experiment.kernel_inputs]
- [ ] [serial] Build and statically inspect each selected output class independently with exact member and receipt identities. r[kernelscript_experiment.artifacts]

## Phase 3: Candidate handoff and evidence

- [ ] [depends:mantle.add-onix-kernel-bundle-oci-projections] Emit frontend-neutral experimental ModulePack/BPF Pack candidate projections with target bindings and no deployability claim. r[kernelscript_experiment.handoff]
- [ ] [serial] Add codegen/build receipts binding source, compiler closure, target inputs, plans, generated manifest, outputs, inspections, blockers, and non-claims with BLAKE3. r[kernelscript_experiment.evidence]
- [ ] [parallel] Add positive fixtures for a small userspace+probe program and one separately gated kfunc/module case under an exact kernel cohort. r[kernelscript_experiment.verification]
- [ ] [parallel] Add negative fixtures for generated-file drift/extra files, forbidden Makefile execution, unknown command/path, missing BTF/headers, kernel mismatch, BPF/module compile failure, malformed ELF/BTF metadata, stale output, receipt leak, and production overclaim. r[kernelscript_experiment.verification]

## Phase 4: Validation and closeout

- [ ] [parallel] Document the pinned cohort, beta status, reproducible command path, generated-source review, pack handoff, ChaosControl dependency, upgrade procedure, and non-claims. r[kernelscript_experiment.profile] r[kernelscript_experiment.evidence]
- [ ] [serial] Run focused profile, compiler, codegen, planner, build, static-inspection, schema/receipt, formatting, clippy, and dependency-audit checks. r[kernelscript_experiment.verification]
- [ ] [serial] Run Cairn validation and proposal/design/tasks gates; sync and archive only with positive/negative evidence and without promoting experimental artifacts. r[kernelscript_experiment.verification]
