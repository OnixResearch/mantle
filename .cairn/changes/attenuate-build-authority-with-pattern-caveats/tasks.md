# Tasks: Attenuate build authority with pattern caveats

All tasks remain open. Creating this proposal is not producer acceptance.

## Phase 1: Baseline and stack evaluation

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current ticket, store-view, and project-scope admission paths, including one restriction observation per site. r[mantle.authority_attenuation.caveat_filter_semantics]
- [ ] [serial] T1.2 Evaluate UCAN and Basalt for pattern caveats: filter expressiveness, composition, revocation, and proof-chain transport. Record the decision in an ADR. r[mantle.authority_attenuation.stack_authority_reuse]
- [ ] [serial] T1.3 Define the accepted caveat set, chain-length bound, pattern-size bound, and fail-closed parsing rules. r[mantle.authority_attenuation.caveat_filter_semantics]

## Phase 2: Core filter language

- [ ] [serial] T2.1 Implement pure pattern matching with bindings, template instantiation, rewrite, reject, alternative lists, and unknown-rejects-everything. r[mantle.authority_attenuation.caveat_filter_semantics]
- [ ] [serial] T2.2 Implement right-to-left composition and prove that composing chains cannot widen a grant. r[mantle.authority_attenuation.composition_order]
- [ ] [parallel] T2.3 Add filter fixtures: rewrite with bindings, reject, alternatives, unknown caveat, oversized pattern, and malformed caveat. r[mantle.authority_attenuation.caveat_filter_semantics]

## Phase 3: Grant sites

- [ ] [serial] T3.1 Restrict a remote builder grant to a declared job set, output class, and deadline, with receiver-side checking before job admission. r[mantle.authority_attenuation.attenuated_remote_grant]
- [ ] [serial] T3.2 Restrict a store view to a declared logical-path pattern and prove no read outside the pattern. r[mantle.authority_attenuation.attenuated_store_view]
- [ ] [serial] T3.3 Give project configuration a rewritten, project-namespaced goal capability that cannot request another project's goals. r[mantle.authority_attenuation.attenuated_remote_grant]
- [ ] [serial] T3.4 Report the effective caveat chain with each issued grant. r[mantle.authority_attenuation.composition_order]

## Phase 4: Fixtures and verification

- [ ] [parallel] T4.1 Add positive fixtures: a restricted grant authorizes its declared job, a further attenuation composes, and a view serves only its pattern. r[mantle.authority_attenuation.attenuated_remote_grant]
- [ ] [parallel] T4.2 Add negative fixtures: second job refused, store-view escape refused, cross-project goal refused, unknown caveat rejects everything, chain over bound rejected, and rejection produces no feedback. r[mantle.authority_attenuation.attenuated_store_view]
- [ ] [serial] T4.3 Run the filter, composition, and grant-site rails before and after the change. Preserve exact results. r[mantle.authority_attenuation.composition_order]
- [ ] [serial] T4.4 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[mantle.authority_attenuation.caveat_filter_semantics]
- [ ] [serial] T4.5 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.authority_attenuation.stack_authority_reuse]
