## Phase 1: Resume core and shell

- [x] [depends:dev-cache-source-built-fixed-point] [serial] I1 Record the current marker, provider-adoption, persistent-store, and staging-directory behavior before implementation. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
- [x] [serial] I2 Add a pure bounded resume planner that revalidates source, plan, policy, stage, producer, and output identities. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  Evidence: `crunch-dev-resume-core` is `no_std + alloc`; 15 positive and negative tests pass, including stale identity, conflicting candidate, candidate bound, and malformed report cases.
- [x] [serial] I3 Publish and restore content-addressed stage bundles that include the required transition execution tree and stage outputs. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  Evidence: `8e941df2` adds dev-only 1/2/14/17-payload provider prefixes, shared immutable objects, publication at each provider boundary, and Mantle stage-1 publication before stage 2. Fresh-directory restoration works after removal of the original transition attempt. The Nix integration passed 108 tests with three ignored proofs. The publication-order gate passed three tests. V2 and V3 still own runtime confirmation. See `evidence/prefix-publication-2026-09-05/`.
- [x] [serial] I4 Continue from the first incomplete stage in a fresh staging directory and report restored and executed stages separately. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  Evidence: provider continuation handles transition, StageX, native, and full-provider prefixes; fixed-point continuation handles restored stage 1 and complete validation; `bootstrap.dev-resume-report` separates restored, executed, published, and rejected identities.

## Phase 2: Verification

- [x] [parallel] V1 Add positive fresh-directory resume tests and negative tests for stale, modified, partial, unknown, mismatched, or promoted-path bundles. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  Evidence: the updated Nix integration passed 108 positive and negative tests with three ignored long proofs. Its separate publication-order check passed three tests. Architecture has zero findings across nine negative fixtures and missing-marker controls. Machine contracts pass at 34 contracted and 68 classified surfaces.
- [ ] [depends:prove-source-built-mantle-fixed-point] [serial] V2 Run a complete cold-to-cached-to-adopt dev cycle and record exact source, plan, provider, store, stage, report, and alias evidence. r[source_built_fixed_point_improved_iteration.dev_resume_runtime_confirmation]
- [ ] [depends:prove-source-built-mantle-fixed-point] [serial] V3 Run a fresh promoted cold proof and make sure that it starts from empty authority, ignores dev state, and emits no cache-adoption disposition. r[source_built_fixed_point_improved_iteration.dev_resume_runtime_confirmation]
- [ ] [serial] V4 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, all three gates, and relevant Nix checks. r[source_built_fixed_point_improved_iteration.dev_resume_runtime_confirmation]
