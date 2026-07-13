## Phase 1: Profile and compiler materialization

- [x] [serial] Define the typed beta experiment profile, source/compiler/toolchain/kernel refs, selected output classes, expected generated files, named bounds, and non-claims. r[kernelscript_experiment.profile]
- [x] [serial] Add pinned fixed-output KernelScript source and locked OCaml/dune/opam compiler derivations with offline build/codegen behavior and BLAKE3 identities. r[kernelscript_experiment.compiler]
  - Evidence: the official `v0.1.2` archive is fixed by upstream SHA-256 and measured BLAKE3; the compiler and codegen run offline from the locked nixpkgs OCaml/dune/menhir/library closure with no ambient opam or upstream binary. The production adapter remeasures the archive SHA-256 plus compiler/closure BLAKE3 identities and admits the exact compiler observation through the core. See `evidence/probe-production-evidence.md`.
- [x] [parallel] Add compiler/source drift, missing dependency closure, network attempt, unsupported target, and bound-exceeded negative fixtures. r[kernelscript_experiment.compiler] r[kernelscript_experiment.verification]

## Phase 2: Code generation and explicit build planning

- [x] [serial] Implement the separate codegen derivation and pure generated-project manifest parser/classifier with exact expected-file checks. r[kernelscript_experiment.codegen]
  - Evidence: the pinned Nix route executes compiler codegen for both reviewed fixtures, then its thin `crunch-kernelscript-adapter` shell reads bounded no-follow bytes and invokes the pure core classifier, planner, and receipt constructor. The structural check proves exact probe/kfunc member parity and rejects the old shell-owned `find`/`diff` classifier. See `evidence/probe-production-evidence.md`.
- [x] [serial] Implement a Mantle-owned pure compilation planner for allowlisted userspace, eBPF, optional module, and test steps; retain but never execute generated Makefiles. r[kernelscript_experiment.artifacts]
- [x] [serial] Implement target kernel-build/BTF/header/config/architecture admission and fail closed on absent, ambient-only, or mismatched inputs. r[kernelscript_experiment.kernel_inputs]
- [ ] [serial] Build and statically inspect each selected output class independently with exact member and receipt identities. r[kernelscript_experiment.artifacts]
  - Partial: independent bounded ELF/BTF/module shape inspection exists in the core. The pinned Nix route now invokes the core for generated manifests, compilation plans, and blocked receipts, then builds and externally inspects the probe eBPF object, skeleton, and loader. It does not invoke the core output-inspection path and does not build the module/test classes. See `evidence/probe-production-evidence.md`.

## Phase 3: Candidate handoff and evidence

- [x] [depends:mantle.add-onix-kernel-bundle-oci-projections] Emit frontend-neutral experimental ModulePack/BPF Pack candidate projections with target bindings and no deployability claim. r[kernelscript_experiment.handoff]
- [x] [serial] Add codegen/build receipts binding source, compiler closure, target inputs, plans, generated manifest, outputs, inspections, blockers, and non-claims with BLAKE3. r[kernelscript_experiment.evidence]
- [ ] [parallel] Add positive fixtures for a small userspace+probe program and one separately gated kfunc/module case under an exact kernel cohort. r[kernelscript_experiment.verification]
  - Partial: reviewed `.ks` fixtures now generate under the locked Linux `6.18.20` cohort, and the probe object/loader pass structural plus exact-kernel verifier/load/attach/detach VM gates. The checked route does not build or VM-load the module/kfunc case, and reported OnixOS Git identifiers are metadata rather than materialized authority. See `evidence/probe-production-evidence.md`.
- [x] [parallel] Add negative fixtures for generated-file drift/extra files, forbidden Makefile execution, unknown command/path, missing BTF/headers, kernel mismatch, BPF/module compile failure, malformed ELF/BTF metadata, stale output, receipt leak, and production overclaim. r[kernelscript_experiment.verification]

## Phase 4: Validation and closeout

- [x] [parallel] Document the pinned cohort, beta status, reproducible command path, generated-source review, pack handoff, ChaosControl dependency, upgrade procedure, and non-claims. r[kernelscript_experiment.profile] r[kernelscript_experiment.evidence]
- [ ] [serial] Run focused profile, compiler, codegen, planner, build, static-inspection, schema/receipt, formatting, clippy, and dependency-audit checks. r[kernelscript_experiment.verification]
  - Partial: pure-core, adapter positive/negative/parity tests, Nickel integration, wasm no-std, focused formatting/clippy, and the core-backed pinned compiler/codegen/probe structural derivation pass. Full validation remains blocked on checked module/kfunc build/load evidence and the previously recorded workspace dependency-audit blockers. The Nickel formatter remains unavailable in the current dev shell. See `evidence/probe-production-evidence.md`.
- [ ] [serial] Run Cairn validation and proposal/design/tasks gates; sync and archive only with positive/negative evidence and without promoting experimental artifacts. r[kernelscript_experiment.verification]
  - Current explicit-policy Cairn validation plus proposal/design/tasks gates pass for the core-adapter evidence, but this closeout task remains unchecked: sync/archive are forbidden for this session and checked module/kfunc build/load authority remains absent.
