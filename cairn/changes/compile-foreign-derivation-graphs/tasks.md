# Tasks

## Phase 1: Prefix-aware producer boundary

- [x] [serial] I1 Add pure prefix-aware ATerm parsing for declared foreign store prefixes, bounded fields, fixed-output metadata, and exact input edges. r[foreign_derivation_import.prefix_aware_aterm]
- [x] [serial] I2 Add a thin `foreign-import produce-aterm` shell for explicit path-to-file mappings and directory bundles. Keep `produce-nix` as a compatibility surface. r[foreign_derivation_import.prefix_aware_aterm]
- [x] [parallel] I3 Add positive `/nix/store` and `/gnu/store` fixtures plus negative malformed, mixed-prefix, missing-input, duplicate-key, non-UTF-8, and oversized fixtures. r[foreign_derivation_import.prefix_aware_aterm]

## Phase 2: Exact graph compiler

- [x] [serial] I4 Add deterministic bounded dependency ordering with cycle, missing-node, duplicate-edge, root, and full-node-coverage checks. r[foreign_derivation_import.exact_graph_compilation]
- [x] [serial] I5 Add typed derivation, output, and source path maps. Rewrite exact store objects with suffix preservation and reject all unknown or leftover foreign references. r[foreign_derivation_import.exact_graph_compilation]
- [x] [serial] I6 Refactor `crunch-glue` with a pure resolved-registration helper that computes target HDM, output paths, derivation path, and pending registration without weak preliminary identity. r[foreign_derivation_import.exact_graph_compilation]
- [x] [parallel] I7 Add two-node and multi-output tests that prove each parent receives the exact recomputed child path. Add negative collision, cycle, unknown-reference, output-name, and partial-plan tests. r[foreign_derivation_import.exact_graph_compilation]

## Phase 3: Builtins and identity domains

- [x] [serial] I8 Lower declared Guix-like `builtin:download` and Git download nodes to bounded Mantle fetch facts. Reject every other builtin. r[foreign_derivation_import.foreign_builtin_lowering]
- [ ] [serial] I9 Preserve foreign fixed-output and SHA-256 facts while labeling Mantle BLAKE3 plan and target identities. Reject cross-domain digest substitution. r[foreign_derivation_import.foreign_builtin_lowering]
- [ ] [parallel] I10 Add ordered-mirror, fixed-output, executable-download, Git revision, malformed-hash, empty-candidate, unsupported-builtin, and digest-domain tests. r[foreign_derivation_import.foreign_builtin_lowering]

## Phase 4: Executable plan and lifecycle evidence

- [ ] [serial] I11 Emit deterministic `mantle-foreign-executable-plan-v1` artifacts with accepted import identity, selected roots, resolved units, exact path maps, source requirements, profile references, diagnostics, and non-claims. r[foreign_derivation_import.executable_plan]
- [ ] [parallel] I12 Update the trust-model guide and machine-artifact documentation. State that executable plans do not prove source availability, realization, store admission, or output trust. r[foreign_derivation_import.executable_plan]
- [ ] [serial] V1 Run `nix develop -c cargo test -p mantle --bin mantle foreign_derivation_import`, `nix develop -c cargo test -p crunch-glue`, and `nix develop -c cargo test -p mantle --test foreign_import_cli`. Record exact output in `cairn/changes/compile-foreign-derivation-graphs/evidence/verification.md`. r[foreign_derivation_import.exact_graph_compilation]
- [ ] [serial] V2 Run `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs`, `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test`, and `git diff --check`. Record an inline summary and the exact transcript. r[foreign_derivation_import.executable_plan]
- [ ] [serial] V3 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, all three Cairn gates for this change, and `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .`. Archive only after the transcript is current. r[foreign_derivation_import.executable_plan]
