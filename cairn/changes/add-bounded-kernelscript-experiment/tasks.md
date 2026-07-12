## Phase 1: Profile and compiler materialization

- [x] [serial] Define the typed beta experiment profile, source/compiler/toolchain/kernel refs, selected output classes, expected generated files, named bounds, and non-claims. r[kernelscript_experiment.profile]
- [ ] [serial] Add pinned fixed-output KernelScript source and locked OCaml/dune/opam compiler derivations with offline build/codegen behavior and BLAKE3 identities. r[kernelscript_experiment.compiler]
  - Partial: the official `v0.1.2` source archive is pinned by upstream SHA-256 and measured BLAKE3. Upstream provides no immutable opam dependency lock, so compiler materialization and compiler-success claims remain blocked.
- [x] [parallel] Add compiler/source drift, missing dependency closure, network attempt, unsupported target, and bound-exceeded negative fixtures. r[kernelscript_experiment.compiler] r[kernelscript_experiment.verification]

## Phase 2: Code generation and explicit build planning

- [ ] [serial] Implement the separate codegen derivation and pure generated-project manifest parser/classifier with exact expected-file checks. r[kernelscript_experiment.codegen]
  - Partial: the pure exact classifier and offline codegen-plan identity exist. The derivation and compiler execution remain blocked on the missing immutable compiler dependency closure.
- [x] [serial] Implement a Mantle-owned pure compilation planner for allowlisted userspace, eBPF, optional module, and test steps; retain but never execute generated Makefiles. r[kernelscript_experiment.artifacts]
- [x] [serial] Implement target kernel-build/BTF/header/config/architecture admission and fail closed on absent, ambient-only, or mismatched inputs. r[kernelscript_experiment.kernel_inputs]
- [ ] [serial] Build and statically inspect each selected output class independently with exact member and receipt identities. r[kernelscript_experiment.artifacts]
  - Partial: independent bounded ELF/BTF/module shape inspection is implemented over supplied bytes. No authoritative compiler/kernel cohort is available, so no artifact was built and no real output-inspection success is claimed.

## Phase 3: Candidate handoff and evidence

- [x] [depends:mantle.add-onix-kernel-bundle-oci-projections] Emit frontend-neutral experimental ModulePack/BPF Pack candidate projections with target bindings and no deployability claim. r[kernelscript_experiment.handoff]
- [x] [serial] Add codegen/build receipts binding source, compiler closure, target inputs, plans, generated manifest, outputs, inspections, blockers, and non-claims with BLAKE3. r[kernelscript_experiment.evidence]
- [ ] [parallel] Add positive fixtures for a small userspace+probe program and one separately gated kfunc/module case under an exact kernel cohort. r[kernelscript_experiment.verification]
  - Partial: planning-only `.ks`, generated userspace/eBPF, and generated kfunc/module fixtures exist. Their target identities are explicitly synthetic, not an authoritative exact Onix kernel cohort.
- [x] [parallel] Add negative fixtures for generated-file drift/extra files, forbidden Makefile execution, unknown command/path, missing BTF/headers, kernel mismatch, BPF/module compile failure, malformed ELF/BTF metadata, stale output, receipt leak, and production overclaim. r[kernelscript_experiment.verification]

## Phase 4: Validation and closeout

- [x] [parallel] Document the pinned cohort, beta status, reproducible command path, generated-source review, pack handoff, ChaosControl dependency, upgrade procedure, and non-claims. r[kernelscript_experiment.profile] r[kernelscript_experiment.evidence]
- [ ] [serial] Run focused profile, compiler, codegen, planner, build, static-inspection, schema/receipt, formatting, clippy, and dependency-audit checks. r[kernelscript_experiment.verification]
  - Partial: pure-core, Nickel integration, wasm no-std, formatting, and core clippy checks pass. Compiler/codegen/build checks cannot run honestly without the immutable compiler closure and authoritative target cohort; the Nickel formatter is unavailable in the current dev shell. Direct `cargo-deny check` runs but the full workspace gate is already red on `RUSTSEC-2026-0204` (`crossbeam-epoch 0.9.18`) and an unallowlisted `winx 0.36.4` license expression; neither dependency is in this new core crate's locked dependency tree.
- [ ] [serial] Run Cairn validation and proposal/design/tasks gates; sync and archive only with positive/negative evidence and without promoting experimental artifacts. r[kernelscript_experiment.verification]
  - Cairn validation plus proposal/design/tasks gates were refreshed and passed for this partial slice as recorded in `evidence/bounded-functional-core-2026-07-12.md`, but this closeout task remains unchecked and sync/archive were not performed because authoritative compiler/target inputs and execution evidence are absent.
