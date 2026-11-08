# 2026-11-08: captured git repositories for the Cargo-free plan

This continues [source program obligations](2026-11-08-source-program-obligations.md).
Every statement below comes from captured output in this directory. Preparation
still has not succeeded, no provider ran, and no fixed point is accepted:
I3–I5 and V1–V3 remain unchecked.

## Starting boundary

Repeating the planning probe over the hydrated source and the per-crate vendor
mirror left two package/target blockers, both intra-repository path dependencies
of git-sourced packages:

```text
missing-path-dependency-manifest  `turso_core` → vendor-deps/turso/../../core/Cargo.toml
missing-path-dependency-manifest  `ucan`      → vendor-deps/basalt/vendor/ucan/Cargo.toml
```

The pinned revisions do contain both sources: turso keeps `core/` beside its
bindings, and basalt commit `4c5c9e91…` contains `vendor/ucan/**` (699 entries)
beside `ucan.snapshot`. Only the per-crate vendored layout lost them.

## Captured repository trees

`scripts/capture-git-repositories.py` materialises every git source in the frozen
`Cargo.lock` at its locked revision under
`<vendor root>/git-repositories/<host>/<path>`, preferring local repository
databases and fetching only when a revision is absent. It rejects a link that
escapes the repository and replaces an in-tree link whose target contains `..`
with a copy of the resolved content, because the bundle's symlink contract admits
no `..` component.

Captured in this session (`vendor-inputs/git-repositories-capture.json`,
BLAKE3 `08750f74c1c6364ddb460d268b7ab46612390de862a1bb0f95119e25fef3234d`):
11 repositories, 10,298 files, 14 materialized symlinks. Ten came from local
repository databases; `github.com/cachix/turso` was fetched at
`dca55133caa690f90dcdd58d3c4329fb0703659c`.

## Planner resolution

A git-sourced lock package now resolves inside its captured repository when one
is present ([ADR 0146](../../../../adr/0146-resolve-git-packages-inside-captured-repository-trees.md)):
the member manifest is the unique `Cargo.toml` declaring that package name and
its effective version, inherited `version.workspace = true` resolves from the
nearest ancestor workspace manifest, and a workspace-derived fact (package
version, edition, environment, workspace dependency inheritance) for a captured
member uses that same repository workspace rather than the planning root.
Several matches or none leave the package unresolved; traversal order never
chooses. Per-crate vendored directories remain the fallback for registry sources.

## Vendor record refresh

`mantle source bundle refresh-vendor-inputs` replaces only the
`vendored-cargo-inputs` record of a verified profile, preserving the other 65
records and their identities. Applied to the refreshed profile:

```text
input_manifest_blake3   e2f0191b78a6715e7c12394166aae1fc07b0fa6b79a9567cb2d4d54754ce5020
output_manifest_blake3  a6174213ff6b336e251175ee6afbf452178502209d8773f2eb4cb09ba4e368e0
preserved_record_count  65
added_record_count      0
```

The new profile is
`/datapool/mantle-store/cold-inputs-20261030/profile/source-built-fixed-point-sources-refreshed-20261108-vendor-git.json`
(file BLAKE3 `9839769d886b39d8945cdf3eb0d0db62dd677779096c3bf09c6a2a3bac528472`).
The operator adopted `e2f0191b…` for the Mantle source record in this session;
`a6174213…` extends it with the captured repositories. The StageX lineage and
native-provider pins were not changed.

## Resulting plan blockers

With captured repositories the two path-dependency blockers disappear and the
package/target layer narrows to seven concrete blockers
(`2026-11-08-captured-git-plan-probe.json`):

- `unsupported-workspace-dependency-options` for inherited `serde` and `tokio`
  workspace dependencies;
- `missing-workspace-dependency-manifest` for
  `bounded-tree-core` (captured `seed.radicle.garden/zqhtZvste…` repository);
- `git-dependency-reference-mismatch` for `transactional-reconciliation-core`.

Adjacent layers report 14 unit, 40 host-unit and 54 derivation blockers that
are the propagated consequence of those package facts, plus
`unresolved-*-dependency-edge` entries for members the package layer never
admitted (`turso`, `ucan`, `artifact-auth-core`, `bounded-tree-cap`). No source
was substituted and no bound was lowered to reach this state.

Three workspace-inheritance repairs followed from those readings, each matching
Cargo's documented behavior for captured members:

- additive member `features` on an inherited dependency are unioned with the
  inherited set, and a member `default-features` decision may not contradict the
  inherited one (`inherited_workspace_dependency_features_are_additive`);
- a member may mark an inherited dependency `optional`; only platform selection
  remains unsupported;
- a `[workspace.dependencies]` `path` resolves relative to the workspace root,
  never to the inheriting member.

Two further admissions followed. Cargo sets no `--cfg` for this build, so a
bare flag name in a target table is unset and therefore false; malformed atoms
and unrecognized `key = value` predicates remain unadmitted
(`cfg(antithesis)`, `cfg(__ui_tests)` and mixed `all`/`not` expressions now
evaluate, while `cfg(target_osx = "linux")` still rejects).

Further admissions followed. `target_feature` predicates are false because
the plan passes no `-C target-feature`; a member `default-features` value
overrides the inherited one, matching Cargo's inherited-dependency override; and
only dev dependencies of the planning workspace's own members stay strict, so a
captured dependency's dev pin cannot block build-mode facts.

With those, **all four Cargo-free planning layers report ready** over the
captured-repository cohort: `source closure`, `native package/target planning`,
`native unit graph planning`, `native host-unit graph planning` and the
`unit derivation graph` have no blockers
(`2026-11-08-captured-git-plan-probe.json`).

Real authenticated preparation then advanced past planning and rejected a recipe
member path at the authoring boundary:

```json
{"error":"source recipe: non-normalized relative path \"crates/crunch-pipeline/../crunch-build/../crunch-rustc-wrapper/../crunch-rust-cache/../crunch-store/../../vendor/snix-build/build.rs\"","code":1,"kind":"build"}
```

That is the next slice: a plan-derived dependency chain reaches recipe authoring
without lexical normalization. The strict member-path admission is correct and
was not relaxed.

## Checks

`2026-11-08-captured-git-checks.log` is the combined focused run:

```text
captured repository layout + capture binding   2 passed
git source planning and resolution            6 passed
inherited feature/optionality/path handling   1 passed
cfg admission                                 1 passed
vendor input record refresh                   1 passed
Mantle source record refresh                  4 passed
dev-dependency build-mode regression          1 passed
```

Leaf `rustfmt --check`, `python3 -c ast.parse` of the capture script, strict
release Clippy with `-D warnings` and the release CLI build passed. The final
build identity is
`fa8112468982f18a7e9c5d31d21cce53ab366575482610d92c4c1ac6ca157128`.
The probe excerpt now records the ready plan state with full log BLAKE3
`33df1cceeec45d9b…`.
No prepared profile, provider execution, stage equality, fixed point,
determinism receipt or release eligibility is claimed.
